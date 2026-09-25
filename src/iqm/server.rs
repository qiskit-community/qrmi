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
use crate::iqm::error::{classify, ResourceKind};
use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::{QuantumResource, Result};
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
use std::fmt::Write;

/// QRMI implementation for IQM Server API
pub struct IQMServer {
    pub(crate) config: configuration::Configuration,
    pub(crate) backend_name: String,
    pub(crate) calibration_set_id: String,
    pub(crate) acquisition_token: Option<String>,
}

impl IQMServer {
    /// Splits a `<backend_name>` or `<backend_name>,<calibration_set_id>`
    /// string into its parts, defaulting the calibration set id to
    /// `"default"` when omitted.
    fn parse_backend_and_calset(resource_id: &str) -> (&str, &str) {
        let buf: Vec<&str> = resource_id.split(",").collect();
        match buf.as_slice() {
            [name, id, ..] => (name, id),
            [name] => (name, "default"),
            _ => unreachable!("buf should never be empty due to split()"),
        }
    }

    /// Constructs a IQM Server instance.
    ///
    /// Environment variables used:
    /// * QRMI_IQM_ISA_ENDPOINT - IQM Server API endpoint URL
    /// * QRMI_IQM_ISA_TOKEN - IQM Server API token
    /// * QRMI_JOB_ACQUISITION_TOKEN - (optional) pre‐set session ID
    pub fn new(resource_id: &str) -> Result<Self> {
        Self::from_opt(resource_id, None)
    }

    /// Constructs a IQM Server instance from a config map, instead of
    /// environment variables.
    ///
    /// Takes the same `resource_id` and keys as [`Self::new`]'s
    /// environment variables, minus the `<backend_name>_` prefix. Each key
    /// also accepts its fully lowercased form (e.g.
    /// `qrmi_iqm_isa_endpoint`) as a fallback if the exact-case key isn't
    /// present in the map.
    pub fn from_config(resource_id: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_opt(resource_id, Some(&config))
    }

    /// Shared parsing and client-building logic for [`Self::new`]
    /// (`config: None`, reads OS environment variables) and
    /// [`Self::from_config`] (`config: Some`, reads the given map).
    fn from_opt(resource_id: &str, config: Option<&HashMap<String, String>>) -> Result<Self> {
        let (backend_name, calset_id) = Self::parse_backend_and_calset(resource_id);

        // Config keys are the same name as the env vars, minus the
        // `<backend_name>_` prefix (config maps are already scoped to one
        // backend, so there's nothing to prefix).
        let prefix = if config.is_some() {
            String::new()
        } else {
            format!("{backend_name}_")
        };
        let iqm_endpoint = resolve_opt_required(&format!("{prefix}QRMI_IQM_ISA_ENDPOINT"), config)?;
        let iqm_token = resolve_opt_required(&format!("{prefix}QRMI_IQM_ISA_TOKEN"), config)?;
        let acquisition_token = resolve_opt(&format!("{prefix}QRMI_JOB_ACQUISITION_TOKEN"), config);

        let mut client_config = configuration::Configuration::new();
        client_config.base_path = iqm_endpoint;
        client_config.bearer_access_token = Some(iqm_token);

        let converted = if let Some(pos) = backend_name.rfind('_') {
            let mut s = backend_name.to_string();
            s.replace_range(pos..=pos, ":");
            s
        } else {
            backend_name.to_string()
        };

        Ok(Self {
            config: client_config,
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
        resource_kind: ResourceKind,
    ) -> Result<Value>
    where
        B: AsRef<[u8]>,
        E: std::fmt::Debug + Send + Sync + 'static,
    {
        match result {
            Ok(bytes) => Ok(serde_json::from_slice::<Value>(bytes.as_ref())?),
            Err(iqm_server_api::apis::Error::ResponseError(resp))
                if resp.status.as_u16() == 404 =>
            {
                Ok(Value::Null)
            }
            Err(e) => Err(classify(e, resource_kind)),
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
        let health = get_qc_health_v1(&self.config, &self.backend_name)
            .await
            .map_err(|e| classify(e, ResourceKind::Backend))?;
        Ok(health.operational == "online" && health.health.healthy)
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
            let job = job_submit(
                &self.config,
                &self.backend_name,
                &job_type,
                use_timeslot,
                tag.as_deref(),
                Some(job),
            )
            .await
            .map_err(|e| classify(e, ResourceKind::Backend))?;
            Ok(job.id.to_string())
        } else {
            Err(QrmiError::UnsupportedPayload(format!("{payload:?}")))
        }
    }

    /// Stops a running job.
    ///
    /// Fetches the job's current status first, and only actually asks the
    /// server to cancel it if that status is `Waiting` or `Processing` --
    /// a job already in a terminal state (`Completed`/`Failed`/`Cancelled`)
    /// can't be cancelled again, and the server rejects that with 403
    /// (`IllegalJobStatus`, see `crate::iqm::error`'s docs on why that
    /// status isn't otherwise classified). Mirrors
    /// `crate::ibm::quantum_compute_service::QuantumComputeService::task_stop`'s
    /// same guard for the same reason.
    ///
    /// The cancel call's own result is deliberately discarded, same as
    /// that implementation: even after checking, the job could still
    /// finish on its own in the moment between the status check and this
    /// call, and that race isn't an error worth surfacing to the caller.
    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let job = get_job_v1(&self.config, task_id, Some(true), Some(30))
            .await
            .map_err(|e| classify(e, ResourceKind::Job))?;
        if matches!(
            job.status,
            IqmServerJobStatus::Waiting | IqmServerJobStatus::Processing
        ) {
            cancel_job_v1(&self.config, task_id)
                .await
                .map_err(|e| classify(e, ResourceKind::Job))?;
        }
        Ok(())
    }

    /// Returns the current status of a job.
    ///
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let job = get_job_v1(&self.config, task_id, Some(true), Some(30))
            .await
            .map_err(|e| classify(e, ResourceKind::Job))?;
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
            ResourceKind::Job,
        )?;
        let measurement_counts = Self::parse_optional_artifact(
            job_get_artifacts(&self.config, task_id, "measurement_counts").await,
            ResourceKind::Job,
        )?;

        let result = json!({
            "measurements": measurements,
            "measurement_counts": measurement_counts,
        });

        let result_str = serde_json::to_string_pretty(&result)?;
        Ok(TaskResult { value: result_str })
    }

    /// Returns the log messages of the task.
    ///
    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let job = get_job_v1(&self.config, task_id, Some(true), Some(30))
            .await
            .map_err(|e| classify(e, ResourceKind::Job))?;
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
        .map_err(|e| classify(e, ResourceKind::Backend))?;
        let dynamic_quantum_architecture: serde_json::Value =
            serde_json::from_slice(&dynamic_quantum_architecture)?;

        let calibration_set =
            get_calibration_set_v1(&self.config, &self.backend_name, &self.calibration_set_id)
                .await
                .map_err(|e| classify(e, ResourceKind::Backend))?;
        let calibration_set: serde_json::Value = serde_json::from_slice(&calibration_set)?;

        let quality_metrics =
            get_quality_metrics_v1(&self.config, &self.backend_name, &self.calibration_set_id)
                .await
                .map_err(|e| classify(e, ResourceKind::Backend))?;
        let quality_metrics: serde_json::Value = serde_json::from_slice(&quality_metrics)?;

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
            ResourceKind::Backend,
        )?;

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
