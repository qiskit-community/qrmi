//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::models::errors::ExtendedErrorResponse;
use crate::models::versions::ListAPIVersions;
use crate::Client;
use anyhow::{bail, Result};
use log::error;
#[cfg(feature = "skip_tls_cert_verify")]
use std::env;

impl Client {
    /// Returns the list of supported API versions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use direct_access_api::ClientBuilder;
    ///
    ///     let client = ClientBuilder::new("http://localhost:8080")
    ///         .build()
    ///         .unwrap();
    ///     let versions = client.list_api_versions().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    ///
    pub async fn list_api_versions(&self) -> Result<Vec<String>> {
        let url = format!("{}/versions", self.base_url,);
        #[allow(unused_mut)]
        let mut builder = reqwest::Client::builder().connection_verbose(true);
        #[cfg(feature = "skip_tls_cert_verify")]
        if let Ok(skip_cert_verify_envvar) = env::var("DANGER_TLS_SKIP_CERT_VERIFY") {
            if skip_cert_verify_envvar == "true" || skip_cert_verify_envvar == "1" {
                log::warn!("Insecure HTTPS request is being made. Disabling DANGER_TLS_SKIP_CERT_VERIFY is strongly advised for production.");
                builder = builder
                    .danger_accept_invalid_certs(true)
                    .danger_accept_invalid_hostnames(true);
            }
        }
        let http_client = builder.build()?;
        let resp_ = http_client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await;
        match resp_ {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    let json_data = resp.json::<ListAPIVersions>().await?;
                    Ok(json_data.versions.unwrap_or_default())
                } else {
                    match resp.json::<ExtendedErrorResponse>().await {
                        Ok(ExtendedErrorResponse::Json(error)) => {
                            let serialized = serde_json::to_value(&error).unwrap();
                            error!("{}", &serialized);
                            bail!(serialized);
                        }
                        Ok(ExtendedErrorResponse::Text(message)) => {
                            error!("{}", message);
                            bail!(format!("{} ({})", status, message));
                        }
                        Err(_) => {
                            error!("{} {}", status, url);
                            bail!(format!("{} {}", status, url));
                        }
                    }
                }
            }
            Err(e) => {
                error!("{:#?}", e);
                Err(e.into())
            }
        }
    }
}
