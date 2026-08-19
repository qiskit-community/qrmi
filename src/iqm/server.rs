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

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use iqm_server_api::apis::calibration_sets_api::{
    get_calibration_set_v1, get_dynamic_quantum_architecture_v1, get_quality_metrics_v1,
};
use iqm_server_api::apis::configuration;
use iqm_server_api::apis::jobs_api::{cancel_job_v1, get_job_v1, job_get_artifacts, job_submit};
use iqm_server_api::apis::quantum_computers_api::{get_qc_health_v1, qc_get_artifacts};
use iqm_server_api::models::IqmServerJobStatus;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use uuid::Uuid;

/// QRMI implementation for IQM Server API
pub struct IQMServer {
    pub(crate) config: configuration::Configuration,
    pub(crate) backend_name: String,
    pub(crate) calibration_set_id: String,
    pub(crate) acquisition_token: Option<String>,
}

impl IQMServer {
    /// Constructs a IQM Server instance.
    ///
    /// Environment variables used:
    /// * QRMI_IQM_ISA_ENDPOINT - IQM Server API endpoint URL
    /// * QRMI_IQM_ISA_TOKEN - IQM Server API token
    /// * QRMI_JOB_ACQUISITION_TOKEN - (optional) pre‐set session ID
    pub fn new(resource_id: &str) -> Result<Self> {
        let buf: Vec<&str> = resource_id.split(",").collect();
        let (backend_name, calset_id) = match buf.as_slice() {
            [name, id, ..] => (*name, *id),
            [name] => (*name, "default"),
            _ => unreachable!("buf should never be empty due to split()"),
        };

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
            calibration_set_id: calset_id.to_string(),
        })
    }

    /// Interprets the result of an artifact fetch -- `job_get_artifacts` or
    /// `qc_get_artifacts` -- as JSON.
    ///
    /// Returns `Value::Null` only when the provider reports a 404 for this
    /// specific artifact. Both endpoints document this as normal: which
    /// artifacts exist depends on job type (`job_get_artifacts`) or the
    /// quantum computer's Station Control version (`qc_get_artifacts`).
    /// Any other failure -- network, auth, a non-404 error status, or a
    /// response that isn't valid JSON -- is returned as `Err` rather than
    /// being folded into the same `null`.
    fn parse_optional_artifact<B, E>(
        result: std::result::Result<B, iqm_server_api::apis::Error<E>>,
        artifact_type: &str,
    ) -> Result<Value>
    where
        B: AsRef<[u8]>,
        E: std::fmt::Debug + Send + Sync + 'static,
    {
        match result {
            Ok(bytes) => serde_json::from_slice::<Value>(bytes.as_ref())
                .with_context(|| format!("'{artifact_type}' artifact is not valid JSON")),
            Err(iqm_server_api::apis::Error::ResponseError(resp))
                if resp.status.as_u16() == 404 =>
            {
                Ok(Value::Null)
            }
            Err(e) => Err(e).with_context(|| format!("Failed to fetch '{artifact_type}'")),
        }
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

    /// IQM Server has no session concept. This does not contact the
    /// provider; it returns a generated id so callers written against the
    /// trait do not need a special case for this backend.
    async fn acquire(&mut self) -> Result<String> {
        Ok(Uuid::new_v4().to_string())
    }

    /// IQM Server has no session concept, so this is a no-op: nothing is
    /// contacted and nothing is released. See `acquire()`.
    async fn release(&mut self, _acquisition_token: &str) -> Result<()> {
        Ok(())
    }

    /// Starts a job task.
    ///
    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        if let Payload::IQMServer {
            iqmjson,
            job_type,
            use_timeslot,
            tag,
        } = payload
        {
            let job: serde_json::Value = serde_json::from_str(iqmjson.as_str())?;
            match job_submit(
                &self.config,
                &self.backend_name,
                &job_type,
                use_timeslot,
                tag.as_deref(),
                Some(job),
            )
            .await
            {
                Ok(val) => Ok(val.id.to_string()),
                Err(err) => {
                    bail!("An error occurred during starting a task: {:#?}", err);
                }
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload));
        }
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
    ///
    /// Which artifacts exist depends on the job type (see
    /// `job_get_artifacts`'s own documentation), so a 404 for
    /// `measurements` or `measurement_counts` is normal and is represented
    /// as `null` for that field. Any other failure -- network, auth, a
    /// non-404 error status, or a response that isn't valid JSON --
    /// propagates as `Err` instead of being silently swallowed into the
    /// same `null`.
    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let measurements = Self::parse_optional_artifact(
            job_get_artifacts(&self.config, task_id, "measurements").await,
            "measurements",
        )
        .context("Failed to get 'measurements' artifact")?;
        let measurement_counts = Self::parse_optional_artifact(
            job_get_artifacts(&self.config, task_id, "measurement_counts").await,
            "measurement_counts",
        )
        .context("Failed to get 'measurement_counts' artifact")?;

        let result = json!({
            "measurements": measurements,
            "measurement_counts": measurement_counts,
        });

        let result_str =
            serde_json::to_string_pretty(&result).context("Failed to serialize result")?;
        Ok(TaskResult { value: result_str })
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
    /// GET /backends/{id}/properties, plus the QC-level (not calibration-set-bound)
    /// `static-quantum-architectures` artifact, into a single JSON object.
    ///
    /// `dynamic_quantum_architecture`, `calibration_set`, and
    /// `quality_metrics` are expected to always exist, so if any of those
    /// three underlying REST calls fails, or its response is not valid
    /// JSON, this returns `Err` rather than a document with that field
    /// silently replaced by `null`. `static_quantum_architecture` is
    /// different: `qc_get_artifacts`'s own documentation says available
    /// artifacts depend on the quantum computer's Station Control version,
    /// so a 404 for it specifically is normal and represented as `null`;
    /// any other failure for it still propagates as `Err` the same way.
    async fn target(&mut self) -> Result<Target> {
        let dynamic_quantum_architecture = get_dynamic_quantum_architecture_v1(
            &self.config,
            &self.backend_name,
            &self.calibration_set_id,
        )
        .await
        .context("Failed to get dynamic_quantum_architecture")?;
        let dynamic_quantum_architecture: serde_json::Value =
            serde_json::from_slice(&dynamic_quantum_architecture)
                .context("Failed to parse dynamic_quantum_architecture")?;

        let calibration_set =
            get_calibration_set_v1(&self.config, &self.backend_name, &self.calibration_set_id)
                .await
                .context("Failed to get calibration_set")?;
        let calibration_set: serde_json::Value =
            serde_json::from_slice(&calibration_set).context("Failed to parse calibration_set")?;

        let quality_metrics =
            get_quality_metrics_v1(&self.config, &self.backend_name, &self.calibration_set_id)
                .await
                .context("Failed to get quality_metrics")?;
        let quality_metrics: serde_json::Value =
            serde_json::from_slice(&quality_metrics).context("Failed to parse quality_metrics")?;

        // Static, calibration-independent topology. Unlike the three
        // fields above, its absence (404) is expected on some Station
        // Control versions -- see the doc comment above.
        let static_quantum_architecture = Self::parse_optional_artifact(
            qc_get_artifacts(
                &self.config,
                &self.backend_name,
                "static-quantum-architectures",
            )
            .await,
            "static_quantum_architecture",
        )
        .context("Failed to get static_quantum_architecture")?;

        let resp = json!({
            "dynamic_quantum_architecture": dynamic_quantum_architecture,
            "calibration_set": calibration_set,
            "quality_metrics": quality_metrics,
            "static_quantum_architecture": static_quantum_architecture,
        });

        Ok(Target {
            value: resp.to_string(),
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
