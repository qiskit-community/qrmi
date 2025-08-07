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

//! Direct Access API Client

use anyhow::{bail, Result};
#[cfg(feature = "api_version")]
use chrono::Utc;
use retry_policies::policies::ExponentialBackoff;
use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info};
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use reqwest_retry::RetryTransientMiddleware;
use serde::de::DeserializeOwned;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::middleware::auth::{AuthMiddleware, TokenManager};
use crate::models::errors::ErrorResponse;

/// Authorization method and credentials.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// IBM Cloud IAM Bearer Token based authentication
    IbmCloudIam {
        /// API key to access IAM POST /identity/token API
        apikey: String,
        /// Service CRN ("crn:version:cname:ctype:service-name:location:scope:service-instance:resource-type:resource")
        service_crn: String,
        /// IAM endpoint (e.g. <https://iam.cloud.ibm.com>)
        iam_endpoint_url: String,
    },
    /// Deprecated. IBM Cloud App ID access token based authentication
    #[cfg(feature = "ibmcloud_appid_auth")]
    IbmCloudAppId {
        /// App ID username
        username: String,
        /// App ID password
        password: String,
    },
    /// Deprecated. Internal shared key based authentication
    #[cfg(feature = "internal_shared_key_auth")]
    InternalSharedKey {
        /// Client ID
        client_id: String,
        /// Shared Token
        shared_token: String,
    },
}

/// An asynchronous `Client` to make Requests with.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
    /// HTTP client to interact with Direct Access API service
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
    /// The configuration to create [`S3Client`](aws_sdk_s3::Client)
    pub(crate) s3_config: Option<aws_sdk_s3::Config>,
    /// The name of S3 bucket
    pub(crate) s3_bucket: Option<String>,
}

impl Client {
    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self
            .client
            .get(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        if resp.status().is_success() {
            let json_text = resp.text().await?;
            debug!("{}", json_text);
            Ok(serde_json::from_str::<T>(&json_text)?)
        } else {
            let error_resp = resp.json::<ErrorResponse>().await?;
            bail!(format!(
                "{} ({}) {:?}",
                error_resp.title, error_resp.status_code, error_resp.errors
            ));
        }
    }
}

/// A [`ClientBuilder`] can be used to create a [`Client`] with custom configuration.
#[must_use]
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// The base URL this client sends requests to
    base_url: String,
    /// `IBM-API-Version` HTTP header value
    #[cfg(feature = "api_version")]
    api_version: String,
    /// The authentication method & credentials
    auth_method: AuthMethod,
    /// The timeout
    timeout: Option<Duration>,
    /// The connection timeout
    connect_timeout: Option<Duration>,
    /// The read timeout
    read_timeout: Option<Duration>,
    /// The retry policy
    retry_policy: Option<ExponentialBackoff>,
    /// The configuration to create [`S3Client`](aws_sdk_s3::Client)
    s3_config: Option<aws_sdk_s3::Config>,
    /// The name of S3 Bucket used by this [`Client`]
    s3_bucket: Option<String>,
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`] with the specified URL where
    /// Direct Access API service is running.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8080");
    /// ```
    pub fn new(url: impl Into<String>) -> Self {
        let url: String = url.into();
        Self {
            base_url: url,
            #[cfg(feature = "api_version")]
            api_version: Utc::now().format("%Y-%m-%d").to_string(),
            auth_method: AuthMethod::None,
            timeout: None,
            connect_timeout: None,
            read_timeout: None,
            retry_policy: None,
            s3_config: None,
            s3_bucket: None,
        }
    }

    /// Add authentication information to [`ClientBuilder`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::{ClientBuilder, AuthMethod};
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_auth(
    ///          AuthMethod::IbmCloudIam {
    ///              apikey: "your_iam_apikey".to_string(),
    ///              service_crn: "your_service_crn".to_string(),
    ///              iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///          }
    ///     );
    /// ```
    pub fn with_auth(&mut self, auth_method: AuthMethod) -> &mut Self {
        self.auth_method = auth_method;
        self
    }

    /// Enables a total request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished. Also considered a total deadline.
    ///
    /// Default is no timeout.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_timeout(Duration::from_secs(60));
    /// ```
    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set a timeout for only the connect phase of a `Client`.
    ///
    /// Default is `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_connect_timeout(Duration::from_secs(5));
    /// ```
    pub fn with_connect_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Enables a read timeout.
    ///
    /// The timeout applies to each read operation, and resets after a
    /// successful read. This is more appropriate for detecting stalled
    /// connections when the size isn't known beforehand.
    ///
    /// Default is no timeout.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_read_timeout(Duration::from_secs(30));
    /// ```
    pub fn with_read_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.read_timeout = Some(timeout);
        self
    }

    pub fn with_retry_policy(&mut self, policy: ExponentialBackoff) -> &mut Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Set the `IBM-API-Version` header to be used by this client.
    ///
    /// Default is the current datetime in %Y-%m-%d format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_api_version("2024-01-01");
    /// ```
    #[cfg(feature = "api_version")]
    pub fn with_api_version(&mut self, api_version: impl Into<String>) -> &mut Self {
        let api_version: String = api_version.into();
        self.api_version = api_version;
        self
    }

    /// Set the S3 bucket connection parameters for this client.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_s3bucket(
    ///         "my_aws_access_key_id",
    ///         "my_aws_secret_access_key",
    ///         "http://localhost:9000",
    ///         "my_bucket",
    ///         "us-east-1");
    /// ```
    ///
    pub fn with_s3bucket(
        &mut self,
        aws_access_key_id: impl Into<String>,
        aws_secret_access_key: impl Into<String>,
        s3_endpoint_url: impl Into<String>,
        s3_bucket: impl Into<String>,
        s3_region: impl Into<String>,
    ) -> &mut Self {
        let cred = aws_credential_types::Credentials::new(
            aws_access_key_id.into(),
            aws_secret_access_key.into(),
            None,
            None,
            "direct_access_client",
        );
        let s3_config = aws_sdk_s3::config::Builder::new()
            .endpoint_url(s3_endpoint_url.into())
            .credentials_provider(cred)
            .region(aws_sdk_s3::config::Region::new(s3_region.into()))
            .force_path_style(true)
            .build();
        self.s3_config = Some(s3_config);
        self.s3_bucket = Some(s3_bucket.into());
        self
    }

    /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use direct_access_api::{ClientBuilder, AuthMethod};
    ///
    /// let _client = ClientBuilder::new("http://localhost:8280")
    ///     .with_auth(
    ///          AuthMethod::IbmCloudIam {
    ///              apikey: "your_iam_apikey".to_string(),
    ///              service_crn: "your_service_crn".to_string(),
    ///              iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///          }
    ///     )
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);
        if let Some(v) = self.timeout {
            reqwest_client_builder = reqwest_client_builder.timeout(v)
        }

        if let Some(v) = self.read_timeout {
            reqwest_client_builder = reqwest_client_builder.read_timeout(v)
        }

        if let Some(v) = self.connect_timeout {
            reqwest_client_builder = reqwest_client_builder.connect_timeout(v)
        }

        #[allow(unused_mut)]
        let mut headers = header::HeaderMap::new();
        #[cfg(feature = "api_version")]
        {
            let api_ver_value = header::HeaderValue::from_str(self.api_version.as_str())?;
            headers.insert("IBM-API-Version", api_ver_value);
        }
        #[cfg(feature = "internal_shared_key_auth")]
        if let AuthMethod::InternalSharedKey {
            client_id,
            shared_token,
        } = self.auth_method.clone()
        {
            let auth_str = format!("apikey {}:{}", client_id, shared_token);
            let mut auth_value = header::HeaderValue::from_str(auth_str.as_str())?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        if let AuthMethod::IbmCloudIam { service_crn, .. } = self.auth_method.clone() {
            let service_crn_value = header::HeaderValue::from_str(&service_crn)?;
            headers.insert("Service-CRN", service_crn_value);
        }

        reqwest_client_builder = reqwest_client_builder.default_headers(headers);
        let mut reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

        if let Some(v) = self.retry_policy {
            reqwest_builder = reqwest_builder.with(RetryTransientMiddleware::new_with_policy(v))
        }

        #[cfg(feature = "ibmcloud_appid_auth")]
        if let AuthMethod::IbmCloudAppId { .. } = self.auth_method.clone() {
            let token_url = format!("{}/v1/token", self.base_url);
            let token_manager = Arc::new(Mutex::new(TokenManager::new(
                token_url,
                self.auth_method.clone(),
            )));

            let auth_middleware = AuthMiddleware::new(token_manager.clone());
            reqwest_builder = reqwest_builder.with(auth_middleware);
        }
        if let AuthMethod::IbmCloudIam {
            iam_endpoint_url, ..
        } = self.auth_method.clone()
        {
            let token_url = format!("{}/identity/token", iam_endpoint_url);
            let token_manager = Arc::new(Mutex::new(TokenManager::new(
                token_url,
                self.auth_method.clone(),
            )));

            let auth_middleware = AuthMiddleware::new(token_manager.clone());
            reqwest_builder = reqwest_builder.with(auth_middleware);
        }

        let client = reqwest_builder.build();

        Ok(Client {
            base_url: self.base_url.clone(),
            client,
            s3_config: self.s3_config.clone(),
            s3_bucket: self.s3_bucket.clone(),
        })
    }
}

/// An asynchronous client to interact with running primitive jobs.
#[derive(Debug, Clone)]
pub struct PrimitiveJob {
    /// Job identifier
    pub job_id: String,
    pub(crate) client: Client,
    /// S3 client to work with S3 bucket
    pub(crate) s3_client: aws_sdk_s3::Client,
    /// The name of S3 bucket
    pub(crate) s3_bucket: String,
}
