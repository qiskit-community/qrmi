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

use crate::error::{required_config, required_env, QrmiError};
use crate::ibm::error::IbmError;
use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::{QuantumResource, Result};
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
        let apikey = required_env(format!("{resource_id}_QRMI_IBM_QS_IAM_APIKEY"))?;
        let service_crn = required_env(format!("{resource_id}_QRMI_IBM_QS_SERVICE_CRN"))?;
        let iam_endpoint_url = required_env(format!("{resource_id}_QRMI_IBM_QS_IAM_ENDPOINT"))?;

        let s3_endpoint_for_daapi =
            env::var(format!("{resource_id}_QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI")).ok();
        let s3 = if let (
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
            Some(S3BuilderParams {
                aws_access_key_id,
                aws_secret_access_key,
                s3_endpoint,
                s3_endpoint_for_daapi,
                s3_bucket,
                s3_region,
            })
        } else {
            None
        };

        Self::from_parts(
            resource_id,
            daapi_endpoint,
            apikey,
            service_crn,
            iam_endpoint_url,
            s3,
        )
    }

    /// Constructs a QRMI to access IBM Quantum System API Service from a
    /// config map, instead of environment variables.
    ///
    /// # Required keys
    ///
    /// * `endpoint` - IBM Quantum System API endpoint URL
    /// * `iam_apikey` - IBM Cloud API Key
    /// * `service_crn` - Provisioned Quantum System API Service instance
    /// * `iam_endpoint` - IBM Cloud IAM API endpoint URL
    ///
    /// # Optional keys (all required together to enable S3 access)
    ///
    /// * `aws_access_key_id`
    /// * `aws_secret_access_key`
    /// * `s3_endpoint`
    /// * `s3_bucket`
    /// * `s3_region`
    /// * `s3_endpoint_for_qsapi` - Optional override of `s3_endpoint` as seen from the service
    pub fn from_config(resource_id: &str, config: HashMap<String, String>) -> Result<Self> {
        let daapi_endpoint = required_config(&config, "endpoint")?;
        let apikey = required_config(&config, "iam_apikey")?;
        let service_crn = required_config(&config, "service_crn")?;
        let iam_endpoint_url = required_config(&config, "iam_endpoint")?;

        let s3 = match (
            config.get("aws_access_key_id").cloned(),
            config.get("aws_secret_access_key").cloned(),
            config.get("s3_endpoint").cloned(),
            config.get("s3_bucket").cloned(),
            config.get("s3_region").cloned(),
        ) {
            (
                Some(aws_access_key_id),
                Some(aws_secret_access_key),
                Some(s3_endpoint),
                Some(s3_bucket),
                Some(s3_region),
            ) => Some(S3BuilderParams {
                aws_access_key_id,
                aws_secret_access_key,
                s3_endpoint,
                s3_endpoint_for_daapi: config.get("s3_endpoint_for_qsapi").cloned(),
                s3_bucket,
                s3_region,
            }),
            _ => None,
        };

        Self::from_parts(
            resource_id,
            daapi_endpoint,
            apikey,
            service_crn,
            iam_endpoint_url,
            s3,
        )
    }

    /// Builds the IBM Quantum System API client from already-resolved
    /// connection details, shared by [`Self::new`] (resolved from env vars)
    /// and [`Self::from_config`] (resolved from a config map).
    fn from_parts(
        resource_id: &str,
        daapi_endpoint: String,
        apikey: String,
        service_crn: String,
        iam_endpoint_url: String,
        s3: Option<S3BuilderParams>,
    ) -> Result<Self> {
        let mut builder = ClientBuilder::new(daapi_endpoint);
        builder.with_auth(AuthMethod::IbmCloudIam {
            apikey,
            service_crn,
            iam_endpoint_url,
        });

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        builder
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy);

        match s3 {
            Some(s3) => {
                builder.with_s3bucket(
                    &s3.aws_access_key_id,
                    &s3.aws_secret_access_key,
                    &s3.s3_endpoint,
                    &s3.s3_bucket,
                    &s3.s3_region,
                    s3.s3_endpoint_for_daapi,
                );
            }
            None => info!("No S3 bucket configured."),
        }

        Ok(Self {
            api_client: builder.build().unwrap(),
            backend_name: resource_id.to_string(),
        })
    }
}

/// S3 bucket connection details for the job-execution client, resolved
/// either from env vars ([`IBMQuantumSystem::new`]) or a config map
/// ([`IBMQuantumSystem::from_config`]).
struct S3BuilderParams {
    aws_access_key_id: String,
    aws_secret_access_key: String,
    s3_endpoint: String,
    s3_endpoint_for_daapi: Option<String>,
    s3_bucket: String,
    s3_region: String,
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
            .await?;
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
            .await?;
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
