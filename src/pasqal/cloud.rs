// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025
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
use anyhow::{anyhow, bail, Result};
use log::{debug, error, warn};
use pasqal_cloud_api::{Client, ClientBuilder, DeviceType, JobStatus, DEFAULT_AUTH_ENDPOINT};
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
    let root: serde_json::Value = serde_json::from_str(&content).ok()?;
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

/// QRMI implementation for Pasqal Cloud
pub struct PasqalCloud {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

impl PasqalCloud {
    /// Constructs a QRMI to access Pasqal Cloud Service
    ///
    /// # Environment variables
    ///
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID`: Pasqal Cloud Project ID to access the QPU
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN`: Pasqal Cloud Auth Token
    ///
    /// Let's hardcode the rest for now
    pub fn new(backend_name: &str) -> Result<Self> {
        debug!(
            "Initializing PasqalCloud QRMI for backend '{}'",
            backend_name
        );
        let cfg = read_pasqal_config(backend_name)?;

        let project_id_var = format!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID");
        let env_project_id = env::var(&project_id_var)
            .ok()
            .filter(|v| !v.trim().is_empty());
        let project_id = env_project_id
            .or(cfg.project_id.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(backend_name, "QRMI_PASQAL_CLOUD_PROJECT_ID"))
            .ok_or_else(|| {
                anyhow!(
                    "{project_id_var} is not set and no project_id was found in ~/.pasqal/config or /etc/slurm/qrmi_config.json"
                )
            })?;

        let var_name = format!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_TOKEN");
        let env_token = env::var(&var_name).ok().filter(|v| !v.trim().is_empty());
        let auth_token = env_token // Token or None
            .or(cfg.token.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_AUTH_TOKEN",
            ));

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
            .unwrap_or_else(|| DEFAULT_AUTH_ENDPOINT.to_string());

        debug!("Build PasqalCloud client for backend '{}'", backend_name);
        let mut builder = ClientBuilder::for_project(project_id.clone());
        builder.with_auth_endpoint(auth_endpoint.clone());
        if let (Some(username), Some(password)) = (cfg.username.clone(), cfg.password.clone()) {
            builder.with_credentials(username, password);
        } else if let Some(token) = auth_token {
            builder.with_token(token);
        } else {
            error!(
                "No Pasqal Cloud auth details configured for backend '{}': expected username/password or token",
                backend_name
            );
        }
        let api_client = builder.build()?;

        Ok(Self {
            api_client,
            backend_name: backend_name.to_string(),
        })
    }
}

#[async_trait]
impl QuantumResource for PasqalCloud {
    async fn is_accessible(&mut self) -> Result<bool> {
        let device_type = match self.backend_name.parse::<DeviceType>() {
            Ok(dt) => dt,
            Err(_) => {
                let valid_devices = vec![
                    "FRESNEL",
                    "FRESNEL_CAN1",
                    "EMU_MPS",
                    "EMU_FREE",
                    "EMU_FRESNEL",
                ];
                let err = format!(
                    "Device '{}' is invalid. Valid devices: {}",
                    self.backend_name,
                    valid_devices.join(", ")
                );
                bail!(err);
            }
        };

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
            let device_type = match self.backend_name.parse::<DeviceType>() {
                Ok(dt) => dt,
                Err(_) => {
                    let valid_devices = vec![
                        "FRESNEL",
                        "FRESNEL_CAN1",
                        "EMU_MPS",
                        "EMU_FREE",
                        "EMU_FRESNEL",
                    ];
                    let err = format!(
                        "Device '{}' is invalid. Valid devices: {}",
                        self.backend_name,
                        valid_devices.join(", ")
                    );
                    bail!(err);
                }
            };

            match self
                .api_client
                .create_batch(sequence, job_runs, device_type)
                .await
            {
                Ok(batch) => Ok(batch.data.id),
                Err(err) => Err(err),
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
        match self.api_client.get_batch(task_id).await {
            // Assuming a single job per batch for now,
            // Will need to be updated if multiple jobs per batch are supported in the future
            Ok(batch) => match self.api_client.get_job(&batch.data.job_ids[0]).await {
                Ok(job) => {
                    let status = match &job.data.status {
                        JobStatus::Pending => TaskStatus::Queued,
                        JobStatus::Running => TaskStatus::Running,
                        JobStatus::Canceling => TaskStatus::Cancelled,
                        JobStatus::Done => TaskStatus::Completed,
                        JobStatus::Canceled => TaskStatus::Cancelled,
                        JobStatus::TimedOut => TaskStatus::Failed,
                        JobStatus::Error => TaskStatus::Failed,
                        JobStatus::Paused => TaskStatus::Queued,
                    };
                    Ok(status)
                }
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
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
        let device_type = match self.backend_name.parse::<DeviceType>() {
            Ok(dt) => dt,
            Err(_) => {
                let valid_devices = vec![
                    "FRESNEL",
                    "FRESNEL_CAN1",
                    "EMU_MPS",
                    "EMU_FREE",
                    "EMU_FRESNEL",
                ];
                let err = format!(
                    "Device '{}' is invalid. Valid devices: {}",
                    self.backend_name,
                    valid_devices.join(", ")
                );
                panic!("{}", err);
            }
        };

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
