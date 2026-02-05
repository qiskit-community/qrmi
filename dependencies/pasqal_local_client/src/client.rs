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
use crate::munge;


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
    pub user_id: String
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateJob {
    pub sequence: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionPayload {
    pub user_id: String,
}

impl Client {
    pub async fn get_jobs(
        &self,
    ) -> Result<Vec<JobResponse>> {
        let url = format!(
            "{}/jobs",
            self.base_url,
        );
        let resp: Vec<JobResponse> = self.get(&url).await?;
        Ok(resp)
    }

    pub async fn create_job(
        &self,
        sequence: String,
        session_id: &str
    ) -> Result<JobResponse> {
        let url = format!("{}/jobs", self.base_url);
        let job = CreateJob {
            sequence: sequence,
        };
        let resp = self.client.post(url).header("X-Warden-Session", session_id).json(&job).send().await?;
        self.handle_request(resp).await
    }

    pub async fn create_session(
        &self,
        user_id: i32,
    )-> Result<SessionResponse> {
        let url = format!(
            "{}/sessions",
            self.base_url,
        );
        let session = CreateSessionPayload {
            user_id: user_id.to_string(),
        };
        let resp = self.client.post(url).json(&session).send().await?;
        self.handle_request(resp).await
    }

    pub async fn revoke_session(
        &self,
        session_id: &str,
    )-> Result<SessionResponse> {
        let url = format!(
            "{}/sessions/{}",
            self.base_url,
            session_id,
        );
        let resp = self.client.delete(url).send().await?;
        self.handle_request(resp).await
    }


    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.get(url).send().await?;
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
        /// let _builder = ClientBuilder::new();
        /// ```
        pub fn new() -> Self {
            Self {
                base_url: "http://localhost:4207".to_string(),
            }
        }

        /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
        ///
        /// # Example
        ///
        /// ```rust
        /// use pasqal_local_api::{ClientBuilder};
        ///
        /// let _builder = ClientBuilder::new().build()
        /// ```
        pub fn build(&mut self) -> Result<Client> {
            let mut reqwest_client_builder = reqwest::Client::builder();
            reqwest_client_builder = reqwest_client_builder.connection_verbose(true);

            let mut headers = header::HeaderMap::new();
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            );
            // TODO: Cache token?
            let token = munge::encode(b"")?;

            headers.insert(
                reqwest::header::HeaderName::from_static("x-munge-cred"),
                reqwest::header::HeaderValue::from_str(&token).expect("invalid munge token"),
            );

            reqwest_client_builder = reqwest_client_builder.default_headers(headers);
            let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

            Ok(Client {
                base_url: self.base_url.clone(),
                client: reqwest_builder.build(),
            })
        }
    }
