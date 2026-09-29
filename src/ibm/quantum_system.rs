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

use crate::common::{not_found_error, resolve_opt, resolve_opt_required};
use crate::error::QrmiError;
use crate::ibm::error::IbmError;
use crate::models::{
    Payload, ResourceCapacity, ResourceStatus, ResourceStatusCode, ResourceType, Target,
    TaskResult, TaskStatus,
};
use crate::{QuantumResource, Result};
use log::info;
use quantum_system_api::utils::s3::S3Client;
use quantum_system_api::{
    models::Backend, models::BackendLanesConfiguration, models::BackendStatus, models::Job,
    models::JobStatus, models::Jobs, models::LogLevel, models::ProgramId, AuthMethod, Client,
    ClientBuilder,
};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::Jitter;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;

/// QRMI implementation for IBM Quantum System API
pub struct IBMQuantumSystem {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
    /// Settings only needed for job execution, resolved once at construction
    /// so that task operations never read process environment variables.
    pub(crate) task_settings: TaskSettings,
}

/// Job execution settings for [`IBMQuantumSystem`]. These are optional at
/// construction (scheduler integrations don't need them) and only become
/// required when a task operation uses them.
#[derive(Default)]
pub(crate) struct TaskSettings {
    /// Prefix of the keys the settings were read from: `<resource_id>_` for
    /// environment variables, empty for a config map.
    pub(crate) key_prefix: String,
    /// Whether the settings were read from a config map rather than the
    /// environment. Only affects which error is raised for a missing value.
    pub(crate) from_config: bool,
    pub(crate) timeout_secs: Option<u64>,
    pub(crate) s3_bucket: Option<String>,
    pub(crate) s3_endpoint: Option<String>,
    pub(crate) s3_access_key_id: Option<String>,
    pub(crate) s3_secret_access_key: Option<String>,
    pub(crate) s3_region: Option<String>,
}

impl TaskSettings {
    fn required<'a>(&self, value: &'a Option<String>, key: &str) -> Result<&'a String> {
        value.as_ref().ok_or_else(|| self.missing(key))
    }

    fn missing(&self, key: &str) -> QrmiError {
        not_found_error(format!("{}{key}", self.key_prefix), self.from_config)
    }

    pub(crate) fn timeout_secs(&self) -> Result<u64> {
        self.timeout_secs
            .ok_or_else(|| self.missing("QRMI_JOB_TIMEOUT_SECONDS"))
    }

    pub(crate) fn s3(&self) -> Result<S3Env> {
        Ok(S3Env {
            bucket: self
                .required(&self.s3_bucket, "QRMI_IBM_QS_S3_BUCKET")?
                .clone(),
            endpoint: self
                .required(&self.s3_endpoint, "QRMI_IBM_QS_S3_ENDPOINT")?
                .clone(),
            access_key_id: self
                .required(&self.s3_access_key_id, "QRMI_IBM_QS_AWS_ACCESS_KEY_ID")?
                .clone(),
            secret_access_key: self
                .required(
                    &self.s3_secret_access_key,
                    "QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY",
                )?
                .clone(),
            region: self
                .required(&self.s3_region, "QRMI_IBM_QS_S3_REGION")?
                .clone(),
        })
    }
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
        Self::from_opt(resource_id, None)
    }

    /// Constructs a QRMI to access IBM Quantum System API Service from a
    /// config map, instead of environment variables.
    ///
    /// Accepts the same keys as [`Self::new`]'s environment variables,
    /// minus the `<resource_id>_` prefix. Each key also accepts its fully
    /// lowercased form (e.g. `qrmi_ibm_qs_endpoint`) as a fallback if the
    /// exact-case key isn't present in the map.
    pub fn from_config(resource_id: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_opt(resource_id, Some(&config))
    }

    /// Shared parsing and client-building logic for [`Self::new`]
    /// (`config: None`, reads OS environment variables) and
    /// [`Self::from_config`] (`config: Some`, reads the given map).
    fn from_opt(resource_id: &str, config: Option<&HashMap<String, String>>) -> Result<Self> {
        // Config keys are the same name as the env vars, minus the
        // `<resource_id>_` prefix (config maps are already scoped to one
        // resource, so there's nothing to prefix).
        let prefix = if config.is_some() {
            String::new()
        } else {
            format!("{resource_id}_")
        };
        let daapi_endpoint =
            resolve_opt_required(&format!("{prefix}QRMI_IBM_QS_ENDPOINT"), config)?;

        let binding = ClientBuilder::new(daapi_endpoint);
        let mut builder = binding;

        let apikey = resolve_opt_required(&format!("{prefix}QRMI_IBM_QS_IAM_APIKEY"), config)?;
        let service_crn =
            resolve_opt_required(&format!("{prefix}QRMI_IBM_QS_SERVICE_CRN"), config)?;
        let iam_endpoint_url =
            resolve_opt_required(&format!("{prefix}QRMI_IBM_QS_IAM_ENDPOINT"), config)?;

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

        let s3_endpoint_for_daapi = resolve_opt(
            &format!("{prefix}QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI"),
            config,
        );

        let timeout_var = format!("{prefix}QRMI_JOB_TIMEOUT_SECONDS");
        let timeout_secs = resolve_opt(&timeout_var, config)
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|source| QrmiError::ParseError {
                        name: timeout_var,
                        value,
                        source: Box::new(source),
                    })
            })
            .transpose()?;

        let task_settings = TaskSettings {
            key_prefix: prefix.clone(),
            from_config: config.is_some(),
            timeout_secs,
            s3_bucket: resolve_opt(&format!("{prefix}QRMI_IBM_QS_S3_BUCKET"), config),
            s3_endpoint: resolve_opt(&format!("{prefix}QRMI_IBM_QS_S3_ENDPOINT"), config),
            s3_access_key_id: resolve_opt(
                &format!("{prefix}QRMI_IBM_QS_AWS_ACCESS_KEY_ID"),
                config,
            ),
            s3_secret_access_key: resolve_opt(
                &format!("{prefix}QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY"),
                config,
            ),
            s3_region: resolve_opt(&format!("{prefix}QRMI_IBM_QS_S3_REGION"), config),
        };

        if let Ok(s3) = task_settings.s3() {
            builder.with_s3bucket(
                &s3.access_key_id,
                &s3.secret_access_key,
                &s3.endpoint,
                &s3.bucket,
                &s3.region,
                s3_endpoint_for_daapi,
            );
        } else {
            info!("No S3 bucket configured.");
        }

        Ok(Self {
            api_client: builder.build()?,
            backend_name: resource_id.to_string(),
            task_settings,
        })
    }
}

/// S3 connection details, resolved from [`TaskSettings`]. Used by
/// [`IBMQuantumSystem::task_result`] and [`IBMQuantumSystem::task_logs`],
/// which both need to fetch an object from S3.
pub(crate) struct S3Env {
    pub(crate) bucket: String,
    endpoint: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
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

    async fn status(&mut self) -> Result<ResourceStatus> {
        let (backend, lane_config, jobs) = tokio::try_join!(
            self.api_client.get_backend::<Backend>(&self.backend_name),
            self.api_client
                .get_backend_lanes_configuration::<BackendLanesConfiguration>(&self.backend_name),
            self.api_client.list_jobs::<Jobs>()
        )?;

        let count = jobs
            .jobs
            .iter()
            .filter(|job| job.backend == self.backend_name)
            .count() as u64;

        let status = match backend.status {
            BackendStatus::Online => ResourceStatusCode::Online,
            BackendStatus::Offline => ResourceStatusCode::Offline,
            BackendStatus::Paused => ResourceStatusCode::Paused,
        };

        Ok(ResourceStatus {
            status,
            status_reason: None,
            busy: backend.locked,
            healthy: None,
            capacity: Some(ResourceCapacity {
                available_slots: lane_config.hpc_workload_manager.lanes.saturating_sub(count),
                max_slots: lane_config.hpc_workload_manager.lanes,
            }),
            pending_job_count: None,
        })
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let timeout_secs = self.task_settings.timeout_secs()?;

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
        let s3 = self.task_settings.s3()?;
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
        let s3 = self.task_settings.s3()?;
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
