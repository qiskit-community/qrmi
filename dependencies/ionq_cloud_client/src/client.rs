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
use reqwest::StatusCode;
//use crate::models::batch::BatchStatus;
use anyhow::{bail, Result};
use log::debug;
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IonQBackend {
    pub backend: String,
    pub status: String,
    pub qubits: u32,
    pub degraded: Option<bool>,
    pub average_queue_time: Option<u64>,
    pub last_updated: Option<String>,
    pub noise_models: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionSettings {
    // IonQ may omit these fields depending on backend / defaults (notably simulator).
    // Make them optional to avoid failing decode.
    pub job_count_limit: Option<u32>,
    pub duration_limit_min: Option<u32>,
    pub cost_limit: Option<SessionCostLimit>,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionCostLimit {
    pub unit: String,
    pub value: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IonQJob {
    pub id: String,
    pub status: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateJobRequest<'a> {
    #[serde(rename = "type")]
    job_type: &'a str,
    backend: String,
    shots: i32,
    name: &'a str,
    input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    settings: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRequestData {
    pub backend: String,
    // IonQ API calls this "settings" in v0.4 docs; keep our internal name
    // but serialize with the expected field name.
    #[serde(skip_serializing_if = "Option::is_none", rename = "settings")]
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
        job_type: &str,
        shots: i32,
        name: &str,
        session_id: Option<&str>,
        metadata: Option<Value>,
        settings: Option<Value>,
        input: Value,
    ) -> Result<IonQJob> {
        // POST /jobs
        let url = format!("{}/jobs", self.base_url);

        if shots <= 0 {
            bail!("shots must be > 0");
        }

        let req = CreateJobRequest {
            job_type,
            backend: backend.to_string(),
            shots,
            name,
            input,
            session_id,
            metadata,
            settings,
        };

        let raw: Value = self.post(&url, req).await?;
        extract_job(raw)
    }

    pub async fn create_jobs_batch(
        &self,
        backend: Backend,
        job_type: &str,
        shots: i32,
        name: &str,
        session_id: Option<&str>,
        metadata: Option<Value>,
        settings: Option<Value>,
        inputs: &[Value],
    ) -> Result<Vec<IonQJob>> {
        // IonQ may have batch facilities, but to keep QRMI reliable and simple,
        // do a client-side batch submission (N independent jobs).
        let mut out = Vec::with_capacity(inputs.len());
        for (i, input) in inputs.iter().enumerate() {
            let job_name = format!("{name}-{i}");
            let job = self
                .create_job(
                    backend.clone(),
                    job_type,
                    shots,
                    &job_name,
                    session_id,
                    metadata.clone(),
                    settings.clone(),
                    input.clone(),
                )
                .await?;
            out.push(job);
        }
        Ok(out)
    }

    pub async fn get_job(&self, id: String) -> Result<IonQJob> {
        // GET /jobs/{id}
        let url = format!("{}/jobs/{}", self.base_url, id);
        let raw: Value = self.get(&url).await?;
        extract_job(raw)
    }

    pub async fn cancel_job(&self, id: String) -> Result<IonQJob> {
        // PUT /jobs/{id}/cancel
        let url = format!("{}/jobs/{}/cancel", self.base_url, id);
        let raw: Value = self.put(&url).await?;
        extract_job(raw)
    }

    pub async fn delete_job(&self, id: String) -> Result<Value> {
        // DELETE /jobs/{id}
        let url = format!("{}/jobs/{}", self.base_url, id);
        let raw: Value = self.delete(&url).await?;
        Ok(raw)
    }

    pub async fn get_job_probabilities(&self, id: &str) -> Result<Value> {
        // Different IonQ deployments have used either:
        // - /jobs/{id}/results/probabilities
        // - /jobs/{id}/results
        // Try probabilities first, then fall back to results on 404.
        let url_probs = format!("{}/jobs/{}/results/probabilities", self.base_url, id);
        let resp = self.client.get(&url_probs).send().await?;
        if resp.status().is_success() {
            return self.handle_request(resp).await;
        }
        if resp.status() == StatusCode::NOT_FOUND {
            let url_results = format!("{}/jobs/{}/results", self.base_url, id);
            let resp2 = self.client.get(&url_results).send().await?;
            return self.handle_request(resp2).await;
        }

        let status = resp.status();
        let json_text = resp.text().await?;
        bail!("Status: {}, Fail {}", status, json_text);
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

    pub(crate) async fn delete<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.delete(url).send().await?;
        self.handle_request(resp).await
    }

    async fn handle_request<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        if resp.status().is_success() {
            let json_text = resp.text().await?;
            debug!("{}", json_text);

            let val: Value = serde_json::from_str(&json_text)?;

            // Try direct decode first.
            if let Ok(out) = serde_json::from_value::<T>(val.clone()) {
                return Ok(out);
            }

            // Then try the common { "data": ... } envelope.
            if let Ok(wrapper) = serde_json::from_value::<Response<T>>(val.clone()) {
                return Ok(wrapper.data);
            }

            bail!("Unexpected IonQ response shape: {val}");
        } else {
            let status = resp.status();
            let json_text = resp.text().await?;
            bail!("Status: {}, Fail {}", status, json_text);
        }
    }
}

fn extract_job(raw: Value) -> Result<IonQJob> {
    // Be tolerant to response envelopes:
    // - { "id": "...", "status": "...", ... }
    // - { "job": { ... } }
    // - { "data": { ... } }
    let job_val = if raw.get("data").is_some() {
        raw.get("data").cloned().unwrap_or(raw)
    } else if raw.get("job").is_some() {
        raw.get("job").cloned().unwrap_or(raw)
    } else {
        raw
    };

    if let Ok(job) = serde_json::from_value::<IonQJob>(job_val.clone()) {
        return Ok(job);
    }

    // Manual fallback if IonQ adds fields / changes envelope
    let id = job_val
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("IonQ job response missing 'id': {job_val}"))?
        .to_string();
    let status = job_val
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let session_id = job_val
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(IonQJob {
        id,
        status,
        session_id,
    })
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
        let mut reqwest_client_builder =
            reqwest::Client::builder().connection_verbose(log::log_enabled!(log::Level::Trace));

        let mut headers = header::HeaderMap::new();

        // IonQ expects:  Authorization: apiKey <KEY>
        // i.e., header value is "apiKey <KEY>"
        let auth_val = header::HeaderValue::from_str(&format!("apiKey {}", self.api_key))?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_val);
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
