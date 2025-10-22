// This code is part of Qiskit.
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

use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{bail, Result};
use direct_access_api::utils::s3::S3Client;
use direct_access_api::{
    models::Backend, models::BackendStatus, models::Job, models::JobStatus, models::LogLevel,
    models::ProgramId, AuthMethod, Client, ClientBuilder,
};
use log::info;
use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

use async_trait::async_trait;

/// QRMI implementation for IBM Qiskit Runtime Direct Access
pub struct IBMDirectAccess {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

const DEFAULT_ENDPOINT: &str = "http://localhost:8080";

impl IBMDirectAccess {
    /// Constructs a QRMI to access IBM Qiskit Runtime Direct Access Service
    ///
    /// # Environment variables
    ///
    /// * `QRMI_IBM_DA_ENDPOINT`: IBM Qiskit Runtime Direct Access API endpoint URL
    /// * `QRMI_IBM_DA_AWS_ACCESS_KEY_ID`: AWS Access Key ID to access S3 bucket
    /// * `QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY`: AWS Secret Access Key to access S3 bucket
    /// * `QRMI_IBM_DA_S3_ENDPOINT`: S3 API endpoint URL
    /// * `QRMI_IBM_DA_S3_ENDPOINT_FOR_DAAPI`: S3 API endpoint URL accessed from DA API service. Depending on the network configuration, the IP address used to access S3 may differ between access from the API client and access from the DA API service. In such cases, this environment variable should specify the URL used when accessing S3 from the DA API service.
    /// * `QRMI_IBM_DA_S3_BUCKET`: S3 Bucket name
    /// * `QRMI_IBM_DA_S3_REGION`: S3 Region name
    /// * `QRMI_IBM_DA_IAM_ENDPOINT`: IBM Cloud IAM API endpoint URL
    /// * `QRMI_IBM_DA_IAM_APIKEY`: IBM Cloud API Key
    /// * `QRMI_IBM_DA_SERVICE_CRN`: Provisioned Direct Access Service instance
    /// * `QRMI_JOB_TIMEOUT_SECONDS`: Time (in seconds) after which job should time out and get cancelled.
    pub fn new(resource_id: &str) -> Result<Self> {
        // Check to see if the environment variables required to run this program are set.
        let daapi_endpoint = env::var(format!("{resource_id}_QRMI_IBM_DA_ENDPOINT"))
            .unwrap_or(DEFAULT_ENDPOINT.to_string());

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        let binding = ClientBuilder::new(daapi_endpoint);
        let mut builder = binding;
        builder
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy);

        let s3_endpoint_for_daapi =
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_ENDPOINT_FOR_DAAPI")).ok();
        if let (
            Ok(aws_access_key_id),
            Ok(aws_secret_access_key),
            Ok(s3_endpoint),
            Ok(s3_bucket),
            Ok(s3_region),
        ) = (
            env::var(format!("{resource_id}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_ENDPOINT")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_BUCKET")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_REGION")),
        ) {
            builder.with_s3bucket(
                &aws_access_key_id,
                &aws_secret_access_key,
                &s3_endpoint,
                &s3_bucket,
                &s3_region,
                s3_endpoint_for_daapi,
            );
        } else {
            info!("No S3 bucket configured.");
        }

        if let (Ok(apikey), Ok(service_crn), Ok(iam_endpoint_url)) = (
            env::var(format!("{resource_id}_QRMI_IBM_DA_IAM_APIKEY")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_SERVICE_CRN")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_IAM_ENDPOINT")),
        ) {
            let auth_method = AuthMethod::IbmCloudIam {
                apikey,
                service_crn,
                iam_endpoint_url,
            };
            builder.with_auth(auth_method);
        } else {
            info!("No authentication configured.");
        }

        Ok(Self {
            api_client: builder.build().unwrap(),
            backend_name: resource_id.to_string(),
        })
    }
}

#[async_trait]
impl QuantumResource for IBMDirectAccess {
    async fn is_accessible(&mut self) -> bool {
        match self
            .api_client
            .get_backend::<Backend>(&self.backend_name)
            .await
        {
            Ok(val) => {
                if matches!(val.status, BackendStatus::Online) {
                    return true;
                }
                false
            }
            Err(_err) => false,
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        // Direct Access does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // Direct Access does not support session concept, so simply ignores
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let timeout = match env::var(format!("{0}_QRMI_JOB_TIMEOUT_SECONDS", self.backend_name)) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_JOB_TIMEOUT_SECONDS is not set: {}", &err));
            }
        };
        let timeout_secs = match timeout.parse::<u64>() {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("Failed to parse timeout value: {}", &err));
            }
        };

        if let Payload::QiskitPrimitive { input, program_id } = payload {
            let job: serde_json::Value = serde_json::from_str(input.as_str())?;
            if let Ok(program_id_enum) = ProgramId::from_str(&program_id) {
                match self
                    .api_client
                    .run_primitive(
                        &self.backend_name,
                        program_id_enum,
                        timeout_secs,
                        LogLevel::Debug,
                        &job,
                        None,
                    )
                    .await
                {
                    Ok(val) => Ok(val.job_id),
                    Err(err) => {
                        bail!(format!(
                            "An error occurred during starting a task: {}",
                            &err
                        ));
                    }
                }
            } else {
                bail!(format!("Unknown program ID is specified. {}", &program_id));
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload));
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let status = self.api_client.get_job_status(task_id).await?;
        if matches!(status, JobStatus::Running) {
            let _ = self.api_client.cancel_job(task_id, false).await;
        }
        self.api_client.delete_job(task_id).await?;
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let status = self.api_client.get_job_status(task_id).await?;
        match status {
            JobStatus::Running => Ok(TaskStatus::Running),
            JobStatus::Completed => Ok(TaskStatus::Completed),
            JobStatus::Cancelled => Ok(TaskStatus::Cancelled),
            JobStatus::Failed => Ok(TaskStatus::Failed),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let s3_bucket = match env::var(format!("{0}_QRMI_IBM_DA_S3_BUCKET", self.backend_name)) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_IBM_DA_S3_BUCKET is not set: {}", &err));
            }
        };

        let s3_endpoint = match env::var(format!("{0}_QRMI_IBM_DA_S3_ENDPOINT", self.backend_name))
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_IBM_DA_S3_ENDPOINT is not set: {}", &err));
            }
        };

        let aws_access_key_id = match env::var(format!(
            "{0}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID",
            self.backend_name
        )) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "QRMI_IBM_DA_AWS_ACCESS_KEY_ID is not set: {}",
                    &err
                ));
            }
        };

        let aws_secret_access_key = match env::var(format!(
            "{0}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY",
            self.backend_name
        )) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY is not set: {}",
                    &err
                ));
            }
        };

        let s3_region = match env::var(format!("{0}_QRMI_IBM_DA_S3_REGION", self.backend_name)) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_IBM_DA_S3_REGION is not set: {}", &err));
            }
        };

        let s3_client = S3Client::new(
            s3_endpoint,
            aws_access_key_id,
            aws_secret_access_key,
            s3_region,
        );

        let job = self.api_client.get_job::<Job>(task_id).await?;
        if matches!(job.status, JobStatus::Failed) {
            let reason_code = job.reason_code.map_or("".to_string(), |v| v.to_string());
            let reason_message = job.reason_message.unwrap_or("".to_string());
            let reason_solution = job.reason_solution.unwrap_or("".to_string());
            bail!(
                format!(
                    "Unable to retrieve result for task {}. Task failed. code: {}, message: {}, solution: {}",
                    task_id, reason_code, reason_message, reason_solution
                )
            );
        }
        if matches!(job.status, JobStatus::Cancelled) {
            bail!(format!(
                "Unable to retrieve result for task {}. Task was cancelled.",
                task_id
            ));
        }
        if matches!(job.status, JobStatus::Running) {
            bail!(format!(
                "Unable to retrieve result for task {}. Task is running.",
                task_id
            ));
        }
        let s3_object_key = format!("results_{}.json", task_id);
        let object = s3_client.get_object(&s3_bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(TaskResult {
            value: retrieved_txt,
        })
    }

    async fn target(&mut self) -> Result<Target> {
        let mut resp = json!({});
        if let Ok(config) = self
            .api_client
            .get_backend_configuration::<serde_json::Value>(&self.backend_name)
            .await
        {
            resp["configuration"] = config;
        } else {
            resp["configuration"] = json!(null);
        }

        if let Ok(props) = self
            .api_client
            .get_backend_properties::<serde_json::Value>(&self.backend_name)
            .await
        {
            resp["properties"] = props;
        } else {
            resp["properties"] = json!(null);
        }

        Ok(Target {
            value: resp.to_string(),
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}
