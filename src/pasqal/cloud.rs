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
use log::{debug, error, warn};
use pasqal_cloud_api::{Client, ClientBuilder, DeviceType, JobStatus};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use async_trait::async_trait;

#[derive(Debug, Clone, Default)]
struct PasqalConfig {
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    project_id: Option<String>,
    auth_endpoint: Option<String>,
}

const DEFAULT_PASQAL_CLOUD_AUTH_ENDPOINT: &str = "authenticate.pasqal.cloud/oauth/token";

fn strip_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn read_pasqal_config(_backend_name: &str) -> Result<PasqalConfig> {
    let mut config_path_candidates: Vec<PathBuf> = Vec::new();
    let mut pasqal_config_root_path: Option<PathBuf> = None;

    if let Ok(config_root) = env::var("PASQAL_CONFIG_ROOT") {
        if !config_root.trim().is_empty() {
            let mut path = PathBuf::from(config_root);
            path.push(".pasqal");
            path.push("config");
            pasqal_config_root_path = Some(path.clone());
            config_path_candidates.push(path);
        }
    }

    if let Ok(home) = env::var("HOME") {
        if !home.trim().is_empty() {
            let mut path = PathBuf::from(home);
            path.push(".pasqal");
            path.push("config");
            config_path_candidates.push(path);
        }
    }

    let content = match config_path_candidates
        .iter()
        .find_map(|path| fs::read_to_string(path).ok())
    {
        Some(content) => content,
        None => {
            if let Some(path) = pasqal_config_root_path {
                warn!(
                    "PASQAL_CONFIG_ROOT is set but config file was not found: {}",
                    path.display()
                );
            }
            return Ok(PasqalConfig::default());
        }
    };

    let mut config = PasqalConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let (k, v) = match line.split_once('=') {
            Some((k, v)) => (k.trim(), strip_quotes(v).trim()),
            None => continue,
        };
        if k.is_empty() {
            continue;
        }

        match k.to_ascii_lowercase().as_str() {
            "username" => config.username = Some(v.to_string()),
            "password" => config.password = Some(v.to_string()),
            "token" => config.token = Some(v.to_string()),
            "project_id" => config.project_id = Some(v.to_string()),
            "auth_endpoint" => config.auth_endpoint = Some(v.to_string()),
            _ => {}
        }
    }

    Ok(config)
}

fn read_qrmi_config_env_value(backend_name: &str, key: &str) -> Option<String> {
    let content = fs::read_to_string("/etc/slurm/qrmi_config.json").ok()?;
    read_qrmi_config_env_value_from_content(&content, backend_name, key)
}

fn read_qrmi_config_env_value_from_content(
    content: &str,
    backend_name: &str,
    key: &str,
) -> Option<String> {
    let root: serde_json::Value = serde_json::from_str(content).ok()?;
    let resources = root.get("resources")?.as_array()?;

    for r in resources {
        // TODO: we can make this let Some(name) = ... to be more robust I think?
        let name = r.get("name")?.as_str()?;
        if name != backend_name {
            continue;
        }
        let env = r.get("environment")?.as_object()?;
        let v = env.get(key)?.as_str()?.trim();
        if v.is_empty() {
            return None;
        }
        return Some(v.to_string());
    }
    None
}

fn resolve_pasqal_credentials(cfg: &PasqalConfig) -> (Option<String>, Option<String>) {
    // Get credentials from environment variables or config, with preference to environment variables.
    let username = env::var("PASQAL_USERNAME")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or(cfg.username.clone().filter(|v| !v.trim().is_empty()));
    let password = env::var("PASQAL_PASSWORD")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or(cfg.password.clone().filter(|v| !v.trim().is_empty()));
    (username, password)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PasqalTaskKind {
    Batch,
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
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT`: Optional auth endpoint URL/path. Default: `authenticate.pasqal.cloud/oauth/token`
    /// * `PASQAL_USERNAME`: Pasqal Cloud username
    /// * `PASQAL_PASSWORD`: Pasqal Cloud password
    ///
    /// # Config file fallback
    ///
    /// * `~/.pasqal/config`: Optional fallback for `username`, `password`, `token`, `project_id`, `auth_endpoint`
    pub fn new(backend_name: &str) -> Result<Self> {
        debug!(
            "Initializing PasqalCloud QRMI for backend '{}'",
            backend_name
        );

        let cfg = read_pasqal_config(backend_name)?;

        // Project ID
        let project_id_var = format!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID");
        let env_project_id = env::var(&project_id_var)
            .ok()
            .filter(|v| !v.trim().is_empty());
        // Preference order: explicit env var > user ~/.pasqal/config > cluster wide qrmi_config.json provides default.
        let project_id = env_project_id
            .or(cfg.project_id.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(backend_name, "QRMI_PASQAL_CLOUD_PROJECT_ID"))
            .ok_or_else(|| {
                anyhow!(
                    "{project_id_var} is not set and no project_id was found in ~/.pasqal/config or /etc/slurm/qrmi_config.json"
                )
            })?;

        // Auth token
        let var_name = format!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_TOKEN");
        let env_token = env::var(&var_name).ok().filter(|v| !v.trim().is_empty());
        let auth_token = env_token // Token or None
            .or(cfg.token.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_AUTH_TOKEN",
            ));

        // Auth endpoint
        let auth_endpoint_var = format!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT");
        let env_auth_endpoint = env::var(&auth_endpoint_var)
            .ok()
            .filter(|v| !v.trim().is_empty());
        let auth_endpoint = env_auth_endpoint
            .or(cfg.auth_endpoint.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_AUTH_ENDPOINT",
            ))
            .unwrap_or_else(|| DEFAULT_PASQAL_CLOUD_AUTH_ENDPOINT.to_string());

        // Build Pasqal Cloud API client
        debug!("Build PasqalCloud client for backend '{}'", backend_name);
        let mut builder = ClientBuilder::new(project_id.clone());
        builder.with_auth_endpoint(auth_endpoint.clone());

        // Preference order for credentials: explicit username/password in env > config > direct token.
        let (username, password) = resolve_pasqal_credentials(&cfg);
        if let (Some(username), Some(password)) = (username, password) {
            builder.with_credentials(username, password);
        } else if let Some(token) = auth_token {
            builder.with_token(token);
        } else {
            error!(
                "No Pasqal Cloud auth details configured for backend '{}': expected PASQAL_USERNAME/PASQAL_PASSWORD, ~/.pasqal/config credentials, or token",
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

    fn map_batch_status(status: &JobStatus) -> TaskStatus {
        match status {
            JobStatus::Pending => TaskStatus::Queued,
            JobStatus::Running => TaskStatus::Running,
            JobStatus::Canceling => TaskStatus::Cancelled,
            JobStatus::Done => TaskStatus::Completed,
            JobStatus::Canceled => TaskStatus::Cancelled,
            JobStatus::Error => TaskStatus::Failed,
        }
    }

    // todo, to be removed. this is because for cuda-q we use the
    // cuda-q specific get job instead of having the same flow as for
    // normal jobs. This should be fixed partially server side, and here
    // fixing it in cuda-q will be slightly annoying.
    fn map_status(status: &str) -> Result<TaskStatus> {
        let parsed: JobStatus = serde_json::from_str(&format!("\"{status}\""))
            .map_err(|err| anyhow!("Unexpected job status '{status}': {err}"))?;
        Ok(Self::map_batch_status(&parsed))
    }

    fn task_kind(&self, task_id: &str) -> Result<PasqalTaskKind> {
        match self.task_kinds.get(task_id) {
            Some(task_kind) => Ok(*task_kind),
            None => Err(anyhow!("Missing task kind for task ID: {}", task_id)),
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
        // be submitted and queued through the cloud
        // Thus we only check that the device is not retired
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
                            .insert(batch.data.id.clone(), PasqalTaskKind::Batch);
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

    // TODO: We should not have special cudaq path here. not needed?
    // right now I'm a bit confused about what task_id is though. Is it batch_id or job_id?
    // Maybe we could also switch QRMI in general onto job_id instead of batch_id to avoid this? TBD.
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        match self.task_kind(task_id)? {
            PasqalTaskKind::Batch => match self.api_client.get_batch(task_id).await {
                Ok(batch) => match self.api_client.get_job(&batch.data.job_ids[0]).await {
                    Ok(job) => Ok(Self::map_batch_status(&job.data.status)),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            },
            PasqalTaskKind::Cudaq => match self.api_client.get_cudaq_job(task_id).await {
                Ok(job) => match Self::map_status(&job.data.status) {
                    Ok(status) => Ok(status),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            },
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        match self.api_client.get_batch_results(task_id).await {
            Ok(resp) => Ok(TaskResult { value: resp }),
            Err(_err) => Err(_err),
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
            Ok(resp) => Ok(Target {
                value: resp.data.specs,
            }),
            Err(err) => Err(err),
        }
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}

#[cfg(test)]
#[path = "tests/cloud.rs"]
mod tests;
