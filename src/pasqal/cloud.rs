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
use log::debug;
use pasqal_cloud_api::{BatchStatus, Client, ClientBuilder, DeviceType, DEFAULT_AUTH_ENDPOINT};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
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

fn now_unix_seconds() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| anyhow!("Failed to read system time: {e}"))?
        .as_secs() as i64)
}

fn read_pasqal_config(_backend_name: &str) -> Result<PasqalConfig> {
    let home = match env::var("HOME") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => return Ok(PasqalConfig::default()),
    };

    let mut path = PathBuf::from(home);
    path.push(".pasqal");
    path.push("config");

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Ok(PasqalConfig::default()),
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
    let root: serde_json::Value = serde_json::from_str(&content).ok()?;
    let resources = root.get("resources")?.as_array()?;

    for r in resources {
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
    auth_token: String,
    auth_token_expiry_unix_seconds: Option<i64>,
    project_id: String,
    auth_endpoint: String,
    username: Option<String>,
    password: Option<String>,
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
        let auth_token = env_token
            .or(cfg.token.clone().filter(|v| !v.trim().is_empty()))
            .or(read_qrmi_config_env_value(
                backend_name,
                "QRMI_PASQAL_CLOUD_AUTH_TOKEN",
            ))
            .unwrap_or_else(|| String::new());
        let auth_token_expiry_unix_seconds = Client::jwt_expiry_unix_seconds(&auth_token)?;

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

        let auth_token_state = if auth_token.is_empty() {
            "empty"
        } else {
            "set"
        };
        debug!(
            "Build client using project_id='{}', auth_token={}, auth_endpoint='{}'",
            project_id, auth_token_state, auth_endpoint
        );
        let api_client = ClientBuilder::new(auth_token.clone(), project_id.clone()).build()?;

        Ok(Self {
            api_client,
            backend_name: backend_name.to_string(),
            auth_token,
            auth_token_expiry_unix_seconds,
            project_id,
            auth_endpoint,
            username: cfg.username,
            password: cfg.password,
        })
    }

    async fn ensure_authenticated(&mut self) -> Result<()> {
        let now = now_unix_seconds()?;
        if Client::is_auth_token_usable(&self.auth_token, now) {
            return Ok(());
        }
        if let Some(exp) = self.auth_token_expiry_unix_seconds {
            debug!(
                "Auth token is expired (expired at {}, now is {}), will attempt to refresh",
                exp, now
            );
        }
        let (Some(username), Some(password)) = (self.username.as_deref(), self.password.as_deref())
        else {
            return Ok(());
        };

        debug!(
            "Requesting new auth token for PasqalCloud QRMI (backend '{}')",
            self.backend_name
        );
        let token = Client::request_access_token(&self.auth_endpoint, username, password).await?;
        self.auth_token = token;
        self.auth_token_expiry_unix_seconds = Client::jwt_expiry_unix_seconds(&self.auth_token)?;
        self.api_client =
            ClientBuilder::new(self.auth_token.clone(), self.project_id.clone()).build()?;
        Ok(())
    }
}

#[async_trait]
impl QuantumResource for PasqalCloud {
    async fn is_accessible(&mut self) -> Result<bool> {
        self.ensure_authenticated().await?;
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
        self.ensure_authenticated().await?;
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
        self.ensure_authenticated().await?;
        match self.api_client.cancel_batch(task_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        self.ensure_authenticated().await?;
        match self.api_client.get_batch(task_id).await {
            Ok(batch) => {
                let status = match batch.data.status {
                    BatchStatus::Pending => TaskStatus::Queued,
                    BatchStatus::Running => TaskStatus::Running,
                    BatchStatus::Done => TaskStatus::Completed,
                    BatchStatus::Canceled => TaskStatus::Cancelled,
                    BatchStatus::TimedOut => TaskStatus::Failed,
                    BatchStatus::Error => TaskStatus::Failed,
                    BatchStatus::Paused => TaskStatus::Queued,
                };
                return Ok(status);
            }
            Err(err) => Err(err),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        self.ensure_authenticated().await?;
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
        self.ensure_authenticated().await?;
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
        // `metadata()` returns a plain HashMap (not Result), so auth errors cannot be propagated
        // with `?` here. Long-term fix: change the trait/API to return
        // `Result<HashMap<String, String>>` and bubble this error to callers.
        if let Err(err) = self.ensure_authenticated().await {
            debug!(
                "Failed to ensure authentication while fetching metadata for '{}': {}",
                self.backend_name, err
            );
        }
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}

#[cfg(test)]
#[path = "tests/cloud.rs"]
mod tests;
