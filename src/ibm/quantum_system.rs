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
use log::{info, warn};
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
    /// Job timeout (`QRMI_JOB_TIMEOUT_SECONDS`). Optional at construction,
    /// required by [`QuantumResource::task_start`].
    timeout_secs: std::result::Result<u64, Missing>,
    /// S3 bucket that job results and logs are read from. Optional at
    /// construction, required by [`QuantumResource::task_result`] and
    /// [`QuantumResource::task_logs`].
    s3: std::result::Result<S3Store, Missing>,
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
    ///
    /// The five S3 settings (all except `QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI`)
    /// are only used together: if any of them is missing, S3 is disabled and
    /// `task_start` / `task_result` / `task_logs` fail. `QRMI_JOB_TIMEOUT_SECONDS`
    /// is only required by `task_start`, but if set it must be a valid integer.
    pub fn new(resource_id: &str) -> Result<Self> {
        Self::from_settings(resource_id, &Settings::env(resource_id))
    }

    /// Constructs a QRMI to access IBM Quantum System API Service from a
    /// config map, instead of environment variables.
    ///
    /// Accepts the same keys as [`Self::new`]'s environment variables,
    /// minus the `<resource_id>_` prefix. Each key also accepts its fully
    /// lowercased form (e.g. `qrmi_ibm_qs_endpoint`) as a fallback if the
    /// exact-case key isn't present in the map.
    pub fn from_config(resource_id: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_settings(resource_id, &Settings::map(&config))
    }

    /// Shared parsing and client-building logic for [`Self::new`] and
    /// [`Self::from_config`]. Every setting is resolved here, once, so that
    /// the `QuantumResource` methods never look at the environment directly
    /// and behave the same whichever constructor was used.
    fn from_settings(resource_id: &str, settings: &Settings) -> Result<Self> {
        let mut builder = ClientBuilder::new(settings.require("QRMI_IBM_QS_ENDPOINT")?);

        builder.with_auth(AuthMethod::IbmCloudIam {
            apikey: settings.require("QRMI_IBM_QS_IAM_APIKEY")?,
            service_crn: settings.require("QRMI_IBM_QS_SERVICE_CRN")?,
            iam_endpoint_url: settings.require("QRMI_IBM_QS_IAM_ENDPOINT")?,
        });

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        builder
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy);

        // Parse eagerly so that a malformed value is reported at construction
        // time rather than on the first `task_start`.
        let timeout_secs = match settings.optional("QRMI_JOB_TIMEOUT_SECONDS") {
            Ok(value) => Ok(value
                .parse::<u64>()
                .map_err(|source| QrmiError::ParseError {
                    name: settings.key("QRMI_JOB_TIMEOUT_SECONDS"),
                    value,
                    source: Box::new(source),
                })?),
            Err(missing) => Err(missing),
        };

        // The same S3 settings feed both the API client (which uploads job
        // input) and our own S3 client (which downloads results and logs).
        let s3_settings = S3Settings::resolve(settings);
        match &s3_settings {
            Ok(s3) => {
                builder.with_s3bucket(
                    &s3.access_key_id,
                    &s3.secret_access_key,
                    &s3.endpoint,
                    &s3.bucket,
                    &s3.region,
                    settings.get("QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI"),
                );
            }
            Err(_) => info!("No S3 bucket configured."),
        }

        Ok(Self {
            api_client: builder.build()?,
            backend_name: resource_id.to_string(),
            timeout_secs,
            s3: s3_settings.map(S3Store::new),
        })
    }
}

/// Where `QRMI_IBM_QS_*` settings are read from: OS environment variables
/// prefixed with `<resource_id>_`, or an unprefixed config map (config maps are
/// already scoped to one resource, so there's nothing to prefix).
struct Settings<'a> {
    prefix: String,
    config: Option<&'a HashMap<String, String>>,
}

impl<'a> Settings<'a> {
    fn env(resource_id: &str) -> Self {
        Self {
            prefix: format!("{resource_id}_"),
            config: None,
        }
    }

    fn map(config: &'a HashMap<String, String>) -> Self {
        Self {
            prefix: String::new(),
            config: Some(config),
        }
    }

    /// The full key looked up for `name`, as it should appear in errors.
    fn key(&self, name: &str) -> String {
        format!("{}{name}", self.prefix)
    }

    fn get(&self, name: &str) -> Option<String> {
        resolve_opt(&self.key(name), self.config)
    }

    /// A setting the resource cannot be constructed without.
    fn require(&self, name: &str) -> Result<String> {
        resolve_opt_required(&self.key(name), self.config)
    }

    /// A setting only some operations need. If it is absent, the returned
    /// [`Missing`] lets those operations report it later with the same error
    /// kind [`Self::require`] would have used.
    fn optional(&self, name: &str) -> std::result::Result<String, Missing> {
        self.get(name).ok_or_else(|| Missing {
            key: self.key(name),
            from_env: self.config.is_none(),
        })
    }
}

/// A setting that was absent at construction but is needed by an operation.
#[derive(Debug, Clone)]
struct Missing {
    key: String,
    from_env: bool,
}

impl Missing {
    /// The error [`resolve_opt_required`] would have returned for this key.
    fn to_error(&self) -> QrmiError {
        if self.from_env {
            QrmiError::EnvVarNotSet(self.key.clone())
        } else {
            QrmiError::MissingConfigKey(self.key.clone())
        }
    }
}

/// The five S3 settings, which are only meaningful together.
struct S3Settings {
    bucket: String,
    endpoint: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

impl S3Settings {
    /// Resolves all five settings. If none is set, S3 is silently disabled;
    /// if only some are, the missing ones are logged since that is almost
    /// certainly a misconfiguration. Either way the first missing setting is
    /// returned, to be reported when an operation needs S3.
    fn resolve(settings: &Settings) -> std::result::Result<Self, Missing> {
        let bucket = settings.optional("QRMI_IBM_QS_S3_BUCKET");
        let endpoint = settings.optional("QRMI_IBM_QS_S3_ENDPOINT");
        let access_key_id = settings.optional("QRMI_IBM_QS_AWS_ACCESS_KEY_ID");
        let secret_access_key = settings.optional("QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY");
        let region = settings.optional("QRMI_IBM_QS_S3_REGION");

        let all = [
            &bucket,
            &endpoint,
            &access_key_id,
            &secret_access_key,
            &region,
        ];
        let missing: Vec<&str> = all
            .iter()
            .filter_map(|r| r.as_ref().err().map(|m| m.key.as_str()))
            .collect();
        if !missing.is_empty() && missing.len() < all.len() {
            warn!(
                "Incomplete S3 configuration, S3 is disabled. Missing: {}",
                missing.join(", ")
            );
        }

        Ok(Self {
            bucket: bucket?,
            endpoint: endpoint?,
            access_key_id: access_key_id?,
            secret_access_key: secret_access_key?,
            region: region?,
        })
    }
}

/// S3 bucket that job results and logs are read from.
struct S3Store {
    client: S3Client,
    bucket: String,
}

impl S3Store {
    fn new(settings: S3Settings) -> Self {
        Self {
            client: S3Client::new(
                settings.endpoint,
                settings.access_key_id,
                settings.secret_access_key,
                settings.region,
            ),
            bucket: settings.bucket,
        }
    }

    /// Reads an object and decodes it as UTF-8 text.
    async fn get_text(&self, key: &str) -> Result<String> {
        let object = self.client.get_object(&self.bucket, key).await?;
        Ok(String::from_utf8(object)?)
    }
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
        let timeout_secs = *self.timeout_secs.as_ref().map_err(Missing::to_error)?;

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
        let s3 = self.s3.as_ref().map_err(Missing::to_error)?;

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
        Ok(TaskResult {
            value: s3.get_text(&format!("results_{task_id}.json")).await?,
        })
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let s3 = self.s3.as_ref().map_err(Missing::to_error)?;
        s3.get_text(&format!("logs_{task_id}.json")).await
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
