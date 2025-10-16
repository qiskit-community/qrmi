// This code is part of Qiskit.
//
// Copyright (C): 2025 UKRI-STFC (Hartree Centre)
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

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

use crate::ibm::qiskit_runtime_service::models::{
    CreateJobRequestOneOfAllOfParams, EstimatorV2Input, NoiseLearnerInput, SamplerV2Input,
};
use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{bail, Result};
use log::error;
use qiskit_runtime_client::apis::{auth, backends_api, configuration, jobs_api, sessions_api};
use qiskit_runtime_client::models;
use qiskit_runtime_client::models::create_session_request_one_of::Mode;
use qiskit_runtime_client::models::create_job_request_one_of::LogLevel;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

use async_trait::async_trait;

/// QRMI implementation for IBM Qiskit Runtime Service.
pub struct IBMQiskitRuntimeService {
    pub(crate) config: configuration::Configuration,
    pub(crate) backend_name: String,
    pub(crate) session_id: Option<String>,
    pub(crate) calibration_id: Option<String>,
    pub(crate) timeout_secs: Option<i32>,
    pub(crate) session_mode: String,
    pub(crate) session_max_ttl: i32,
    pub(crate) api_key: String,
    pub(crate) iam_endpoint: String,
    pub(crate) token_expiration: u64,
    pub(crate) token_lifetime: u64,
}

impl IBMQiskitRuntimeService {
    /// Constructs a QRS service instance.
    ///
    /// Environment variables used:
    /// * QRMI_IBM_QRS_ENDPOINT - QRS endpoint URL
    /// * QRMI_IBM_QRS_IAM_ENDPOINT - IAM endpoint URL
    /// * QRMI_IBM_QRS_IAM_APIKEY - IAM API key for QRS
    /// * QRMI_IBM_QRS_SERVICE_CRN - QRS service instance CRN
    /// * QRMI_IBM_QRS_SESSION_MODE - Session mode (default: dedicated)
    /// * QRMI_IBM_QRS_SESSION_MAX_TTL - Session max_ttl (default: 28800)
    /// * QRMI_IBM_QRS_TIMEOUT_SECONDS or QRMI_JOB_TIMEOUT_SECONDS - (optional) Cost for the job (seconds)
    /// * QRMI_IBM_QRS_SESSION_ID or QRMI_JOB_ACQUISITION_TOKEN - (optional) pre‐set session ID
    pub fn new(backend_name: &str) -> Self {
        let qrs_endpoint = env::var(format!("{backend_name}_QRMI_IBM_QRS_ENDPOINT"))
            .unwrap_or_else(|_| {
                panic!("{backend_name}_QRMI_IBM_QRS_ENDPOINT environment variable is not set")
            });
        let iam_endpoint = env::var(format!("{backend_name}_QRMI_IBM_QRS_IAM_ENDPOINT"))
            .unwrap_or_else(|_| {
                panic!("{backend_name}_QRMI_IBM_QRS_IAM_ENDPOINT environment variable is not set")
            });
        let api_key =
            env::var(format!("{backend_name}_QRMI_IBM_QRS_IAM_APIKEY")).unwrap_or_else(|_| {
                panic!("{backend_name}_QRMI_IBM_QRS_IAM_APIKEY environment variable is not set")
            });
        let service_crn = env::var(format!("{backend_name}_QRMI_IBM_QRS_SERVICE_CRN"))
            .unwrap_or_else(|_| {
                panic!("{backend_name}_QRMI_IBM_QRS_SERVICE_CRN environment variable is not set")
            });
        let session_mode = env::var(format!("{backend_name}_QRMI_IBM_QRS_SESSION_MODE"))
            .unwrap_or_else(|_| "dedicated".to_string());
        let session_max_ttl: i32 = env::var(format!("{backend_name}_QRMI_IBM_QRS_SESSION_MAX_TTL"))
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(28800);
        let timeout_secs: Option<i32> =
            env::var(format!("{backend_name}_QRMI_IBM_QRS_TIMEOUT_SECONDS"))
                .ok()
                .or_else(|| env::var(format!("{backend_name}_QRMI_JOB_TIMEOUT_SECONDS")).ok())
                .and_then(|s| s.parse::<i32>().ok());
        let session_id = env::var(format!("{backend_name}_QRMI_IBM_QRS_SESSION_ID"))
            .ok()
            .or_else(|| env::var(format!("{backend_name}_QRMI_JOB_ACQUISITION_TOKEN")).ok());
        // Set up the config
        let mut config = configuration::Configuration::new();
        config.base_path = qrs_endpoint;
        config.bearer_access_token = None;
        config.crn = Some(service_crn);

        Self {
            config,
            backend_name: backend_name.to_string(),
            session_id,
            calibration_id: None,
            timeout_secs,
            session_mode,
            session_max_ttl,
            api_key,
            iam_endpoint,
            token_expiration: 0,
            token_lifetime: 0,
        }
    }
}

impl Default for IBMQiskitRuntimeService {
    fn default() -> Self {
        Self::new("")
    }
}

// Implement the QuantumResource trait using the asynchronous wrappers.
#[async_trait]
impl QuantumResource for IBMQiskitRuntimeService {
    /// Asynchronously checks if a backend is accessible.
    async fn is_accessible(&mut self) -> bool {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }
        match backends_api::get_backend_status(&self.config, &self.backend_name, None).await {
            Ok(status_response) => {
                // Print the status, using "unknown" if no status is available
                let status_str = status_response
                    .status
                    .unwrap_or_else(|| "unknown".to_string());
                // Return true if status is "active" or "online"
                status_str.to_lowercase() == "active" || status_str.to_lowercase() == "online"
            }
            Err(e) => {
                // Print a message indicating an error occurred
                error!("status: error ({:?})", e);
                false
            }
        }
    }

    /// Creates a new session.
    ///
    /// This function wraps the qiskit_runtime_api client call to POST /sessions. The underlying
    /// function (sessions_api::create_session) builds the request with the required headers
    /// (including the API key, IAM token, and service CRN) from the configuration.
    async fn acquire(&mut self) -> Result<String> {
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }

        if let Some(existing_session_id) = self.session_id.clone() {
            let response =
                sessions_api::get_session(&self.config, &existing_session_id, None).await?;
            let active_ttl = response.active_ttl.unwrap_or(1);
            let max_ttl = response.max_ttl.unwrap_or(1);

            if max_ttl / 100 < active_ttl {
                return Ok(existing_session_id);
            } else {
                let _ = self.release(&existing_session_id).await?;
            }
        }

        let mode_value = match self.session_mode.to_lowercase().as_str() {
            "batch" => Mode::Batch,
            "dedicated" => Mode::Dedicated,
            other => bail!(format!("Invalid session mode: {}", other)),
        };
        let create_session_request_one_of = models::CreateSessionRequestOneOf {
            max_ttl: Some(self.session_max_ttl),
            mode: mode_value,
            backend: Some(self.backend_name.clone()),
            backend_name: Some(self.backend_name.clone()),
        };
        let create_session_request = models::CreateSessionRequest::CreateSessionRequestOneOf(
            Box::new(create_session_request_one_of),
        );
        let response =
            sessions_api::create_session(&self.config, None, Some(create_session_request)).await?;

        self.session_id = Some(response.id.clone());
        Ok(response.id)
    }

    /// Deletes the current session.
    ///
    /// This sends a DELETE request to /sessions/{session_id}/close via the qiskit_runtime_api client.
    async fn release(&mut self, acquisition_token: &str) -> Result<()> {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }
        sessions_api::delete_session_close(&self.config, acquisition_token, None).await?;
        self.session_id = None;
        Ok(())
    }

    /// Starts a job task.
    ///
    /// This function sends a POST request to /jobs. The input payload is parsed as JSON,
    /// and the job is created using the qiskit_runtime_api client function jobs_api::create_job.
    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }
        if let Payload::QiskitPrimitive { input, program_id } = payload {
            let params = match program_id.as_ref() {
                "sampler" => {
                    let val: Value = serde_json::from_str(&input)?;
                    let parsed = serde_json::from_value::<SamplerV2Input>(val)?;
                    CreateJobRequestOneOfAllOfParams::SamplerV2Input(Box::new(parsed))
                }
                "estimator" => {
                    let val: Value = serde_json::from_str(&input)?;
                    let parsed = serde_json::from_value::<EstimatorV2Input>(val)?;
                    CreateJobRequestOneOfAllOfParams::EstimatorV2Input(Box::new(parsed))
                }
                "noiselearner" => {
                    let val: Value = serde_json::from_str(&input)?;
                    let parsed = serde_json::from_value::<NoiseLearnerInput>(val)?;
                    CreateJobRequestOneOfAllOfParams::NoiseLearnerInput(Box::new(parsed))
                }
                &_ => {
                    bail!("Unsupported program id: {:?}", program_id);
                }
            };
            let create_job_request_one_of = models::CreateJobRequestOneOf {
                program_id,
                backend: self.backend_name.clone(),
                runtime: None,
                tags: None,
                log_level: Some(LogLevel::Debug),
                cost: self.timeout_secs,
                session_id: self.session_id.clone(),
                calibration_id: self.calibration_id.clone(),
                params: Some(Box::new(params)),
                private: Some(false),
            };
            let create_job_request = models::CreateJobRequest::CreateJobRequestOneOf(Box::new(
                create_job_request_one_of,
            ));
            let response =
                jobs_api::create_job(&self.config, None, None, Some(create_job_request)).await?;

            Ok(response.id)
        } else {
            bail!("Payload type is not supported: {:?}", payload)
        }
    }

    /// Stops a running job.
    ///
    /// This function checks the job status via GET /jobs/{id}. If the job is still running,
    /// it sends a cancellation (POST /jobs/{id}/cancel) before deleting the job with DELETE /jobs/{id}.
    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }
        let job_details = jobs_api::get_job(&self.config, task_id, None, None).await?;
        let status = job_details.status;
        if status == models::job_response::Status::Running
            || status == models::job_response::Status::Queued
        {
            let _ = jobs_api::cancel_job_jid(&self.config, task_id, None, None).await;
            //jobs_api::delete_job_jid(&self.config, task_id, None).await?;
        }
        Ok(())
    }

    /// Returns the current status of a job.
    ///
    /// This function calls GET /jobs/{id} and maps the returned status string to the
    /// TaskStatus enum.
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }
        let job_details = jobs_api::get_job(&self.config, task_id, None, None).await?;
        let status = job_details.status;
        match status {
            models::job_response::Status::Running => Ok(TaskStatus::Running),
            models::job_response::Status::Queued => Ok(TaskStatus::Queued),
            models::job_response::Status::Completed => Ok(TaskStatus::Completed),
            models::job_response::Status::Cancelled
            | models::job_response::Status::CancelledRanTooLong => Ok(TaskStatus::Cancelled),
            models::job_response::Status::Failed => Ok(TaskStatus::Failed),
        }
    }

    /// Retrieves the results of a completed job.
    ///
    /// This function calls GET /jobs/{id}/results and serializes the returned JSON into a string.
    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        } // Check if the task is completed before fetching the results.
        let job_details = jobs_api::get_job(&self.config, task_id, None, None).await?;
        let status = job_details.status;
        if status != models::job_response::Status::Completed {
            bail!("Task is not completed. Current status: {:?}", status);
        }
        let results = jobs_api::get_job_results_jid(&self.config, task_id, None).await?;
        Ok(TaskResult { value: results })
    }

    /// Returns the log messages of the task.
    ///
    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let logs = jobs_api::get_job_logs_jid(&self.config, task_id, None).await?;
        Ok(logs)
    }

    /// Retrieves target details.
    ///
    /// This function combines the results of GET /backends/{id}/configuration and
    /// GET /backends/{id}/properties into a single JSON object.
    async fn target(&mut self) -> Result<Target> {
        // Ensure the bearer token is valid
        if let Err(e) = auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut self.config.bearer_access_token,
            &mut self.token_expiration,
            &mut self.token_lifetime,
        )
        .await
        {
            error!("Token renewal failed: {:?}", e);
        }
        let mut resp = json!({});
        if let Ok(cfg) =
            backends_api::get_backend_configuration(&self.config, &self.backend_name, None).await
        {
            resp["configuration"] = serde_json::to_value(cfg)?;
        } else {
            resp["configuration"] = json!(null);
        }
        if let Ok(props) = backends_api::get_backend_properties(
            &self.config,
            &self.backend_name,
            None,
            None,
            self.calibration_id.as_deref(),
        )
        .await
        {
            resp["properties"] = serde_json::to_value(props)?;
        } else {
            resp["properties"] = json!(null);
        }
        Ok(Target {
            value: resp.to_string(),
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        if let Some(ref session) = self.session_id {
            metadata.insert("session_id".to_string(), session.clone());
        }
        metadata
    }
}
