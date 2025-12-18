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

use crate::models::device::DeviceType;
use anyhow::{bail, Result};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

/// An asynchronous `Client` to make Requests with.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
}

impl Client {
    pub async fn get_device(
        &self,
        device_type: DeviceType,
    ) -> Result<GetDeviceResponseData> {
        let url = format!(
            "{}/backends/{}",
            self.base_url, device_type,
        );
        let resp: Response<Vec<GetDeviceResponseData>> = self.get(&url).await?;

        resp.data
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No devices found for type {:?}", device_type))
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        // let resp = self.client.get(url).send().await?;
        // self.handle_request(resp).await
        todo!()
    }
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

/// A [`ClientBuilder`] can be used to create a [`Client`] with custom configuration.
#[must_use]
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// The base URL this client sends requests to
    base_url: String,
    api_key: String,
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use ionq_cloud_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new(api_key);
    /// ```
    pub fn new(api_key: String) -> Self {
        Self {
            base_url: "https://api.ionq.co/v0.4".to_string(),
            api_key,
        }
    }

    /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ionq_cloud_api::{ClientBuilder, AuthMethod};
    ///
    /// let _builder = ClientBuilder::new()
    ///     .with_api_key("your_api_key".to_string())
    ///     .build()
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Authorization: {}", self.api_key)).unwrap(),
        );
        // reqwest_client_builder = reqwest_client_builder.default_headers(headers);
        // let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

        Ok(Client {
            base_url: self.base_url.clone(),
            //client: reqwest_builder.build()
        })

        // This is work in progress!
    }
}
