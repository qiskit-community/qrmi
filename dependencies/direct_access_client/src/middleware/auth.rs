//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use http::Extensions;
#[allow(unused_imports)]
use log::{debug, error};
use reqwest::{header::HeaderValue, Client, Request, Response};
use reqwest_middleware::{Middleware, Next};
use reqwest_retry::{policies::ExponentialBackoff, Jitter};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use tokio::sync::Mutex;

use crate::models::{
    auth::GetAccessTokenResponse, errors::ErrorResponse, errors::IAMErrorResponse,
};
use crate::AuthMethod;

const DEFAULT_RETRIES: u32 = 5;
const DEFAULT_INITIAL_RETRY_INTERVAL: f64 = 1.0;
const DEFAULT_MAX_RETRY_INTERVAL: f64 = 10.0;
const DEFAULT_EXPONENTIAL_BASE: u32 = 2;

pub(crate) struct TokenManager {
    access_token: Option<String>,
    token_expiry: Option<Instant>,
    client: reqwest_middleware::ClientWithMiddleware,
    token_url: String,
    auth_method: AuthMethod,
}
impl TokenManager {
    pub(crate) fn new(
        token_url: impl Into<String>,
        auth_method: AuthMethod,
        timeout: Option<Duration>,
        connect_timeout: Option<Duration>,
        read_timeout: Option<Duration>,
        retry_policy: Option<reqwest_retry::policies::ExponentialBackoff>,
    ) -> Result<Self> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);
        if let Some(v) = timeout {
            reqwest_client_builder = reqwest_client_builder.timeout(v)
        }

        if let Some(v) = read_timeout {
            reqwest_client_builder = reqwest_client_builder.read_timeout(v)
        }

        if let Some(v) = connect_timeout {
            reqwest_client_builder = reqwest_client_builder.connect_timeout(v)
        }
        let mut reqwest_builder =
            reqwest_middleware::ClientBuilder::new(reqwest_client_builder.build()?);
        if let Some(v) = retry_policy {
            reqwest_builder =
                reqwest_builder.with(reqwest_retry::RetryTransientMiddleware::new_with_policy(v))
        } else {
            let default_policy = ExponentialBackoff::builder()
                .retry_bounds(
                    Duration::from_secs_f64(DEFAULT_INITIAL_RETRY_INTERVAL),
                    Duration::from_secs_f64(DEFAULT_MAX_RETRY_INTERVAL),
                )
                .jitter(Jitter::Bounded)
                .base(DEFAULT_EXPONENTIAL_BASE)
                .build_with_max_retries(DEFAULT_RETRIES);
            reqwest_builder = reqwest_builder.with(
                reqwest_retry::RetryTransientMiddleware::new_with_policy(default_policy),
            )
        }
        Ok(Self {
            access_token: None,
            token_expiry: None,
            client: reqwest_builder.build(),
            token_url: token_url.into(),
            auth_method,
        })
    }
    async fn get_access_token(&mut self) -> reqwest_middleware::Result<()> {
        #[cfg(feature = "ibmcloud_appid_auth")]
        if let AuthMethod::IbmCloudAppId { username, password } = self.auth_method.clone() {
            use base64::{engine::general_purpose::STANDARD, prelude::*};
            let base64_str = STANDARD.encode(format!("{}:{}", username, password).as_bytes());
            let response = self
                .client
                .post(&self.token_url)
                .header(reqwest::header::ACCEPT, "application/json")
                .header(
                    reqwest::header::AUTHORIZATION,
                    format!("Basic {}", base64_str),
                )
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .await?;
            let status = response.status();
            if status.is_success() {
                let token_response: GetAccessTokenResponse = response.json().await?;
                self.access_token = Some(token_response.access_token);
                self.token_expiry =
                    Some(Instant::now() + Duration::from_secs(token_response.expires_in));
            } else {
                let reason = status.canonical_reason().unwrap_or_default().to_string();
                return Err(reqwest_middleware::Error::Middleware(anyhow!(format!(
                    "{} ({})",
                    reason, status
                ))));
            }
        }
        if let AuthMethod::IbmCloudIam { apikey, .. } = self.auth_method.clone() {
            let params = [
                ("grant_type", "urn:ibm:params:oauth:grant-type:apikey"),
                ("apikey", &apikey),
            ];
            let response = self
                .client
                .post(&self.token_url)
                .header(reqwest::header::ACCEPT, "application/json")
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .form(&params)
                .send()
                .await?;
            let status = response.status();
            if status.is_success() {
                let token_response: GetAccessTokenResponse = response.json().await?;
                self.access_token = Some(token_response.access_token);
                self.token_expiry = Some(
                    Instant::now()
                        + Duration::from_secs((token_response.expires_in as f64 * 0.9) as u64),
                );
            } else {
                let _error_response = response.json::<IAMErrorResponse>().await;
                match _error_response {
                    Ok(error_response) => {
                        let reason_text: String;
                        if let Some(details) = error_response.details {
                            reason_text = details.clone();
                        } else {
                            reason_text = error_response.message.clone();
                        }
                        return Err(reqwest_middleware::Error::Middleware(anyhow!(format!(
                            "Failed to obtain access token. reason: {} ({})",
                            reason_text, error_response.code
                        ))));
                    }
                    Err(_) => {
                        let reason = status.canonical_reason().unwrap_or_default().to_string();
                        return Err(reqwest_middleware::Error::Middleware(anyhow!(format!(
                            "Failed to obtain access token. reason: {} ({}), url: {}",
                            reason, status, &self.token_url
                        ))));
                    }
                }
            }
        }

        Ok(())
    }
    async fn ensure_token_validity(&mut self) -> reqwest_middleware::Result<()> {
        if self.access_token.is_none()
            || self.token_expiry.unwrap_or_else(Instant::now) <= Instant::now()
        {
            self.get_access_token().await?;
        }
        Ok(())
    }
    async fn get_token(&mut self) -> reqwest_middleware::Result<String> {
        self.ensure_token_validity().await?;
        Ok(self.access_token.clone().unwrap())
    }
}

#[derive(Clone)]
pub(crate) struct AuthMiddleware {
    token_manager: Arc<Mutex<TokenManager>>,
}
impl AuthMiddleware {
    pub(crate) fn new(token_manager: Arc<Mutex<TokenManager>>) -> Self {
        Self { token_manager }
    }
}
#[async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(
        &self,
        request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let mut token_manager = self.token_manager.lock().await;
        let token = token_manager.get_token().await?;
        // add authentication header to the request
        let mut cloned_req = request.try_clone().unwrap();
        debug!("current token {}", token);
        cloned_req.headers_mut().insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );

        // send a request
        let response = next
            .clone()
            .run(cloned_req.try_clone().unwrap(), extensions)
            .await;

        // retry if token is expired.
        if response.is_err()
            || response.as_ref().unwrap().status() == reqwest::StatusCode::UNAUTHORIZED
        {
            debug!("renew access token");
            token_manager.get_access_token().await?;
            let token = token_manager.get_token().await?;
            debug!("new token {}", token);
            let mut new_request = request.try_clone().unwrap();
            new_request.headers_mut().insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
            return next.clone().run(new_request, extensions).await;
        }
        response
    }
}
