//
// (C) Copyright Pasqal SAS 2025, 2026
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

use crate::models;
use log::debug;
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

pub const DEFAULT_BASE_URL: &str = "https://apis.pasqal.cloud";

/// An asynchronous `Client` to make Requests with.
/// The `Client` handles authentication and provides methods for all Pasqal Cloud API endpoints.
/// The authentication logic is implemented in the `auth` module,
/// but the main `Client` logic is implemented below.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
    /// HTTP client to interact with Pasqal Cloud service
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
    pub(crate) project_id: String,
    pub(crate) auth_token: String,
    pub(crate) auth_header: Option<header::HeaderValue>,
    pub(crate) auth_endpoint: String,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<String>,
    /// How many times a failed HTTP request (including token refresh) is
    /// retried, or `None` to not retry at all.
    pub(crate) max_retries: Option<u32>,
}

impl Client {
    pub async fn get_device(
        &mut self,
        device_type: models::DeviceType,
    ) -> Result<models::GetDeviceResponseData> {
        let url = format!(
            "{}/core-fast/api/v1/devices?device_type={}",
            self.base_url, device_type,
        );
        let resp: models::Response<Vec<models::GetDeviceResponseData>> =
            self.get_unauthenticated(&url).await?;

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
        device_type: models::DeviceType,
    ) -> Result<models::Response<models::CreateBatchResponseData>> {
        let url = format!("{}/core-fast/api/v1/batches", self.base_url);
        let batch = models::Batch {
            sequence_builder: sequence,
            jobs: Vec::from([models::Job { runs: job_runs }]),
            device_type: device_type.to_string(),
            project_id: self.project_id.clone(),
        };
        self.post(&url, batch).await
    }

    // CUDA-Q jobs are created with a separate endpoint from normal batches,
    // and the payload is different as well.
    pub async fn create_cudaq_job(
        &mut self,
        sequence: Value,
        shots: i32,
        device_type: models::DeviceType,
    ) -> Result<models::Response<models::CreateCudaqJobResponseData>> {
        let url = format!("{}/core-fast/api/v1/cudaq/job", self.base_url);
        let payload = serde_json::json!({
            "machine": device_type.to_string(),
            "shots": shots,
            "project_id": self.project_id,
            "sequence": sequence,
        });
        self.post(&url, payload).await
    }

    pub async fn cancel_batch(
        &mut self,
        batch_id: &str,
    ) -> Result<models::Response<models::CancelBatchResponseData>> {
        let url = format!(
            "{}/core-fast/api/v2/batches/{}/cancel",
            self.base_url, batch_id
        );
        self.patch(&url).await
    }

    pub async fn get_batch(
        &mut self,
        batch_id: &str,
    ) -> Result<models::Response<models::GetBatchResponseData>> {
        let url = format!("{}/core-fast/api/v2/batches/{}", self.base_url, batch_id);
        self.get(&url).await
    }

    // Why are we doing this with a separate endpoint from get_batch?
    // The get_batch endpoint returns a list of job ids but not the actual results,
    // and the get_cudaq_job endpoint returns the results directly.
    // Originally to make things easier for the CUDA-Q integration.
    // For QRMI  we could consider just using the normal endpoint.
    pub async fn get_cudaq_job(
        &mut self,
        job_id: &str,
    ) -> Result<models::Response<models::GetCudaqJobResponseData>> {
        let url = format!("{}/core-fast/api/v1/cudaq/job/{}", self.base_url, job_id);
        self.get(&url).await
    }

    pub async fn get_job(
        &mut self,
        job_id: &str,
    ) -> Result<models::Response<models::GetJobResponseData>> {
        let url = format!("{}/core-fast/api/v2/jobs/{}", self.base_url, job_id);
        self.get(&url).await
    }

    pub async fn get_batch_results(&mut self, batch_id: &str) -> Result<String> {
        let url = format!(
            "{}/core-fast/api/v1/batches/{}/full_results",
            self.base_url, batch_id
        );

        let resp: models::Response<HashMap<String, models::JobResult>> = self.get(&url).await?;

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

    pub async fn get_device_specs(&mut self, device_type: models::DeviceType) -> Result<String> {
        // In case of emulator devices, we return public specs to enable users to build sequences
        // according to current cloud specs
        if device_type.to_string().starts_with("EMU") {
            let url = format!("{}/core-fast/api/v1/devices/public-specs", self.base_url);
            let raw_response: models::Response<Vec<models::GetDeviceSpecsResponseData>> =
                self.get_unauthenticated(&url).await?;
            let specs_string = serde_json::to_string(&raw_response.data)?;
            Ok(specs_string)
        } else {
            let url = format!(
                "{}/core-fast/api/v1/devices/specs/{}",
                self.base_url, device_type
            );
            let raw_response: models::Response<models::GetDeviceSpecsResponseData> =
                self.get(&url).await?;
            // Matching the data structure from the /public-specs endpoint
            let specs_string = serde_json::to_string(&vec![raw_response.data])?;
            Ok(specs_string)
        }
    }
}

impl Client {
    fn build_http_client(
        max_retries: Option<u32>,
    ) -> Result<reqwest_middleware::ClientWithMiddleware> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);
        let mut reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);
        if let Some(max_retries) = max_retries {
            reqwest_builder = pasqal_common::with_retry(reqwest_builder, max_retries);
        }
        Ok(reqwest_builder.build())
    }

    pub(crate) async fn get<T: DeserializeOwned>(&mut self, url: &str) -> Result<T> {
        let resp = self
            .authenticated(self.client.get(url))
            .await?
            .send()
            .await?;
        self.handle_request(resp).await
    }

    // Unauthenticated GET request for public endpoints
    pub(crate) async fn get_unauthenticated<T: DeserializeOwned>(
        &mut self,
        url: &str,
    ) -> Result<T> {
        let resp = self.client.get(url).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn patch<T: DeserializeOwned>(&mut self, url: &str) -> Result<T> {
        let resp = self
            .authenticated(self.client.patch(url))
            .await?
            .send()
            .await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn post<T: DeserializeOwned, U: Serialize>(
        &mut self,
        url: &str,
        body: U,
    ) -> Result<T> {
        let resp = self
            .authenticated(self.client.post(url))
            .await?
            .json(&body)
            .send()
            .await?;
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
    client_id: Option<String>,
    client_secret: Option<String>,
    /// How many times a failed HTTP request is retried, or `None` to not retry
    /// at all. Defaults to [`pasqal_common::DEFAULT_MAX_RETRIES`]; changed with
    /// [`Self::with_max_retries`] or [`Self::with_retry_disabled`].
    max_retries: Option<u32>,
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_cloud_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("your_project_id".to_string());
    /// ```
    pub fn new(project_id: String) -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            token: String::new(),
            project_id,
            auth_endpoint: String::new(),
            username: None,
            password: None,
            client_id: None,
            client_secret: None,
            max_retries: Some(pasqal_common::DEFAULT_MAX_RETRIES),
        }
    }

    /// Retry a failed HTTP request at most `max_retries` times.
    pub fn with_max_retries(&mut self, max_retries: u32) -> &mut Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Disable HTTP request retries entirely.
    ///
    /// Retries are enabled by default.
    pub fn with_retry_disabled(&mut self) -> &mut Self {
        self.max_retries = None;
        self
    }

    pub fn with_base_url(&mut self, base_url: String) -> &mut Self {
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

    pub fn with_service_account_credentials(
        &mut self,
        client_id: String,
        client_secret: String,
    ) -> &mut Self {
        self.client_id = Some(client_id);
        self.client_secret = Some(client_secret);
        self
    }

    /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_cloud_api::ClientBuilder;
    ///
    /// let mut builder = ClientBuilder::new("your_project_id".to_string());
    /// builder.with_token("your_token".to_string());
    /// let _client = builder.build();
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        debug!(
            "Initialize Client (project_id set: {}, auth_token set: {}, username/password set: {}/{}, client_id/client_secret set: {}/{})",
            !self.project_id.trim().is_empty(),
            !self.token.trim().is_empty(),
            self.username.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
            self.password.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
            self.client_id.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
            self.client_secret.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false),
        );
        Ok(Client {
            base_url: self.base_url.clone(),
            client: Client::build_http_client(self.max_retries)?,
            project_id: self.project_id.clone(),
            auth_token: self.token.clone(),
            auth_header: Client::make_auth_header(&self.token)?,
            auth_endpoint: self.auth_endpoint.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            max_retries: self.max_retries,
        })
    }
}
