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

use crate::models::version::ServiceVersion;
use crate::Client;
use anyhow::Result;

impl Client {
    /// Returns the latest supported API version.
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
    ///     let version = client.get_service_version().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    ///
    pub async fn get_service_version(&self) -> Result<String> {
        let url = format!("{}/version", self.base_url,);
        let json_data = self.get::<ServiceVersion>(&url, &self.plain_client).await?;
        Ok(json_data.version)
    }
}
