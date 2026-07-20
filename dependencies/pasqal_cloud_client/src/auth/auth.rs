// This code is part of Qiskit.
//
// (C) Copyright Pasqal 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::client::Client;
use crate::models::{AuthError, AuthTokenResponse};
use anyhow::{bail, Result};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use log::debug;
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

const AUTH_TOKEN_EXPIRY_GRACE_SECONDS: i64 = 10;
const AUTH_GRANT_TYPE: &str = "http://auth0.com/oauth/grant-type/password-realm";
const AUTH_SERVICE_ACCOUNT_GRANT_TYPE: &str = "client_credentials";
const AUTH_REALM: &str = "pcs-users";
const AUTH_CLIENT_ID: &str = "PeZvo7Atx7IVv3iel59asJSb4Ig7vuSB";
const AUTH_AUDIENCE: &str = "https://apis.pasqal.cloud/account/api/v1";

/// Credentials used to request a Pasqal Cloud access token.
pub enum AccessTokenRequest<'a> {
    /// Username/password authentication for regular users.
    UsernamePassword {
        username: &'a str,
        password: &'a str,
    },
    /// Client credentials authentication for service accounts.
    ServiceAccount {
        client_id: &'a str,
        client_secret: &'a str,
    },
}

impl AccessTokenRequest<'_> {
    fn form_params(&self) -> Vec<(&'static str, &str)> {
        match self {
            Self::UsernamePassword { username, password } => vec![
                ("grant_type", AUTH_GRANT_TYPE),
                ("realm", AUTH_REALM),
                ("client_id", AUTH_CLIENT_ID),
                ("audience", AUTH_AUDIENCE),
                ("username", username),
                ("password", password),
            ],
            Self::ServiceAccount {
                client_id,
                client_secret,
            } => vec![
                ("grant_type", AUTH_SERVICE_ACCOUNT_GRANT_TYPE),
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("audience", AUTH_AUDIENCE),
            ],
        }
    }
}

fn now_unix_seconds() -> Result<i64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64)
}

// Authentication-related functionality for Pasqal Cloud `Client`
impl Client {
    pub(crate) fn make_auth_header(token: &str) -> Result<Option<header::HeaderValue>> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        Ok(Some(header::HeaderValue::from_str(&format!(
            "Bearer {token}"
        ))?))
    }

    fn attach_headers(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest_middleware::RequestBuilder> {
        let req = req.header(reqwest::header::CONTENT_TYPE, "application/json");
        match &self.auth_header {
            Some(auth) => Ok(req.header(reqwest::header::AUTHORIZATION, auth.clone())),
            None => bail!("authorization header is not available"),
        }
    }

    pub(crate) async fn authenticated(
        &mut self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<reqwest_middleware::RequestBuilder> {
        self.ensure_authenticated().await?;
        self.attach_headers(req)
    }

    pub(crate) async fn ensure_authenticated(&mut self) -> Result<()> {
        let now = now_unix_seconds()?;
        if Self::is_auth_token_usable(&self.auth_token, now) {
            return Ok(());
        }

        if let Some(exp) = Self::jwt_expiry_unix_seconds(&self.auth_token)? {
            debug!(
                "Auth token is expired or near expiry (exp {}, now {}), will attempt to refresh",
                exp, now
            );
        }

        let token = if let (Some(username), Some(password)) =
            (self.username.as_deref(), self.password.as_deref())
        {
            debug!("Requesting new user auth token from Pasqal Cloud");
            Self::request_access_token(
                &self.auth_endpoint,
                AccessTokenRequest::UsernamePassword { username, password },
                self.max_retries,
            )
            .await?
        } else if let (Some(client_id), Some(client_secret)) =
            (self.client_id.as_deref(), self.client_secret.as_deref())
        {
            debug!("Requesting new service account auth token from Pasqal Cloud");
            Self::request_access_token(
                &self.auth_endpoint,
                AccessTokenRequest::ServiceAccount {
                    client_id,
                    client_secret,
                },
                self.max_retries,
            )
            .await?
        } else {
            return Err(AuthError::MissingCredentialsForRefresh.into());
        };
        self.auth_token = token;
        self.auth_header = Self::make_auth_header(&self.auth_token)?;
        Ok(())
    }

    /// Request a Pasqal Cloud access token.
    ///
    /// `max_retries` controls how many times the token request is retried on
    /// transient failures; pass `0` to disable retries.
    pub async fn request_access_token(
        auth_endpoint: &str,
        request: AccessTokenRequest<'_>,
        max_retries: u32,
    ) -> Result<String> {
        let auth_endpoint = Self::normalize_auth_endpoint(auth_endpoint)?;
        let client_params = request.form_params();

        let client_builder = ReqwestClientBuilder::new(reqwest::Client::new());
        let client = pasqal_common::with_retry(client_builder, max_retries).build();

        let resp = client
            .post(auth_endpoint)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .form(&client_params)
            .send()
            .await?;

        if resp.status().is_success() {
            let token: AuthTokenResponse = resp.json().await?;
            Ok(token.access_token)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("Token request failed: {} {}", status, body);
        }
    }

    fn normalize_auth_endpoint(auth_endpoint: &str) -> Result<String> {
        if auth_endpoint.trim().is_empty() {
            bail!("auth endpoint must be configured when requesting an access token");
        } else if auth_endpoint.contains("://") {
            Ok(auth_endpoint.trim().to_string())
        } else {
            Ok(format!("https://{}", auth_endpoint.trim()))
        }
    }

    /// Read `exp` from a JWT payload without validating the JWT signature.
    ///
    /// This helper is only used for local token-expiry checks to decide whether to refresh.
    /// Token authenticity is still enforced by the Pasqal Cloud API when requests are sent.
    pub fn jwt_expiry_unix_seconds(token: &str) -> Result<Option<i64>> {
        let token = token.trim();
        if token.is_empty() {
            return Ok(None);
        }

        let mut parts = token.split('.');
        let _header = match parts.next() {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };
        let payload = match parts.next() {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };
        if parts.next().is_none() {
            return Ok(None);
        }

        let payload_bytes = URL_SAFE_NO_PAD.decode(payload)?;
        let v: Value = serde_json::from_slice(&payload_bytes)?;
        let Some(exp) = v.get("exp") else {
            return Ok(None);
        };

        if let Some(n) = exp.as_i64() {
            Ok(Some(n))
        } else if let Some(s) = exp.as_str() {
            Ok(Some(s.parse::<i64>()?))
        } else {
            Ok(None)
        }
    }

    /// Check whether a cached auth token is usable at `now_unix_seconds`.
    ///
    /// Non-empty tokens without JWT expiry are treated as usable.
    /// JWT tokens are treated as expired if they are within the grace period of their expiry.
    pub fn is_auth_token_usable(token: &str, now_unix_seconds: i64) -> bool {
        if token.trim().is_empty() {
            return false;
        }

        match Self::jwt_expiry_unix_seconds(token) {
            Ok(Some(exp)) => exp > now_unix_seconds + AUTH_TOKEN_EXPIRY_GRACE_SECONDS,
            Ok(None) => {
                debug!(
                    "Auth token has no parseable JWT expiry; treating token as usable until rejected by API"
                );
                true
            }
            Err(err) => {
                debug!(
                    "Failed to parse auth token expiry from JWT payload; treating token as unusable: {}",
                    err
                );
                false
            }
        }
    }
}
