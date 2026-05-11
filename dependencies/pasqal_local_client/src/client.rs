//
// (C) Copyright Pasqal SAS 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Pasqal Cloud API Client

#[cfg(feature = "munge")]
use crate::munge;
use anyhow::{bail, Result};

use crate::models::job::JobStatus;
use log::debug;
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// An asynchronous `Client` to make Requests with.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
    /// HTTP client to interact with Pasqal Cloud service
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobResponse {
    pub id: i32,
    pub user_id: String,
    pub status: JobStatus,
    pub results: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateJob {
    pub sequence: String,
    pub shots: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccessibleResponse {
    pub is_accessible: bool,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionResponse {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceSpecsResponse {
    pub specs: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetTaskLogsResponse {
    pub logs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionPayload {
    pub user_id: String,
    pub slurm_job_id: String,
}

impl Client {
    /// Return authentication headers for request with a fresh munge token
    async fn create_headers(&self) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        // Generate fresh munge token for each request
        #[cfg(feature = "munge")]
        {
            let token = munge::encode(b"")?;
            headers.insert(
                reqwest::header::HeaderName::from_static("x-munge-cred"),
                reqwest::header::HeaderValue::from_str(&token).expect("invalid munge token"),
            );
            Ok(headers)
        }

        #[cfg(not(feature = "munge"))]
        {
            bail!("Munge support is disabled. Compile with --features munge to use the Pasqal Local client.")
        }
    }

    pub async fn get_jobs(&self) -> Result<Vec<JobResponse>> {
        let url = format!("{}/jobs", self.base_url);
        self.get(&url).await
    }

    pub async fn get_job(&self, job_id: &str) -> Result<JobResponse> {
        let url = format!("{}/jobs/{}", self.base_url, job_id);
        self.get(&url).await
    }

    pub async fn create_job(
        &self,
        sequence: String,
        shots: i32,
        session_id: &str,
    ) -> Result<JobResponse> {
        let url = format!("{}/jobs", self.base_url);
        let job = CreateJob { sequence, shots };

        let headers = self.create_headers().await?;
        let resp = self
            .client
            .post(url)
            .headers(headers)
            .header("X-Warden-Session", session_id)
            .json(&job)
            .send()
            .await?;

        self.handle_request(resp).await
    }

    pub async fn get_accessible(&self) -> Result<AccessibleResponse> {
        let url = format!("{}/accessible", self.base_url);

        let resp = self.client.get(url).send().await?;

        self.handle_request(resp).await
    }

    pub async fn create_session(
        &self,
        user_id: i32,
        slurm_job_id: &str,
    ) -> Result<SessionResponse> {
        let url = format!("{}/sessions", self.base_url);
        let session = CreateSessionPayload {
            user_id: user_id.to_string(),
            slurm_job_id: slurm_job_id.to_string(),
        };

        let headers = self.create_headers().await?;
        let resp = self
            .client
            .post(url)
            .headers(headers)
            .json(&session)
            .send()
            .await?;

        self.handle_request(resp).await
    }

    pub async fn revoke_session(&self, session_id: &str) -> Result<SessionResponse> {
        let url = format!("{}/sessions/{}", self.base_url, session_id);

        let headers = self.create_headers().await?;
        let resp = self.client.delete(url).headers(headers).send().await?;

        self.handle_request(resp).await
    }

    pub async fn get_device_specs(&mut self) -> Result<GetDeviceSpecsResponse> {
        let url = format!("{}/qpu/specs", self.base_url);
        let resp: GetDeviceSpecsResponse = self.get(&url).await?;
        Ok(resp)
    }

    pub async fn get_task_logs(&mut self, task_id: &str) -> Result<GetTaskLogsResponse> {
        let url = format!("{}/jobs/{}/logs", self.base_url, task_id);
        let resp: GetTaskLogsResponse = self.get(&url).await?;
        Ok(resp)
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let headers = self.create_headers().await?;
        let resp = self.client.get(url).headers(headers).send().await?;

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
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_local_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:4207");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        let base_url: String = base_url.into();
        Self { base_url }
    }

    /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_local_api::{ClientBuilder};
    ///
    /// let _builder = ClientBuilder::new("http://localhost:4207").build();
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);

        let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

        Ok(Client {
            base_url: self.base_url.clone(),
            client: reqwest_builder.build(),
        })
    }
}
