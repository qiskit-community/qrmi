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

use crate::models::errors::ExtendedErrorResponse;
use crate::{Client, PrimitiveJob};
use anyhow::{bail, Result};
use http::StatusCode;
use log::error;

impl Client {
    /// Delete the specified job if it has terminated.
    ///
    /// # Example
    ///
    /// ```no_run
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
    ///     client.delete_job("db4afb4a-2232-4b15-b750-3a327f05fc28").await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    /// - authentication failed.
    /// - specified job is not found.
    /// - a job has not yet terminated and has to be cancelled before it can be deleted.
    ///
    pub async fn delete_job(&self, job_id: &str) -> Result<()> {
        let url = format!("{}/v1/jobs/{}", self.base_url, &job_id);
        let resp_ = self.client.delete(&url).send().await;

        match resp_ {
            Ok(resp) => {
                let status_code = resp.status();
                if status_code == StatusCode::NO_CONTENT {
                    Ok(())
                } else {
                    match resp.json::<ExtendedErrorResponse>().await {
                        Ok(ExtendedErrorResponse::Json(error)) => {
                            let serialized = serde_json::to_value(&error).unwrap();
                            error!("{}", &serialized);
                            bail!(format!(
                                "An error occurred while sending the request to delete the job to the API. {}",
                                serialized
                            ));
                        }
                        Ok(ExtendedErrorResponse::Text(message)) => {
                            error!("{}", message);
                            bail!(format!(
                                "An error occurred while sending the request to delete the job to the API. {} ({})",
                                status_code, message
                            ));
                        }
                        Err(_) => {
                            error!("{} {}", status_code, url);
                            bail!(format!(
                                "An error occurred while sending the request to delete the job to the API. {} {}",
                                status_code, url
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                let err_msg = Client::explain_reqwest_middleware_error(&e);
                bail!(format!(
                    "An error occurred while sending the request to delete the job to the API. {}",
                    err_msg
                ));
            }
        }
    }
}

impl PrimitiveJob {
    /// Delete the specified job if it has terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde_json::json;
    /// use direct_access_api::{AuthMethod, ClientBuilder, models::ProgramId, models::LogLevel};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let payload = json!({
    ///         "pubs":[
    ///             [
    ///                 "OPENQASM 3.0; include \\\"stdgates.inc\\\"; bit[2] meas; rz(pi/2) $0; sx $0; rz(pi/2) $0; cx $0, $1; meas[0] = measure $0; meas[1] = measure $1;",[],128
    ///             ],
    ///         ],
    ///         "supports_qiskit": false,
    ///         "version":2,
    ///     });
    ///
    ///     let client = ClientBuilder::new("http://localhost:8290")
    ///         .with_auth(AuthMethod::IbmCloudIam {
    ///             apikey: "your_iam_apikey".to_string(),
    ///             service_crn: "your_service_crn".to_string(),
    ///             iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///         })
    ///         .with_s3bucket(
    ///             "my_aws_access_key_id",
    ///             "my_aws_secret_access_key",
    ///             "http://localhost:9000",
    ///             "my_bucket",
    ///             "us-east-1",
    ///             None::<String>,
    ///         )
    ///         .build()
    ///         .unwrap();
    ///
    ///     let primitive_job = client
    ///         .run_primitive("ibm_brisbane", ProgramId::Sampler, 3600, LogLevel::Info, &payload, None)
    ///         .await?;
    ///     primitive_job.delete().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    pub async fn delete(&self) -> Result<()> {
        self.client.delete_job(&self.job_id).await
    }
}
