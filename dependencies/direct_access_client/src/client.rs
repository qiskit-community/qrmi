//
// (C) Copyright IBM 2024-2026
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
use std::time::Duration;

#[allow(unused_imports)]
use log::{debug, error, info, warn};
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
#[cfg(feature = "iqp_retry_policy")]
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, Jitter, Retryable, RetryableStrategy,
};
use serde::de::DeserializeOwned;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use std::error::Error;

#[cfg(feature = "skip_tls_cert_verify")]
use std::env;

use crate::middleware::auth::{AuthMiddleware, TokenManager};
use crate::models::errors::ExtendedErrorResponse;

#[cfg(feature = "iqp_retry_policy")]
struct RetryStrategyExcept429;
#[cfg(feature = "iqp_retry_policy")]
impl RetryableStrategy for RetryStrategyExcept429 {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(success) if success.status() == 429 || success.status() == 423 => None,
            Ok(success) => default_on_request_success(success),
            Err(error) => default_on_request_failure(error),
        }
    }
}

#[cfg(feature = "iqp_retry_policy")]
struct RetryStrategy429;
#[cfg(feature = "iqp_retry_policy")]
impl RetryableStrategy for RetryStrategy429 {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(success) if success.status() == 429 || success.status() == 423 => {
                Some(Retryable::Transient)
            }
            Ok(_success) => None,
            // but maybe retry a request failure
            Err(error) => default_on_request_failure(error),
        }
    }
}

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
    /// HTTP plain client to interact with services (no auth, no headers)
    pub(crate) plain_client: reqwest_middleware::ClientWithMiddleware,
    /// The configuration to create [`S3Client`](aws_sdk_s3::Client)
    pub(crate) s3_config: Option<aws_sdk_s3::Config>,
    /// The name of S3 bucket
    pub(crate) s3_bucket: Option<String>,
    /// The configuration to create [`S3Client`](aws_sdk_s3::Client) for accessing from DA API service. Depending on the network configuration, the IP address used to access S3 may differ between access from the API client and access from the DA API service. In such cases, this property should specify the URL used when accessing S3 from the DA API service.
    pub(crate) s3_config_for_daapi: Option<aws_sdk_s3::Config>,
}

impl Client {
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        client: &reqwest_middleware::ClientWithMiddleware,
    ) -> Result<T> {
        let resp_ = client.get(url).send().await;
        match resp_ {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    let json_text = resp.text().await?;
                    debug!("{}", json_text);
                    Ok(serde_json::from_str::<T>(&json_text)?)
                } else {
                    match resp.json::<ExtendedErrorResponse>().await {
                        Ok(ExtendedErrorResponse::Json(error)) => {
                            let serialized = serde_json::to_value(&error).unwrap();
                            error!("{}", &serialized);
                            bail!(format!(
                                "An error occurred while fetching data from API. {}",
                                &serialized
                            ));
                        }
                        Ok(ExtendedErrorResponse::Text(message)) => {
                            error!("{}", message);
                            bail!(format!(
                                "An error occurred while fetching data from API. {} ({})",
                                status, message
                            ));
                        }
                        Err(_) => {
                            error!("{} {}", status, url);
                            bail!(format!(
                                "An error occurred while fetching data from API. {} {}",
                                status, url
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                let err_msg = Client::explain_reqwest_middleware_error(&e);
                bail!(format!(
                    "An error occurred while fetching data from API. {}",
                    err_msg
                ));
            }
        }
    }

    pub fn explain_reqwest_middleware_error(e: &reqwest_middleware::Error) -> String {
        let mut v = Vec::new();
        let mut i = 0;
        match e {
            reqwest_middleware::Error::Middleware(middleware_err) => {
                let mut src = middleware_err.source();
                while let Some(err) = src {
                    v.push(format!("reasons[{}]: {}", i, err));
                    src = err.source();
                    i += 1;
                }
            }
            reqwest_middleware::Error::Reqwest(req_err) => {
                let mut src = req_err.source();
                while let Some(err) = src {
                    v.push(format!("reasons[{}]: {}", i, err));
                    src = err.source();
                    i += 1;
                }
            }
        }
        if v.is_empty() {
            e.to_string()
        } else {
            v.join(", ")
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
    api_version: Option<String>,
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
    /// The configuration to create [`S3Client`](aws_sdk_s3::Client). Depending on the network configuration, the IP address used to access S3 may differ between access from the API client and access from the DA API service. In such cases, this property should specify the URL used when accessing S3 from the DA API service.
    s3_config_for_daapi: Option<aws_sdk_s3::Config>,
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
            api_version: None,
            auth_method: AuthMethod::None,
            timeout: None,
            connect_timeout: None,
            read_timeout: None,
            retry_policy: None,
            s3_config: None,
            s3_bucket: None,
            s3_config_for_daapi: None,
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

    /// Specify a retry policy of REST API calls.
    ///
    /// Default is no retry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use reqwest_retry::policies::ExponentialBackoff;
    /// use reqwest_retry::Jitter;
    /// use direct_access_api::ClientBuilder;
    ///
    /// let retry_policy = ExponentialBackoff::builder()
    ///     .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
    ///     .jitter(Jitter::Bounded)
    ///     .base(2)
    ///     .build_with_max_retries(5);
    /// let _builder = ClientBuilder::new("http://localhost:8280")
    ///     .with_retry_policy(retry_policy);
    /// ```
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
        self.api_version = Some(api_version);
        self
    }

    /// Set the S3 bucket connection parameters for this client.
    ///
    /// Depending on the network configuration, the IP address used to access S3 may differ between access from the API client and access from the DA API service. In such cases, `s3_endpoint_url_for_daapi` argument should specify the URL used when accessing S3 from the DA API service.
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
    ///         "us-east-1",
    ///         None::<String>);
    /// ```
    ///
    pub fn with_s3bucket(
        &mut self,
        aws_access_key_id: impl Into<String>,
        aws_secret_access_key: impl Into<String>,
        s3_endpoint_url: impl Into<String>,
        s3_bucket: impl Into<String>,
        s3_region: impl Into<String>,
        s3_endpoint_url_for_daapi: Option<impl Into<String>>,
    ) -> &mut Self {
        let cred = aws_credential_types::Credentials::new(
            aws_access_key_id.into(),
            aws_secret_access_key.into(),
            None,
            None,
            "direct_access_client",
        );

        let config_builder = aws_sdk_s3::config::Builder::new()
            .credentials_provider(cred.clone())
            .region(aws_sdk_s3::config::Region::new(s3_region.into()))
            .force_path_style(true);
        let s3_config = config_builder
            .clone()
            .endpoint_url(s3_endpoint_url.into())
            .build();
        self.s3_config = Some(s3_config);

        if let Some(endpoint_for_daapi) = s3_endpoint_url_for_daapi {
            let s3_config_for_daapi = config_builder
                .clone()
                .endpoint_url(endpoint_for_daapi.into())
                .build();
            self.s3_config_for_daapi = Some(s3_config_for_daapi);
        }

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
        let mut reqwest_plain_client_builder = reqwest::Client::builder();

        #[cfg(feature = "skip_tls_cert_verify")]
        if let Ok(skip_cert_verify_envvar) = env::var("DANGER_TLS_SKIP_CERT_VERIFY") {
            if skip_cert_verify_envvar == "true" || skip_cert_verify_envvar == "1" {
                warn!("Insecure HTTPS request is being made. Disabling DANGER_TLS_SKIP_CERT_VERIFY is strongly advised for production.");
                reqwest_client_builder = reqwest_client_builder
                    .danger_accept_invalid_certs(true)
                    .danger_accept_invalid_hostnames(true);
                reqwest_plain_client_builder = reqwest_plain_client_builder
                    .danger_accept_invalid_certs(true)
                    .danger_accept_invalid_hostnames(true);
            }
        }

        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);
        reqwest_plain_client_builder = reqwest_plain_client_builder.connection_verbose(true);
        if let Some(v) = self.timeout {
            reqwest_client_builder = reqwest_client_builder.timeout(v);
            reqwest_plain_client_builder = reqwest_plain_client_builder.timeout(v)
        }

        if let Some(v) = self.read_timeout {
            reqwest_client_builder = reqwest_client_builder.read_timeout(v);
            reqwest_plain_client_builder = reqwest_plain_client_builder.read_timeout(v)
        }

        if let Some(v) = self.connect_timeout {
            reqwest_client_builder = reqwest_client_builder.connect_timeout(v);
            reqwest_plain_client_builder = reqwest_plain_client_builder.connect_timeout(v)
        }

        #[allow(unused_mut)]
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_str("application/json")?,
        );
        #[cfg(feature = "api_version")]
        if let Some(api_ver_value) = &self.api_version {
            let api_ver_value = header::HeaderValue::from_str(api_ver_value.as_str())?;
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
        let mut middleware_client_builder =
            ReqwestClientBuilder::new(reqwest_client_builder.build()?);
        let mut middleware_plain_client_builder =
            ReqwestClientBuilder::new(reqwest_plain_client_builder.build()?);

        if let Some(v) = self.retry_policy {
            #[cfg(feature = "iqp_retry_policy")]
            {
                let mut retry_policy_429 = ExponentialBackoff::builder()
                    .retry_bounds(Duration::from_secs(1), Duration::from_secs(60))
                    .jitter(Jitter::Bounded)
                    .base(2)
                    .build_with_max_retries(1);
                retry_policy_429.max_n_retries = None;
                middleware_client_builder = middleware_client_builder
                    .with(RetryTransientMiddleware::new_with_policy_and_strategy(
                        v,
                        RetryStrategyExcept429,
                    ))
                    .with(RetryTransientMiddleware::new_with_policy_and_strategy(
                        retry_policy_429,
                        RetryStrategy429,
                    ))
            }
            #[cfg(not(feature = "iqp_retry_policy"))]
            {
                middleware_client_builder =
                    middleware_client_builder.with(RetryTransientMiddleware::new_with_policy(v));
            }
            middleware_plain_client_builder =
                middleware_plain_client_builder.with(RetryTransientMiddleware::new_with_policy(v));
        };

        #[cfg(feature = "ibmcloud_appid_auth")]
        if let AuthMethod::IbmCloudAppId { .. } = self.auth_method.clone() {
            let token_url = format!("{}/v1/token", self.base_url);
            let token_manager = Arc::new(Mutex::new(TokenManager::new(
                token_url,
                self.auth_method.clone(),
                self.timeout,
                self.connect_timeout,
                self.read_timeout,
                self.retry_policy,
            )?));

            let auth_middleware = AuthMiddleware::new(token_manager.clone());
            middleware_client_builder = middleware_client_builder.with(auth_middleware);
        }
        if let AuthMethod::IbmCloudIam {
            iam_endpoint_url, ..
        } = self.auth_method.clone()
        {
            let token_url = format!("{}/identity/token", iam_endpoint_url);
            let token_manager = Arc::new(Mutex::new(TokenManager::new(
                token_url,
                self.auth_method.clone(),
                self.timeout,
                self.connect_timeout,
                self.read_timeout,
                self.retry_policy,
            )?));

            let auth_middleware = AuthMiddleware::new(token_manager.clone());
            middleware_client_builder = middleware_client_builder.with(auth_middleware);
        }

        let client = middleware_client_builder.build();
        let plain_client = middleware_plain_client_builder.build();

        Ok(Client {
            base_url: self.base_url.clone(),
            client,
            plain_client,
            s3_config: self.s3_config.clone(),
            s3_bucket: self.s3_bucket.clone(),
            s3_config_for_daapi: self.s3_config_for_daapi.clone(),
        })
    }
}

/// An asynchronous client to interact with running primitive jobs.
#[derive(Debug, Clone)]
pub struct PrimitiveJob {
    /// Job identifier. Recommended to be UUID.
    pub job_id: String,
    /// HTTP client
    pub(crate) client: Client,
    /// S3 client to work with S3 bucket
    pub(crate) s3_client: aws_sdk_s3::Client,
    /// The name of S3 bucket
    pub(crate) s3_bucket: String,
}
