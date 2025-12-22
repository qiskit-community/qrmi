//
// (C) Copyright IBM 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! IonQ Cloud API Client

use crate::models::backend::Backend;
//use crate::models::batch::BatchStatus;
use anyhow::{bail, Result};
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
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IonQBackend {
    pub backend: String,
    pub status: String,
    pub qubits: u32,
    pub degraded: Option<bool>,
    pub average_queue_time: Option<u64>,
    pub last_updated: Option<String>,
    pub noise_models: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionData {
    pub id: String,
    pub created_at: String,
    pub organization_id: String,
    pub backend: String,
    pub project_id: String,
    pub creator_id: String,

    pub ended_at: Option<String>,
    pub ender_id: Option<String>,

    pub active: bool,
    pub status: String,

    pub started_at: Option<String>,

    pub settings: SessionSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionSettings {
    pub job_count_limit: u32,
    pub duration_limit_min: u32,
    pub cost_limit: Option<SessionCostLimit>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionCostLimit {
    pub unit: String,
    pub value: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IonQJob {
    pub id: String,
    pub status: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRequestData {
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<SessionLimits>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionLimits {
    pub job_count_limit: u32,
    pub duration_limit_min: u32,
    pub cost_limit: SessionCostLimit,
}

impl Client {
    pub async fn get_backend(&self, backend: Backend) -> Result<IonQBackend> {
        let url = format!("{}/backends/{}", self.base_url, backend);
        let data: IonQBackend = self.get(&url).await?;
        Ok(data)
    }

    pub async fn create_session(
        &self,
        backend: Backend,
        session_request_data: &SessionRequestData,
    ) -> Result<SessionData> {
        let url = format!("{}/sessions", self.base_url);
        let data: SessionData = self.post(&url, session_request_data).await?;
        Ok(data)
    }

    pub async fn end_session(&self, id: &str) -> Result<SessionData> {
        let url = format!("{}/sessions/{}/end", self.base_url, id);
        let data: SessionData = self.post(&url, ()).await?;
        Ok(data)
    }

    pub async fn create_job(
        &self,
        backend: Backend,
        shots: i32,
        name: &str,
        metadata: &str,
        settings: &str,
        input: &str,
    ) -> Result<Response<IonQJob>> {
        todo!()
    }

    pub async fn create_jobs_batch(
        &self,
        backend: Backend,
        shots: i32,
        name: &str,
        metadata: &str,
        settings: &str,
        inputs: &[&str],
    ) -> Result<Response<IonQJob>> {
        todo!()
    }

    pub async fn get_job(&self, id: String) -> Result<Response<IonQJob>> {
        // new return struct needed
        todo!()
    }

    pub async fn cancel_job(&self, id: String) -> Result<Response<IonQJob>> {
        // new return struct needed
        todo!()
    }

    pub async fn delete_job(&self, id: String) -> Result<Response<IonQJob>> {
        // new return return struct needed
        todo!()
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.get(url).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        url: &str,
        body: U,
    ) -> Result<T> {
        let resp = self.client.post(url).json(&body).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn put<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.put(url).send().await?;
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
    api_key: String,
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`].
    ///
    /// # Example
    /// ```rust
    /// use ionq_cloud_api::ClientBuilder;
    ///
    /// let api_key = "your_api_key".to_string();
    /// let _builder = ClientBuilder::new(api_key);
    /// ```
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.ionq.co/v0.4".to_string(),
            api_key,
        }
    }

    /// Builds a [`Client`] using this builder's configuration.
    ///
    /// # Example
    /// ```rust
    /// use ionq_cloud_api::ClientBuilder;
    ///
    /// let api_key = "your_api_key".to_string();
    /// let _client = ClientBuilder::new(api_key).build().unwrap();
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Authorization: {}", self.api_key))
                .unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        reqwest_client_builder = reqwest_client_builder.default_headers(headers);
        let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

        Ok(Client {
            base_url: self.base_url.clone(),
            client: reqwest_builder.build(),
        })
    }
}
