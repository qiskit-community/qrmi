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

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use iqm_server_api::apis::configuration;
use iqm_server_api::apis::jobs_api::{cancel_job_v1, get_job_v1, job_get_artifacts};
use iqm_server_api::apis::quantum_computers_api::get_qc_health_v1;
use iqm_server_api::models::IqmServerJobStatus;
use log::error;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use uuid::Uuid;

/// QRMI implementation for IQM Server API
pub struct IQMServer {
    pub(crate) config: configuration::Configuration,
    pub(crate) backend_name: String,
    pub(crate) acquisition_token: Option<String>,
}

impl IQMServer {
    /// Constructs a IQM Server instance.
    ///
    /// Environment variables used:
    /// * QRMI_IQM_ISA_ENDPOINT - IQM Server API endpoint URL
    /// * QRMI_IQM_ISA_TOKEN - IQM Server API token
    /// * QRMI_JOB_ACQUISITION_TOKEN - (optional) pre‐set session ID
    pub fn new(backend_name: &str) -> Result<Self> {
        let iqm_endpoint =
            env::var(format!("{backend_name}_QRMI_IQM_ISA_ENDPOINT")).map_err(|_| {
                anyhow!("{backend_name}_QRMI_IQM_ISA_ENDPOINT environment variable is not set")
            })?;
        let iqm_token = env::var(format!("{backend_name}_QRMI_IQM_ISA_TOKEN")).map_err(|_| {
            anyhow!("{backend_name}_QRMI_IQM_ISA_TOKEN environment variable is not set")
        })?;
        let acquisition_token = env::var(format!("{backend_name}_QRMI_JOB_ACQUISITION_TOKEN")).ok();
        // Set up the config
        let mut config = configuration::Configuration::new();
        config.base_path = iqm_endpoint;
        config.bearer_access_token = Some(iqm_token);

        let converted = if let Some(pos) = backend_name.rfind('_') {
            let mut s = backend_name.to_string();
            s.replace_range(pos..=pos, ":");
            s
        } else {
            backend_name.to_string()
        };

        Ok(Self {
            config,
            backend_name: converted,
            acquisition_token,
        })
    }
}

// Implement the QuantumResource trait using the asynchronous wrappers.
#[async_trait]
impl QuantumResource for IQMServer {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::IQMServer)
    }

    /// Asynchronously checks if a backend is accessible.
    async fn is_accessible(&mut self) -> Result<bool> {
        match get_qc_health_v1(&self.config, &self.backend_name).await {
            Ok(health) => Ok(health.healthy),
            Err(err) => {
                bail!(format!("Failed to get backend details: {:#?}", &err));
            }
        }
    }

    /// Creates a new session.
    ///
    async fn acquire(&mut self) -> Result<String> {
        Ok(Uuid::new_v4().to_string())
    }

    /// Deletes the current session.
    ///
    /// This sends a DELETE request to /sessions/{session_id}/close via the qiskit_runtime_api client.
    async fn release(&mut self, _acquisition_token: &str) -> Result<()> {
        Ok(())
    }

    /// Starts a job task.
    ///
    /// This function sends a POST request to /jobs. The input payload is parsed as JSON,
    /// and the job is created using the qiskit_runtime_api client function jobs_api::create_job.
    async fn task_start(&mut self, _payload: Payload) -> Result<String> {
        bail!("Function not supported")
    }

    /// Stops a running job.
    ///
    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        match cancel_job_v1(&self.config, task_id).await {
            Ok(_job) => Ok(()),
            Err(err) => {
                bail!(format!("Failed to cancel a job({}): {:#?}", task_id, &err));
            }
        }
    }

    /// Returns the current status of a job.
    ///
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let job = get_job_v1(&self.config, task_id, Some(true), Some(30)).await?;
        match job.status {
            IqmServerJobStatus::Waiting => Ok(TaskStatus::Queued),
            IqmServerJobStatus::Processing => Ok(TaskStatus::Running),
            IqmServerJobStatus::Completed => Ok(TaskStatus::Completed),
            IqmServerJobStatus::Failed => Ok(TaskStatus::Failed),
            IqmServerJobStatus::Cancelled => Ok(TaskStatus::Cancelled),
        }
    }

    /// Retrieves the results of a completed job.
    ///
    /// This function calls GET /jobs/{id}/results and serializes the returned JSON into a string.
    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let mut result = json!({
            "measurements": Value::Null,
            "measurement_counts": Value::Null,
        });

        match job_get_artifacts(&self.config, task_id, "measurements").await {
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(json) => result["measurements"] = json,
                Err(e) => error!("Failed to json-parse measurements: {:?}", e),
            },
            Err(e) => error!("Failed to obtain 'measurements' artifact: {:?}", e),
        }
        match job_get_artifacts(&self.config, task_id, "measurement_counts").await {
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(json) => result["measurement_counts"] = json,
                Err(e) => error!("Failed to json-parse measurement_counts: {:?}", e),
            },
            Err(e) => error!("Failed to obtain 'measurement_counts' artifact: {:?}", e),
        }

        match serde_json::to_string_pretty(&result) {
            Ok(result_str) => Ok(TaskResult { value: result_str }),
            Err(e) => bail!("Failed to serialize result: {:?}", e),
        }
    }

    /// Returns the log messages of the task.
    ///
    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let job = get_job_v1(&self.config, task_id, Some(true), Some(30)).await?;
        let mut log = String::new();
        writeln!(log, "Timeline   :").unwrap();
        for event in &job.timeline {
            writeln!(
                log,
                "  {} [{:<24}] {}",
                event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                event.source,
                event.status,
            )
            .unwrap();
        }
        if job.messages.is_empty() {
            writeln!(log, "Messages   : (none)").unwrap();
        } else {
            writeln!(log, "Messages   :").unwrap();
            for msg in &job.messages {
                writeln!(log, "  [{:<24}] {}", msg.source, msg.message).unwrap();
            }
        }
        Ok(log)
    }

    /// Retrieves target details.
    ///
    /// This function combines the results of GET /backends/{id}/configuration and
    /// GET /backends/{id}/properties into a single JSON object.
    async fn target(&mut self) -> Result<Target> {
        Ok(Target {
            value: "".to_string(),
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        if let Some(ref acquisition_token) = self.acquisition_token {
            metadata.insert(
                "acquisition_token".to_string(),
                acquisition_token.to_string(),
            );
        }
        metadata
    }
}

#[cfg(test)]
#[path = "tests/iqm_server.rs"]
mod tests;
