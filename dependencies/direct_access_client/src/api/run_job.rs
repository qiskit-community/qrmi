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
use anyhow::{bail, Result};
use http::StatusCode;

impl Client {
    /// Run a job. Refer Direct Access API specifications for more details of the payload format.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use serde_json::json;
    ///     use direct_access_api::{AuthMethod, ClientBuilder};
    ///
    ///     let payload = json!({
    ///         "id": "bb2861da-d2c9-4de0-8f0b-4e399c4b02ac",
    ///         "backend": "ibm_brisbane",
    ///         "program_id": "estimator",
    ///         "log_level": "info",
    ///         "timeout_secs": 10000,
    ///         "storage": {
    ///             "input": {
    ///                 "presigned_url": "http://localhost:9000/test/params_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=aB7R0W5XLlo0iwd3yUCY6F2XvVg%3D&Expires=1730043158",
    ///                 "type": "s3_compatible"
    ///             },
    ///             "results": {
    ///                 "presigned_url": "http://localhost:9000/test/results_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=MABxhJ2gV6RvWin6llS64jZwY2M%3D&Expires=1730043158",
    ///                 "type": "s3_compatible"
    ///             },
    ///             "logs": {
    ///                 "presigned_url": "http://localhost:9000/test/logs_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=bH3QxADk5r2ojcaXqs46VKfjM1s%3D&Expires=1730043158",
    ///                 "type": "s3_compatible"
    ///             }
    ///         }
    ///     });
    ///
    ///     let client = ClientBuilder::new("http://localhost:8290")
    ///         .with_auth(AuthMethod::IbmCloudIam {
    ///             apikey: "your_iam_apikey".to_string(),
    ///             service_crn: "your_service_crn".to_string(),
    ///             iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///         })
    ///         .build()
    ///         .unwrap();
    ///     let _job_id = client.run_job(&payload).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    /// - invalid input is received.
    /// - authentication failed.
    /// - the request took too long to read.
    /// - a duplicate job with same job ID was submitted.
    /// - the request body is too large.
    /// - validation of the request failed. The error message contains details about the specific validation error.
    /// - backend is reserved and jobs outside of the reservation cannot be run.
    /// - per backend concurrent job limit has been reached.
    pub async fn run_job(&self, payload: &serde_json::Value) -> Result<String> {
        let url = format!("{}/v1/jobs", self.base_url);
        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await?;
        let status_code = resp.status();
        if status_code == StatusCode::NO_CONTENT {
            return Ok(payload["id"].as_str().unwrap().to_string());
        }
        let json_data = resp.json::<serde_json::Value>().await?;
        bail!(json_data.to_string())
    }
}
