// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025, 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{anyhow, bail, Result};
use log::{debug, warn};
use pasqal_cloud_api::{Client, ClientBuilder, DeviceType, JobStatus};
use std::collections::HashMap;
use uuid::Uuid;

use super::cloud_config::PasqalConfig;
use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PasqalTaskKind {
    Pulser,
    Cudaq,
}

/// QRMI implementation for Pasqal Cloud
pub struct PasqalCloud {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
    task_kinds: HashMap<String, PasqalTaskKind>,
}

impl PasqalCloud {
    /// Constructs a QRMI to access Pasqal Cloud Service
    ///
    /// # Environment variables
    ///
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID`: Pasqal Cloud Project ID to access the QPU
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN`: Pasqal Cloud Auth Token
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_CLIENT_ID`: Pasqal Cloud service account client ID
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_CLIENT_SECRET`: Pasqal Cloud service account client secret
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT`: Optional auth endpoint URL/path. Default: `authenticate.pasqal.cloud/oauth/token`
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_BASE_URL`: Optional Pasqal Cloud API base URL. Default: `https://apis.pasqal.cloud`
    /// * `PASQAL_CONFIG_ROOT`: Optional root containing `.pasqal/config`
    /// * `<backend_name>_PASQAL_CONFIG_ROOT`: Optional backend-specific root containing `.pasqal/config`
    /// * `PASQAL_USERNAME`: Pasqal Cloud username
    /// * `PASQAL_PASSWORD`: Pasqal Cloud password
    ///
    /// # Config file fallback
    ///
    /// * `~/.pasqal/config`: Optional fallback for `username`, `password`, `client_id`, `client_secret`, `token`, `project_id`, `auth_endpoint`
    pub fn new(backend_name: &str) -> Result<Self> {
        debug!(
            "Initializing PasqalCloud QRMI for backend '{}'",
            backend_name
        );

        let cfg = PasqalConfig::read(backend_name)?;
        let project_id = cfg.project_id(backend_name).unwrap_or_default();
        let auth_token = cfg.auth_token(backend_name);
        let auth_endpoint = cfg.auth_endpoint(backend_name);
        let base_url = cfg.base_url(backend_name);

        // Build Pasqal Cloud API client
        debug!("Build PasqalCloud client for backend '{}'", backend_name);
        let mut builder = ClientBuilder::new(project_id.clone());
        builder.with_auth_endpoint(auth_endpoint.clone());
        if let Some(base_url) = base_url {
            builder.with_base_url(base_url);
        }
        if super::retries_disabled() {
            debug!("HTTP retries disabled for backend '{}'", backend_name);
            builder.with_retry_disabled();
        }

        let (username, password) = cfg.credentials();
        let (client_id, client_secret) = cfg.service_account_credentials(backend_name);
        let has_user_credentials = username.is_some() && password.is_some();
        let has_service_account_credentials = client_id.is_some() && client_secret.is_some();
        let has_auth_token = auth_token.is_some();
        if let (Some(username), Some(password)) = (username, password) {
            builder.with_credentials(username, password);
        }
        if let (Some(client_id), Some(client_secret)) = (client_id, client_secret) {
            builder.with_service_account_credentials(client_id, client_secret);
        }
        if let Some(token) = auth_token {
            builder.with_token(token);
        }
        if project_id.is_empty() {
            debug!(
                "No Pasqal Cloud project_id configured for backend '{}'; unauthenticated operations can still be used",
                backend_name
            );
        }
        if !has_user_credentials && !has_service_account_credentials && !has_auth_token {
            debug!(
                "No Pasqal Cloud auth details configured for backend '{}': expected PASQAL_USERNAME/PASQAL_PASSWORD, ~/.pasqal/config credentials, client_id/client_secret, or token",
                backend_name
            );
        }
        let api_client = builder.build()?;

        Ok(Self {
            api_client,
            backend_name: backend_name.to_string(),
            task_kinds: HashMap::new(),
        })
    }

    fn parse_device_type(&self) -> Result<DeviceType> {
        self.backend_name.parse::<DeviceType>().map_err(|_| {
            anyhow!(
                "Device '{}' is invalid. Valid devices: {}",
                self.backend_name,
                [
                    "FRESNEL",
                    "FRESNEL_CAN1",
                    "EMU_MPS",
                    "EMU_FREE",
                    "EMU_FRESNEL"
                ]
                .join(", ")
            )
        })
    }

    fn is_cudaq_sequence(sequence: &str) -> bool {
        // Checks sequence kind by looking for "setup" and "hamiltonian" fields in the sequence JSON, which are present in CUDA-Q sequences.
        let Ok(value) = serde_json::from_str::<serde_json::Value>(sequence) else {
            return false;
        };
        value.get("setup").is_some() && value.get("hamiltonian").is_some()
    }

    fn map_job_status(status: &JobStatus) -> TaskStatus {
        match status {
            JobStatus::Pending => TaskStatus::Queued,
            JobStatus::Running => TaskStatus::Running,
            JobStatus::Canceling => TaskStatus::Cancelled,
            JobStatus::Done => TaskStatus::Completed,
            JobStatus::Canceled => TaskStatus::Cancelled,
            JobStatus::Error => TaskStatus::Failed,
        }
    }

    fn normalize_cudaq_result(result: &serde_json::Value) -> String {
        let counter = result.get("counter").unwrap_or(result);
        let normalized_counter = match counter.as_object() {
            Some(counter_obj) if counter_obj.values().all(|value| value.is_number()) => {
                counter.clone()
            }
            Some(counter_obj) if counter_obj.len() == 1 => {
                let nested = counter_obj.values().next().unwrap_or(counter);
                nested
                    .get("counter")
                    .filter(|value| value.is_object())
                    .cloned()
                    .or_else(|| nested.as_object().map(|_| nested.clone()))
                    .unwrap_or_else(|| counter.clone())
            }
            _ => counter.clone(),
        };
        serde_json::json!({ "counter": normalized_counter }).to_string()
    }

    async fn task_status_from_job_id(&mut self, job_id: &str) -> Result<TaskStatus> {
        let job = self.api_client.get_job(job_id).await?;
        Ok(Self::map_job_status(&job.data.status))
    }

    async fn task_status_from_batch_id(&mut self, batch_id: &str) -> Result<TaskStatus> {
        let batch = self.api_client.get_batch(batch_id).await?;
        let job_id = batch
            .data
            .job_ids
            .first()
            .ok_or_else(|| anyhow!("No jobs found for batch '{}'", batch_id))?;
        self.task_status_from_job_id(job_id).await
    }

    async fn task_result_from_cudaq(&mut self, task_id: &str) -> Result<TaskResult> {
        let job = self.api_client.get_cudaq_job(task_id).await?;
        Ok(TaskResult {
            value: Self::normalize_cudaq_result(&job.data.result),
        })
    }

    fn task_kind(&self, task_id: &str) -> PasqalTaskKind {
        match self.task_kinds.get(task_id).copied() {
            Some(kind) => kind,
            None => {
                warn!(
                    "Missing task kind for task ID '{}'; defaulting to pulser endpoint",
                    task_id
                );
                PasqalTaskKind::Pulser
            }
        }
    }
}

#[async_trait]
impl QuantumResource for PasqalCloud {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::PasqalCloud)
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        let device_type = self.parse_device_type()?;

        // The device may be down temporarily but jobs can still
        // be submitted and queued through the cloud.
        // Thus we only check that the device is not retired.
        match self.api_client.get_device(device_type).await {
            Ok(device) => Ok(device.availability == "ACTIVE"),
            Err(err) => bail!("Failed to get device: {}", err),
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        // TBD on cloud side for POC
        // Pasqal Cloud does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // TBD on cloud side for POC
        // Pasqal Cloud does not support session concept, so simply ignores
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        debug!(
            "Starting task on PasqalCloud QRMI (backend '{}')",
            self.backend_name
        );
        if let Payload::PasqalCloud { sequence, job_runs } = payload {
            let device_type = self.parse_device_type()?;

            if Self::is_cudaq_sequence(&sequence) {
                let sequence_value: serde_json::Value = match serde_json::from_str(&sequence) {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(anyhow!("Failed to parse CUDA-Q sequence payload: {err}"));
                    }
                };
                match self
                    .api_client
                    .create_cudaq_job(sequence_value, job_runs, device_type)
                    .await
                {
                    Ok(job) => {
                        self.task_kinds
                            .insert(job.data.id.clone(), PasqalTaskKind::Cudaq);
                        Ok(job.data.id)
                    }
                    Err(err) => Err(err),
                }
            } else {
                match self
                    .api_client
                    .create_batch(sequence, job_runs, device_type)
                    .await
                {
                    Ok(batch) => {
                        self.task_kinds
                            .insert(batch.data.id.clone(), PasqalTaskKind::Pulser);
                        Ok(batch.data.id)
                    }
                    Err(err) => Err(err),
                }
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload))
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        debug!(
            "Stopping task '{}' on PasqalCloud QRMI (backend '{}')",
            task_id, self.backend_name
        );
        match self.api_client.cancel_batch(task_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        self.task_status_from_batch_id(task_id).await
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        match self.task_kind(task_id) {
            PasqalTaskKind::Pulser => match self.api_client.get_batch_results(task_id).await {
                Ok(resp) => Ok(TaskResult { value: resp }),
                Err(err) => Err(err),
            },
            PasqalTaskKind::Cudaq => match self.task_result_from_cudaq(task_id).await {
                Ok(result) => Ok(result),
                Err(err) => Err(err),
            },
        }
    }

    async fn task_logs(&mut self, _task_id: &str) -> Result<String> {
        Ok("There are no logs for this job.".to_string())
    }

    async fn target(&mut self) -> Result<Target> {
        debug!(
            "Getting target information for PasqalCloud QRMI (backend '{}')",
            self.backend_name
        );
        let device_type = self.parse_device_type()?;
        match self.api_client.get_device_specs(device_type).await {
            Ok(resp) => Ok(Target { value: resp }),
            Err(err) => Err(err),
        }
    }

    async fn metadata(&mut self) -> std::collections::HashMap<String, String> {
        let mut metadata: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}

#[cfg(test)]
#[path = "tests/cloud.rs"]
mod tests;
