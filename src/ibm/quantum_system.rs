// This code is part of Qiskit.
//
// (C) Copyright IBM 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::error::{required_env, QrmiError};
use crate::ibm::error::IbmError;
use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::{QuantumResource, Result};
use anyhow::Context;
use log::info;
use quantum_system_api::utils::s3::S3Client;
use quantum_system_api::{
    models::Backend, models::BackendStatus, models::Job, models::JobStatus, models::LogLevel,
    models::ProgramId, AuthMethod, Client, ClientBuilder,
};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::Jitter;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

use async_trait::async_trait;

/// QRMI implementation for IBM Quantum System API
pub struct IBMQuantumSystem {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

impl IBMQuantumSystem {
    /// Constructs a QRMI to access IBM Quantum System API Service
    ///
    /// # Environment variables
    ///
    /// * `QRMI_IBM_QS_ENDPOINT`: IBM Quantum System API endpoint URL
    /// * `QRMI_IBM_QS_AWS_ACCESS_KEY_ID`: AWS Access Key ID to access S3 bucket
    /// * `QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY`: AWS Secret Access Key to access S3 bucket
    /// * `QRMI_IBM_QS_S3_ENDPOINT`: S3 API endpoint URL
    /// * `QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI`: S3 API endpoint URL accessed from Quantum System API service. Depending on the network configuration, the IP address used to access S3 may differ between access from the API client and access from the Quantum System API service. In such cases, this environment variable should specify the URL used when accessing S3 from the DA API service.
    /// * `QRMI_IBM_QS_S3_BUCKET`: S3 Bucket name
    /// * `QRMI_IBM_QS_S3_REGION`: S3 Region name
    /// * `QRMI_IBM_QS_IAM_ENDPOINT`: IBM Cloud IAM API endpoint URL
    /// * `QRMI_IBM_QS_IAM_APIKEY`: IBM Cloud API Key
    /// * `QRMI_IBM_QS_SERVICE_CRN`: Provisioned Quantum System API Service instance
    /// * `QRMI_JOB_TIMEOUT_SECONDS`: Time (in seconds) after which job should time out and get cancelled.
    pub fn new(resource_id: &str) -> Result<Self> {
        // Check to see if the environment variables required to run this program are set.
        let daapi_endpoint = required_env(format!("{resource_id}_QRMI_IBM_QS_ENDPOINT"))?;

        let binding = ClientBuilder::new(daapi_endpoint);
        let mut builder = binding;

        let apikey = required_env(format!("{resource_id}_QRMI_IBM_QS_IAM_APIKEY"))?;
        let service_crn = required_env(format!("{resource_id}_QRMI_IBM_QS_SERVICE_CRN"))?;
        let iam_endpoint_url = required_env(format!("{resource_id}_QRMI_IBM_QS_IAM_ENDPOINT"))?;

        let auth_method = AuthMethod::IbmCloudIam {
            apikey,
            service_crn,
            iam_endpoint_url,
        };
        builder.with_auth(auth_method);

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        builder
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy);

        let s3_endpoint_for_daapi =
            env::var(format!("{resource_id}_QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI")).ok();

        if let (
            Ok(aws_access_key_id),
            Ok(aws_secret_access_key),
            Ok(s3_endpoint),
            Ok(s3_bucket),
            Ok(s3_region),
        ) = (
            env::var(format!("{resource_id}_QRMI_IBM_QS_AWS_ACCESS_KEY_ID")),
            env::var(format!("{resource_id}_QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY")),
            env::var(format!("{resource_id}_QRMI_IBM_QS_S3_ENDPOINT")),
            env::var(format!("{resource_id}_QRMI_IBM_QS_S3_BUCKET")),
            env::var(format!("{resource_id}_QRMI_IBM_QS_S3_REGION")),
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

        Ok(Self {
            api_client: builder.build().unwrap(),
            backend_name: resource_id.to_string(),
        })
    }
}

/// S3 connection details, read from the `<backend_name>_QRMI_IBM_QS_*` environment
/// variables. Used by [`IBMQuantumSystem::task_result`] and
/// [`IBMQuantumSystem::task_logs`], which both need to fetch an object from S3.
struct S3Env {
    bucket: String,
    endpoint: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

fn s3_env(backend_name: &str) -> Result<S3Env> {
    Ok(S3Env {
        bucket: required_env(format!("{backend_name}_QRMI_IBM_QS_S3_BUCKET"))?,
        endpoint: required_env(format!("{backend_name}_QRMI_IBM_QS_S3_ENDPOINT"))?,
        access_key_id: required_env(format!("{backend_name}_QRMI_IBM_QS_AWS_ACCESS_KEY_ID"))?,
        secret_access_key: required_env(format!(
            "{backend_name}_QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY"
        ))?,
        region: required_env(format!("{backend_name}_QRMI_IBM_QS_S3_REGION"))?,
    })
}

#[async_trait]
impl QuantumResource for IBMQuantumSystem {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::IBMQuantumSystem)
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        let backend = self
            .api_client
            .get_backend::<Backend>(&self.backend_name)
            .await
            .context("failed to get backend details")?;
        Ok(matches!(backend.status, BackendStatus::Online))
    }

    async fn acquire(&mut self) -> Result<String> {
        // Quantum System API does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // Quantum System API does not support session concept, so simply ignores
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let timeout_env_name = format!("{0}_QRMI_JOB_TIMEOUT_SECONDS", self.backend_name);
        let timeout = required_env(&timeout_env_name)?;
        let timeout_secs = timeout
            .parse::<u64>()
            .map_err(|source| QrmiError::ParseError {
                name: timeout_env_name,
                value: timeout,
                source: Box::new(source),
            })?;

        let Payload::QiskitPrimitive { input, program_id } = payload else {
            return Err(QrmiError::UnsupportedPayload(format!("{payload:?}")));
        };

        let job_input: serde_json::Value = serde_json::from_str(input.as_str())?;
        let program_id_enum = ProgramId::from_str(&program_id)
            .map_err(|_| IbmError::UnknownProgramId(program_id.clone()))?;

        let job = self
            .api_client
            .run_primitive(
                &self.backend_name,
                program_id_enum,
                timeout_secs,
                LogLevel::Debug,
                &job_input,
                None,
            )
            .await
            .context("failed to start task")?;
        Ok(job.job_id)
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
        let s3 = s3_env(&self.backend_name)?;
        let s3_client = S3Client::new(
            s3.endpoint,
            s3.access_key_id,
            s3.secret_access_key,
            s3.region,
        );

        let job = self.api_client.get_job::<Job>(task_id).await?;
        if matches!(job.status, JobStatus::Failed) {
            let reason_code = job.reason_code.map_or("".to_string(), |v| v.to_string());
            let reason_message = job.reason_message.unwrap_or("".to_string());
            let reason_solution = job.reason_solution.unwrap_or("".to_string());
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: format!(
                    "task failed. code: {reason_code}, message: {reason_message}, solution: {reason_solution}"
                ),
            });
        }
        if matches!(job.status, JobStatus::Cancelled) {
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: "task was cancelled".to_string(),
            });
        }
        if matches!(job.status, JobStatus::Running) {
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: "task is running".to_string(),
            });
        }
        let s3_object_key = format!("results_{}.json", task_id);
        let object = s3_client.get_object(&s3.bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(TaskResult {
            value: retrieved_txt,
        })
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let s3 = s3_env(&self.backend_name)?;
        let s3_client = S3Client::new(
            s3.endpoint,
            s3.access_key_id,
            s3.secret_access_key,
            s3.region,
        );

        let s3_object_key = format!("logs_{}.json", task_id);
        let object = s3_client.get_object(&s3.bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(retrieved_txt)
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

#[cfg(test)]
#[path = "tests/quantum_system.rs"]
mod tests;
