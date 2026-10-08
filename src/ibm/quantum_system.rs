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

use crate::common::{resolve_opt, resolve_opt_required};
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
    /// Where the `QRMI_IBM_QS_*` settings are read from. Kept so that
    /// `QRMI_JOB_TIMEOUT_SECONDS` is read when `task_start` runs.
    settings: Settings,
    /// S3 bucket that job results and logs are read from. `None` if S3 was
    /// not configured at construction.
    s3: Option<S3Store>,
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
        Self::from_settings(resource_id, Settings::env(resource_id))
    }

    /// Constructs a QRMI to access IBM Quantum System API Service from a
    /// config map, instead of environment variables.
    ///
    /// Accepts the same keys as [`Self::new`]'s environment variables,
    /// minus the `<resource_id>_` prefix. Each key also accepts its fully
    /// lowercased form (e.g. `qrmi_ibm_qs_endpoint`) as a fallback if the
    /// exact-case key isn't present in the map.
    pub fn from_config(resource_id: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_settings(resource_id, Settings::map(config))
    }

    /// Shared logic for [`Self::new`] and [`Self::from_config`].
    fn from_settings(resource_id: &str, settings: Settings) -> Result<Self> {
        // Check to see if the settings required to run this program are set.
        let daapi_endpoint = settings.require("QRMI_IBM_QS_ENDPOINT")?;

        let mut builder = ClientBuilder::new(daapi_endpoint);

        let apikey = settings.require("QRMI_IBM_QS_IAM_APIKEY")?;
        let service_crn = settings.require("QRMI_IBM_QS_SERVICE_CRN")?;
        let iam_endpoint_url = settings.require("QRMI_IBM_QS_IAM_ENDPOINT")?;

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

        let s3_endpoint_for_daapi = settings.get("QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI");

        // S3 is optional: only the task_* methods need it. If it is configured,
        // the API client uses it to upload job input, and our own S3 client
        // uses it to download job results and logs.
        let s3 = match s3_env(&settings) {
            Ok(s3) => {
                builder.with_s3bucket(
                    &s3.access_key_id,
                    &s3.secret_access_key,
                    &s3.endpoint,
                    &s3.bucket,
                    &s3.region,
                    s3_endpoint_for_daapi,
                );
                Some(S3Store {
                    client: S3Client::new(
                        s3.endpoint,
                        s3.access_key_id,
                        s3.secret_access_key,
                        s3.region,
                    ),
                    bucket: s3.bucket,
                })
            }
            Err(e) => {
                info!("No S3 bucket configured ({e}).");
                None
            }
        };

        Ok(Self {
            api_client: builder.build()?,
            backend_name: resource_id.to_string(),
            settings,
            s3,
        })
    }

    /// The S3 bucket, or an error if S3 was not configured at construction.
    ///
    /// The error is the one [`s3_env`] reports for the first missing setting
    /// ([`QrmiError::EnvVarNotSet`] or [`QrmiError::MissingConfigKey`]).
    fn s3_store(&self) -> Result<&S3Store> {
        match &self.s3 {
            Some(s3) => Ok(s3),
            None => Err(s3_env(&self.settings).err().unwrap_or_else(|| {
                // Only reachable if the settings were added after construction.
                QrmiError::InvalidConfig(format!(
                    "S3 bucket was not configured when '{}' was created",
                    self.backend_name
                ))
            })),
        }
    }
}

/// Where the `QRMI_IBM_QS_*` settings are read from: OS environment variables
/// prefixed with `<resource_id>_`, or an unprefixed config map (config maps are
/// already scoped to one resource, so there's nothing to prefix).
struct Settings {
    prefix: String,
    config: Option<HashMap<String, String>>,
}

impl Settings {
    fn env(resource_id: &str) -> Self {
        Self {
            prefix: format!("{resource_id}_"),
            config: None,
        }
    }

    fn map(config: HashMap<String, String>) -> Self {
        Self {
            prefix: String::new(),
            config: Some(config),
        }
    }

    /// The full key looked up for `name`, as it appears in errors.
    fn key(&self, name: &str) -> String {
        format!("{}{name}", self.prefix)
    }

    fn get(&self, name: &str) -> Option<String> {
        resolve_opt(&self.key(name), self.config.as_ref())
    }

    /// Like [`Self::get`], but a missing value is an error
    /// ([`QrmiError::EnvVarNotSet`] or [`QrmiError::MissingConfigKey`]).
    fn require(&self, name: &str) -> Result<String> {
        resolve_opt_required(&self.key(name), self.config.as_ref())
    }
}

/// S3 connection details, read once at construction.
struct S3Env {
    bucket: String,
    endpoint: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

fn s3_env(settings: &Settings) -> Result<S3Env> {
    Ok(S3Env {
        bucket: settings.require("QRMI_IBM_QS_S3_BUCKET")?,
        endpoint: settings.require("QRMI_IBM_QS_S3_ENDPOINT")?,
        access_key_id: settings.require("QRMI_IBM_QS_AWS_ACCESS_KEY_ID")?,
        secret_access_key: settings.require("QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY")?,
        region: settings.require("QRMI_IBM_QS_S3_REGION")?,
    })
}

/// S3 client and bucket that job results and logs are read from.
struct S3Store {
    client: S3Client,
    bucket: String,
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
        // get_backend is mandatory: without it we cannot determine Online/Offline.
        let backend = self
            .api_client
            .get_backend::<Backend>(&self.backend_name)
            .await?;

        // get_backend_lanes_configuration requires the `direct-access-lane-configuration.list`
        // IAM permission. Accounts that lack this permission receive a 403, so
        // treat any failure here as "capacity info unavailable" rather than a
        // fatal error -- mirroring the approach taken for IQMServer in #287.
        let mut capacity_error = None;
        let capacity = match tokio::join!(
            self.api_client
                .get_backend_lanes_configuration::<BackendLanesConfiguration>(
                    &self.backend_name
                ),
            self.api_client.list_jobs::<Jobs>()
        ) {
            (Ok(lane_config), Ok(jobs)) => {
                // `list_jobs` also returns finished jobs; only running ones occupy a lane.
                let count = jobs
                    .jobs
                    .iter()
                    .filter(|job| job.backend == self.backend_name)
                    .filter(|job| matches!(job.status, JobStatus::Running))
                    .count() as u64;
                Some(ResourceCapacity {
                    available_slots: lane_config.hpc_workload_manager.lanes.saturating_sub(count),
                    max_slots: lane_config.hpc_workload_manager.lanes,
                })
            }
            (Err(e), _) => {
                // The most common cause is the account missing the
                // `direct-access-lane-configuration.list` IAM permission (HTTP 403).
                capacity_error = Some(format!(
                    "could not retrieve lane configuration ({}); \
                     capacity info will not be available",
                    e
                ));
                None
            }
            (_, Err(e)) => {
                capacity_error = Some(format!(
                    "could not retrieve job list ({}); \
                     capacity info will not be available",
                    e
                ));
                None
            }
        };

        let status = match backend.status {
            BackendStatus::Online => ResourceStatusCode::Online,
            BackendStatus::Offline => ResourceStatusCode::Offline,
            BackendStatus::Paused => ResourceStatusCode::Paused,
        };

        Ok(ResourceStatus {
            status,
            status_reason: capacity_error,
            busy: backend.locked,
            healthy: None,
            capacity,
            pending_job_count: None,
        })
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let timeout = self.settings.require("QRMI_JOB_TIMEOUT_SECONDS")?;
        let timeout_secs = timeout
            .parse::<u64>()
            .map_err(|source| QrmiError::ParseError {
                name: self.settings.key("QRMI_JOB_TIMEOUT_SECONDS"),
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
        let s3 = self.s3_store()?;

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
        let object = s3.client.get_object(&s3.bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(TaskResult {
            value: retrieved_txt,
        })
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let s3 = self.s3_store()?;

        let s3_object_key = format!("logs_{}.json", task_id);
        let object = s3.client.get_object(&s3.bucket, &s3_object_key).await?;
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
