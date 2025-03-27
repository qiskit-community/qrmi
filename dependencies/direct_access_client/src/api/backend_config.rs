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

use crate::Client;
use anyhow::Result;
use serde::de::DeserializeOwned;

impl Client {
    /// Returns the configuration for the specified backend.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use anyhow::Result;
    /// use direct_access_api::{AuthMethod, ClientBuilder};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = ClientBuilder::new("http://localhost:8080")
    ///         .with_auth(AuthMethod::IbmCloudIam {
    ///             apikey: "your_iam_apikey".to_string(),
    ///             service_crn: "your_service_crn".to_string(),
    ///             iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///         })
    ///         .build()
    ///         .unwrap();
    ///     let _config = client
    ///         .get_backend_configuration::<serde_json::Value>("ibm_brisbane")
    ///         .await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    /// - authentication failed.
    /// - specified backend is not found.
    pub async fn get_backend_configuration<T: DeserializeOwned>(
        &self,
        backend_name: &str,
    ) -> Result<T> {
        let url = format!(
            "{}/v1/backends/{}/configuration",
            self.base_url, backend_name
        );
        self.get::<T>(&url).await
    }
}
