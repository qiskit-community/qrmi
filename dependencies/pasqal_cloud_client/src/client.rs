//
// (C) Copyright Pasqal SAS 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Pasqal Cloud API Client

use anyhow::{bail, Result};

use crate::models::batch::JobStatus;
use crate::models::device::DeviceType;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use log::debug;
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const DEFAULT_AUTH_ENDPOINT: &str = "authenticate.pasqal.cloud/oauth/token";
pub const DEFAULT_BASE_URL: &str = "https://apis.pasqal.cloud";
const AUTH_TOKEN_EXPIRY_GRACE_SECONDS: i64 = 10;
const AUTH_GRANT_TYPE: &str = "http://auth0.com/oauth/grant-type/password-realm";
const AUTH_REALM: &str = "pcs-users";
const AUTH_CLIENT_ID: &str = "PeZvo7Atx7IVv3iel59asJSb4Ig7vuSB";
const AUTH_AUDIENCE: &str = "https://apis.pasqal.cloud/account/api/v1";

fn now_unix_seconds() -> Result<i64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64)
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("auth token is missing or expired and username/password are not configured")]
    MissingCredentialsForRefresh,
}

/// An asynchronous `Client` to make Requests with.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
    /// HTTP client to interact with Pasqal Cloud service
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
    pub(crate) project_id: String,
    pub(crate) auth_token: String,
    pub(crate) auth_token_expiry_unix_seconds: Option<i64>,
    pub(crate) auth_endpoint: String,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceResponseData {
    pub status: String,
    pub availability: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceSpecsResponseData {
    pub specs: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBatchResponseData {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBatchResponseData {
    pub status: JobStatus,
    pub job_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetJobResponseData {
    pub status: JobStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelBatchResponseData {}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub runs: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct JobResult {
    counter: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Batch {
    pub sequence_builder: String,
    pub jobs: Vec<Job>,
    pub device_type: String,
    pub project_id: String,
}

impl Client {
    fn build_http_client(token: &str) -> Result<reqwest_middleware::ClientWithMiddleware> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        reqwest_client_builder = reqwest_client_builder.default_headers(headers);
        let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);
        Ok(reqwest_builder.build())
    }

    async fn ensure_authenticated(&mut self) -> Result<()> {
        // Ensure the client has a usable auth token, refreshing it in-place if necessary.
        // If the client is missing credentials to refresh the token, this will return an error instead.

        let now = now_unix_seconds()?;
        if Self::is_auth_token_usable(&self.auth_token, now) {
            return Ok(());
        }

        if let Some(exp) = self.auth_token_expiry_unix_seconds {
            debug!(
                "Auth token is expired or near expiry (exp {}, now {}), will attempt to refresh",
                exp, now
            );
        }

        let (Some(username), Some(password)) = (self.username.as_deref(), self.password.as_deref())
        else {
            return Err(AuthError::MissingCredentialsForRefresh.into());
        };

        debug!("Requesting new auth token from Pasqal Cloud");

        // Request a new token and update the client
        let token = Self::request_access_token(&self.auth_endpoint, username, password).await?;
        self.auth_token = token;
        self.auth_token_expiry_unix_seconds = Self::jwt_expiry_unix_seconds(&self.auth_token)?;
        self.client = Self::build_http_client(&self.auth_token)?;
        Ok(())
    }

    pub async fn get_device(&mut self, device_type: DeviceType) -> Result<GetDeviceResponseData> {
        let url = format!(
            "{}/core-fast/api/v1/devices?device_type={}",
            self.base_url, device_type,
        );
        let resp: Response<Vec<GetDeviceResponseData>> = self.get(&url).await?;

        resp.data
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No devices found for type {:?}", device_type))
    }

    /// Pasqal Cloud works with batches of jobs rather than
    /// individual jobs, see:
    /// https://docs.pasqal.com/cloud/batches/
    pub async fn create_batch(
        &mut self,
        sequence: String,
        job_runs: i32,
        device_type: DeviceType,
    ) -> Result<Response<CreateBatchResponseData>> {
        let url = format!("{}/core-fast/api/v1/batches", self.base_url);
        let batch = Batch {
            sequence_builder: sequence,
            jobs: Vec::from([Job { runs: job_runs }]),
            device_type: device_type.to_string(),
            project_id: self.project_id.clone(),
        };
        self.post(&url, batch).await
    }

    pub async fn cancel_batch(
        &mut self,
        batch_id: &str,
    ) -> Result<Response<CancelBatchResponseData>> {
        let url = format!(
            "{}/core-fast/api/v2/batches/{}/cancel",
            self.base_url, batch_id
        );
        self.patch(&url).await
    }

    pub async fn get_batch(&mut self, batch_id: &str) -> Result<Response<GetBatchResponseData>> {
        let url = format!("{}/core-fast/api/v2/batches/{}", self.base_url, batch_id);
        self.get(&url).await
    }

    pub async fn get_job(&mut self, job_id: &str) -> Result<Response<GetJobResponseData>> {
        let url = format!("{}/core-fast/api/v2/jobs/{}", self.base_url, job_id);
        self.get(&url).await
    }

    pub async fn get_batch_results(&mut self, batch_id: &str) -> Result<String> {
        let url = format!(
            "{}/core-fast/api/v1/batches/{}/full_results",
            self.base_url, batch_id
        );

        let resp: Response<HashMap<String, JobResult>> = self.get(&url).await?;

        let data = resp.data;

        // Ensure exactly one job
        match data.len() {
            0 => bail!("No results found"),
            1 => {
                let first_job_result = data.into_values().next().unwrap();
                // Return JSON string of job results
                Ok(serde_json::to_string(&first_job_result)?)
            }
            _ => bail!("Unexpected multiple jobs in one Pasqal cloud batch"),
        }
    }

    pub async fn get_device_specs(
        &mut self,
        device_type: DeviceType,
    ) -> Result<Response<GetDeviceSpecsResponseData>> {
        // Not authenticated
        let url = format!(
            "{}/core-fast/api/v1/devices/specs/{}",
            self.base_url, device_type
        );
        self.get(&url).await
    }

    pub(crate) async fn get<T: DeserializeOwned>(&mut self, url: &str) -> Result<T> {
        self.ensure_authenticated().await?;
        let resp = self.client.get(url).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn patch<T: DeserializeOwned>(&mut self, url: &str) -> Result<T> {
        self.ensure_authenticated().await?;
        let resp = self.client.patch(url).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn post<T: DeserializeOwned, U: Serialize>(
        &mut self,
        url: &str,
        body: U,
    ) -> Result<T> {
        self.ensure_authenticated().await?;
        let resp = self.client.post(url).json(&body).send().await?;
        self.handle_request(resp).await
    }

    async fn handle_request<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        if resp.status().is_success() {
            let json_text = resp.text().await?;
            debug!("{}", json_text);
            let val = serde_json::from_str(&json_text)?;
            Ok(val)
        } else {
            let status = resp.status();
            let json_text = resp.text().await?;
            bail!("Status: {}, Fail {}", status, json_text);
        }
    }

    /// Request a Pasqal Cloud access token using username/password.
    ///
    /// This uses the same Auth0 password-realm flow currently documented for Pasqal Cloud.
    pub async fn request_access_token(
        auth_endpoint: &str,
        username: &str,
        password: &str,
    ) -> Result<String> {
        let auth_endpoint = if auth_endpoint.trim().is_empty() {
            format!("https://{DEFAULT_AUTH_ENDPOINT}")
        } else if auth_endpoint.contains("://") {
            auth_endpoint.trim().to_string()
        } else {
            format!("https://{}", auth_endpoint.trim())
        };

        let client_params = [
            ("grant_type", AUTH_GRANT_TYPE),
            ("realm", AUTH_REALM),
            ("client_id", AUTH_CLIENT_ID),
            ("audience", AUTH_AUDIENCE),
            ("username", username),
            ("password", password),
        ];

        let resp = reqwest::Client::new()
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

/// A [`ClientBuilder`] can be used to create a [`Client`] with custom configuration.
#[must_use]
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// The base URL this client sends requests to
    base_url: String,
    token: String,
    project_id: String,
    auth_endpoint: String,
    username: Option<String>,
    password: Option<String>,
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_cloud_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("your_token".to_string(), "your_project_id".to_string());
    /// ```
    pub fn new(token: String, project_id: String) -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            token,
            project_id,
            auth_endpoint: DEFAULT_AUTH_ENDPOINT.to_string(),
            username: None,
            password: None,
        }
    }

    /// Construct a new [`ClientBuilder`] with project id.
    pub fn for_project(project_id: String) -> Self {
        Self::new(String::new(), project_id)
    }

    /// Set custom API base URL.
    pub fn base_url(&mut self, base_url: String) -> &mut Self {
        self.base_url = base_url;
        self
    }

    pub fn with_auth_endpoint(&mut self, auth_endpoint: String) -> &mut Self {
        self.auth_endpoint = auth_endpoint;
        self
    }

    pub fn with_token(&mut self, token: String) -> &mut Self {
        self.token = token;
        self
    }

    pub fn with_credentials(&mut self, username: String, password: String) -> &mut Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_cloud_api::ClientBuilder;
    ///
    /// let mut builder = ClientBuilder::new("your_token".to_string(), "your_project_id".to_string());
    /// let _client = builder.build();
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        debug!(
            "Initialize Client (project_id set: {}, auth_token set: {}, username/password set: {}/{})",
            !self.project_id.trim().is_empty(),
            !self.token.trim().is_empty(),
            self.username.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
            self.password.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
        );
        let auth_token_expiry_unix_seconds = Client::jwt_expiry_unix_seconds(&self.token)?;

        Ok(Client {
            base_url: self.base_url.clone(),
            client: Client::build_http_client(&self.token)?,
            project_id: self.project_id.clone(),
            auth_token: self.token.clone(),
            auth_token_expiry_unix_seconds,
            auth_endpoint: self.auth_endpoint.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct AuthTokenResponse {
    access_token: String,
}
