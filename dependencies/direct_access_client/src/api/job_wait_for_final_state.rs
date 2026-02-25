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

use crate::models::jobs::{Job, JobStatus};
use crate::{Client, PrimitiveJob};
use anyhow::{bail, Result};
use std::time::{Duration, Instant};

impl Client {
    /// Polls for the job status from the API until the status is in a final state and
    /// returns [`Job`] once it is completed.
    /// Otherwise, returns the error if the job does not complete within given `timeout`
    /// if specified.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use direct_access_api::{AuthMethod, ClientBuilder};
    ///
    ///     let client = ClientBuilder::new("http://localhost:8080")
    ///         .with_auth(AuthMethod::IbmCloudIam {
    ///             apikey: "your_iam_apikey".to_string(),
    ///             service_crn: "your_service_crn".to_string(),
    ///             iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///         })
    ///         .build()
    ///         .unwrap();
    ///     let _status = client
    ///         .wait_for_job_final_state("bb2861da-d2c9-4de0-8f0b-4e399c4b02ac", Some(1800.0))
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
    /// - specified job is not found.
    pub async fn wait_for_job_final_state(
        &self,
        job_id: &str,
        timeout: Option<f64>,
    ) -> Result<Job> {
        let start_time = Instant::now();
        loop {
            if let Some(t) = timeout {
                let now = Instant::now();
                let elapsed = now.duration_since(start_time);
                if elapsed >= Duration::from_secs_f64(t) {
                    bail!("timeout occurred while waiting for completion".to_string());
                }
            }

            let job = self.get_job::<Job>(job_id).await?;
            if let JobStatus::Running = job.status {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            } else {
                // now in final state.
                return Ok(job);
            }
        }
    }
}

impl PrimitiveJob {
    /// Polls for the job status from the API until the status is in a final state and
    /// returns [`Job`] once it is completed.
    /// Otherwise, returns the error if the job does not complete within given `timeout`
    /// if specified.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use serde_json::json;
    ///     use direct_access_api::{AuthMethod, ClientBuilder, models::ProgramId, models::LogLevel};
    ///
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
    ///     let _status = primitive_job
    ///         .wait_for_final_state(Some(1800.0))
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
    /// - job is not found.
    pub async fn wait_for_final_state(&self, timeout: Option<f64>) -> Result<Job> {
        self.client
            .wait_for_job_final_state(&self.job_id, timeout)
            .await
    }
}
