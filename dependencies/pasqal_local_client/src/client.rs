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

use crate::munge;
use anyhow::{bail, Result};

use crate::models::job::JobStatus;
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
pub struct QpuSlotsResponse {
    pub qpu_slots_total: u64,
    pub qpu_slots_used: u64,
    pub qpu_slots_available: u64,
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

#[derive(Debug, Clone, Deserialize)]
pub struct Specs {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceSpecs {
    pub device_type: String,
    pub specs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionPayload {
    pub user_id: String,
    pub scheduler_job_id: String,
    pub qpu_slots: i32,
}

impl Client {
    /// Return authentication headers for request with a fresh munge token
    async fn create_headers(&self) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let token = munge::encode(b"")?;
        headers.insert(
            reqwest::header::HeaderName::from_static("x-munge-cred"),
            reqwest::header::HeaderValue::from_str(&token).expect("invalid munge token"),
        );
        Ok(headers)
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

    pub async fn cancel_job(&self, job_id: &str) -> Result<JobResponse> {
        let url = format!("{}/jobs/{}/cancel", self.base_url, job_id);
        let headers = self.create_headers().await?;
        let resp = self.client.post(url).headers(headers).send().await?;

        self.handle_request(resp).await
    }

    pub async fn get_accessible(&self) -> Result<AccessibleResponse> {
        let url = format!("{}/accessible", self.base_url);

        let resp = self.client.get(url).send().await?;

        self.handle_request(resp).await
    }

    /// Returns Warden's QPU slot usage, or `None` if Warden does not
    /// manage QPU slots.
    pub async fn get_qpu_slots(&self) -> Result<Option<QpuSlotsResponse>> {
        let url = format!("{}/qpu-slots", self.base_url);

        let resp = self.client.get(url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        self.handle_request(resp).await.map(Some)
    }

    pub async fn create_session(
        &self,
        user_id: i32,
        scheduler_job_id: &str,
        qpu_slots: i32,
    ) -> Result<SessionResponse> {
        let url = format!("{}/sessions", self.base_url);
        let session = CreateSessionPayload {
            user_id: user_id.to_string(),
            scheduler_job_id: scheduler_job_id.to_string(),
            qpu_slots,
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
        let url = format!("{}/sessions", self.base_url);

        let headers = self.create_headers().await?;
        let resp = self
            .client
            .delete(url)
            .headers(headers)
            .header("X-Warden-Session", session_id)
            .send()
            .await?;

        self.handle_request(resp).await
    }

    pub async fn get_device_specs(&mut self) -> Result<String> {
        let url = format!("{}/qpu/specs", self.base_url);
        let resp: GetDeviceSpecsResponse = self.get(&url).await?;
        let specs: String = resp.specs.clone();
        // Parse into a dynamic JSON value instead of a custom Struct
        let parsed_specs: Specs = serde_json::from_str(&resp.specs)?;

        let device_specs = DeviceSpecs {
            device_type: parsed_specs.name,
            specs,
        };
        // Matching the cloud client's serialized data return type
        Ok(serde_json::to_string(&vec![device_specs])?)
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
            let val = serde_json::from_str(&json_text)?;
            Ok(val)
        } else {
            let status = resp.status();
            let json_text = resp.text().await?;
            bail!("Status: {}, Fail {}", status, redact_error_body(&json_text));
        }
    }
}

/// Drops request values that Warden echoes in validation errors.
///
/// FastAPI 422 responses repeat the rejected `input`, which may be a
/// session credential taken from a request header.
fn redact_error_body(body: &str) -> String {
    let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body) else {
        return body.to_string();
    };
    if let Some(details) = value.get_mut("detail").and_then(|d| d.as_array_mut()) {
        for detail in details.iter_mut().filter_map(|d| d.as_object_mut()) {
            detail.remove("input");
            detail.remove("ctx");
        }
    }
    value.to_string()
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
        // No connection_verbose: it logs raw requests, including the
        // X-Warden-Session and X-Munge-Cred credentials.
        let reqwest_builder = ReqwestClientBuilder::new(reqwest::Client::builder().build()?);

        Ok(Client {
            base_url: self.base_url.clone(),
            client: reqwest_builder.build(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ClientBuilder;

    #[tokio::test]
    async fn get_qpu_slots_reads_slot_usage() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/qpu-slots")
            .with_status(200)
            .with_body(r#"{"qpu_slots_total":10,"qpu_slots_used":4,"qpu_slots_available":6}"#)
            .create_async()
            .await;
        let client = ClientBuilder::new(server.url()).build().unwrap();

        let slots = client.get_qpu_slots().await.unwrap().unwrap();

        mock.assert_async().await;
        assert_eq!(slots.qpu_slots_total, 10);
        assert_eq!(slots.qpu_slots_used, 4);
        assert_eq!(slots.qpu_slots_available, 6);
    }

    #[tokio::test]
    async fn get_qpu_slots_is_none_when_slots_are_not_configured() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/qpu-slots")
            .with_status(404)
            .with_body(r#"{"detail":"QPU slots are not configured."}"#)
            .create_async()
            .await;
        let client = ClientBuilder::new(server.url()).build().unwrap();

        assert!(client.get_qpu_slots().await.unwrap().is_none());
    }

    #[test]
    fn error_bodies_do_not_repeat_request_input() {
        let body = r#"{"detail":[{"type":"uuid_parsing","loc":["header","X-Warden-Session"],"msg":"Input should be a valid UUID","input":"secret-session"}]}"#;
        let redacted = super::redact_error_body(body);
        assert!(!redacted.contains("secret-session"));
        assert!(redacted.contains("Input should be a valid UUID"));
        assert_eq!(super::redact_error_body("not json"), "not json");
    }

    #[tokio::test]
    async fn get_qpu_slots_reports_server_errors() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/qpu-slots")
            .with_status(500)
            .create_async()
            .await;
        let client = ClientBuilder::new(server.url()).build().unwrap();

        assert!(client.get_qpu_slots().await.is_err());
    }
}
