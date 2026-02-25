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

use crate::Client;
use anyhow::{bail, Context, Result};
use log::debug;
use uuid::Uuid;

use aws_sdk_s3::error::DisplayErrorContext;
use serde::de::DeserializeOwned;

use crate::models::jobs::{JobStatus, LogLevel, ProgramId};
use crate::PrimitiveJob;

const S3KEY_INPUT_PREFIX: &str = "input_";
const S3KEY_RESULTS_PREFIX: &str = "results_";
const S3KEY_LOGS_PREFIX: &str = "logs_";

impl Client {
    /// Invokes a Qiskit Runtime primitive. Parameters to inject into the primitive are defined in [EstimatorV2 input](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/estimator_v2_schema.json) and [SamplerV2 input](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/sampler_v2_schema.json).
    /// [`Client`] needs to be created by the [`ClientBuilder`](crate::ClientBuilder) with [`with_s3bucket`](crate::ClientBuilder::with_s3bucket) to use this function.
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
    ///     let _primitive_job = client
    ///         .run_primitive("ibm_brisbane", ProgramId::Sampler, 3600, LogLevel::Info, &payload, None)
    ///         .await?;
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
    /// - S3 authentication failed.
    /// - S3 connection failed.
    /// - S3 bucket is not found.
    ///
    pub async fn run_primitive(
        &self,
        backend: &str,
        program_id: ProgramId,
        timeout_secs: u64,
        log_level: LogLevel,
        payload: &serde_json::Value,
        job_id: Option<String>,
    ) -> Result<PrimitiveJob> {
        let s3_config = self.s3_config.clone().context(
            "S3 bucket is not configured. Use ClientBuilder.with_s3_bucket() to use this function.",
        )?;

        let s3_config_remote: aws_sdk_s3::Config;
        if let Some(v) = self.s3_config_for_daapi.clone() {
            s3_config_remote = v;
        } else {
            s3_config_remote = s3_config.clone();
        }

        let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
        let s3_client_remote = aws_sdk_s3::Client::from_conf(s3_config_remote);
        let id;
        if let Some(value) = job_id {
            id = value;
        } else {
            id = Uuid::new_v4().to_string();
        }
        let s3_bucket = self.s3_bucket.clone().unwrap();

        let converted_vec = serde_json::to_vec::<serde_json::Value>(payload)?;
        let job_param_key = format!("{}{}.json", S3KEY_INPUT_PREFIX, id);
        let _ = match s3_client
            .put_object()
            .bucket(s3_bucket.clone())
            .key(job_param_key.clone())
            .body(converted_vec.into())
            .send()
            .await
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "An error occurred during upload to S3: {}",
                    DisplayErrorContext(&err)
                ));
            }
        };

        let input_presigned_url = crate::storages::s3::get_presigned_url(
            &s3_client_remote,
            &s3_bucket,
            &job_param_key,
            timeout_secs,
        )
        .await?;

        let results_key = format!("{}{}.json", S3KEY_RESULTS_PREFIX, id);
        let results_presigned_url = crate::storages::s3::get_presigned_url_for_put(
            &s3_client_remote,
            &s3_bucket,
            &results_key,
            timeout_secs,
        )
        .await?;

        let logs_key = format!("{}{}.json", S3KEY_LOGS_PREFIX, id);
        let logs_presigned_url = crate::storages::s3::get_presigned_url_for_put(
            &s3_client_remote,
            &s3_bucket,
            &logs_key,
            timeout_secs,
        )
        .await?;

        let job_param = serde_json::json!({
            "id": id,
            "backend": backend.to_string(),
            "program_id": program_id.to_string(),
            "log_level": log_level.to_string(),
            "timeout_secs": timeout_secs,
            "storage": {
                "input": {
                    "presigned_url": input_presigned_url,
                    "type": "s3_compatible".to_string(),
                },
                "results": {
                    "presigned_url": results_presigned_url,
                    "type": "s3_compatible".to_string(),
                },
                "logs": {
                    "presigned_url": logs_presigned_url,
                    "type": "s3_compatible".to_string(),
                },
            }
        });
        debug!("{:#?}", job_param);
        let job_id = self.run_job(&job_param).await?;
        Ok(PrimitiveJob {
            job_id,
            client: self.clone(),
            s3_client,
            s3_bucket,
        })
    }
}

impl PrimitiveJob {
    /// Return the results of the job.
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
    ///     let _result = primitive_job.get_result::<serde_json::Value>().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - Result is not available since job was not succeeded.
    /// - Result is not available until the job is completed.
    /// - S3 authentication failed.
    /// - S3 connection failed.
    /// - S3 bucket is not found.
    ///
    pub async fn get_result<T: DeserializeOwned>(&self) -> Result<T> {
        let status = self.get_status().await?;
        if JobStatus::Cancelled == status || JobStatus::Failed == status {
            bail!("Result is not available since job was not succeeded.".to_string());
        } else if JobStatus::Running == status {
            bail!("Result is not available until the job is completed.".to_string());
        }

        let key = format!("{}{}.json", S3KEY_RESULTS_PREFIX, self.job_id);
        let presigned_url =
            crate::storages::s3::get_presigned_url(&self.s3_client, &self.s3_bucket, &key, 3600)
                .await?;
        debug!("{}", presigned_url);

        let resp = self
            .client
            .plain_client
            .get(presigned_url)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        if resp.status().is_success() {
            let json_data = resp.json::<T>().await?;
            Ok(json_data)
        } else {
            let json_data = resp.json::<serde_json::Value>().await?;
            bail!(format!("{:?}", json_data))
        }
    }

    /// Return the logs of the job.
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
    ///     let _logs = primitive_job.get_logs().await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - Logs are not available until the job is in its final state.
    /// - S3 authentication failed.
    /// - S3 connection failed.
    /// - S3 bucket is not found.
    ///
    pub async fn get_logs(&self) -> Result<String> {
        let in_final_state = self.is_in_final_state().await?;
        if !in_final_state {
            bail!("Logs are not available until the job is in its final state.".to_string());
        }

        let key = format!("{}{}.json", S3KEY_LOGS_PREFIX, self.job_id);
        let presigned_url =
            crate::storages::s3::get_presigned_url(&self.s3_client, &self.s3_bucket, &key, 3600)
                .await?;
        debug!("{}", presigned_url);

        let resp = self
            .client
            .plain_client
            .get(presigned_url)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        let status = resp.status();
        let text_data = resp.text().await?;
        if status.is_success() {
            Ok(text_data)
        } else {
            bail!(text_data)
        }
    }
}
