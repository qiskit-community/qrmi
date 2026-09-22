// This code is part of Qiskit.
//
// (C) Copyright Alice and Bob 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! QRMI implementation for Alice and Bob Felis

use crate::alice_bob::error::{classify, ResourceKind};
use crate::common::resolve_opt_required_any;
use crate::error::QrmiError;
use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::{QuantumResource, Result};
use alice_bob_felis::apis::{configuration, jobs_service, targets_service};
use alice_bob_felis::helpers::decode_api_key;
use alice_bob_felis::models;
use alice_bob_felis::models::{create_external_job, EventType};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;

/// QR implementation for Alice and Bob's Cloud API, Felis
pub struct AliceBobFelis {
    pub(crate) config: configuration::Configuration,
    pub(crate) backend_name: String,
    pub(crate) felis_target: String,
}

impl AliceBobFelis {
    /// Constructs a Felis QR
    ///
    /// # Environment variables
    ///
    /// * QRMI_AB_FELIS_API_KEY: API key obtained from the Felis web console
    /// * QRMI_AB_FELIS_BASE_ENDPOINT: URL for Felis API base endpoint
    ///
    /// These may be optionally be prefixed by the backend name
    pub fn new(backend_name: &str) -> Result<Self> {
        Self::from_opt(backend_name, None)
    }

    /// Constructs a Felis QR from a config map, instead of environment
    /// variables.
    ///
    /// Accepts the same keys as [`Self::new`]'s environment variables,
    /// without the backend-name prefix (a config map is already scoped to
    /// one backend). Each key also accepts its fully lowercased form (e.g.
    /// `qrmi_ab_felis_api_key`) as a fallback if the exact-case key isn't
    /// present in the map.
    pub fn from_config(backend_name: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_opt(backend_name, Some(&config))
    }

    /// Shared parsing and client-building logic for [`Self::new`]
    /// (`config: None`, reads OS environment variables, optionally prefixed
    /// by the backend name) and [`Self::from_config`] (`config: Some`,
    /// reads the given map).
    fn from_opt(backend_name: &str, config: Option<&HashMap<String, String>>) -> Result<Self> {
        // Env vars are per-instance (`<backend_name>_QRMI_...`); config map
        // keys use the same name minus that prefix.
        let prefix = if config.is_some() {
            String::new()
        } else {
            format!("{backend_name}_")
        };
        // Neither name is deprecated, so both are tried and named on failure.
        let prefixed_api_key = format!("{prefix}QRMI_AB_FELIS_API_KEY");
        let api_key =
            resolve_opt_required_any(&[&prefixed_api_key, "QRMI_AB_FELIS_API_KEY"], config)?;
        let prefixed_endpoint = format!("{prefix}QRMI_AB_FELIS_BASE_ENDPOINT");
        let endpoint =
            resolve_opt_required_any(&[&prefixed_endpoint, "QRMI_AB_FELIS_BASE_ENDPOINT"], config)?;
        let mut config = configuration::Configuration::new();
        config.base_path = endpoint;
        config.basic_auth = decode_api_key(&api_key).unwrap();
        Ok(Self {
            config,
            backend_name: backend_name.to_string(),
            felis_target: device_to_target(backend_name),
        })
    }

    pub async fn list_backends(&mut self) -> Result<Vec<String>> {
        let targets = targets_service::list_targets(&self.config)
            .await
            .map_err(|e| classify(e, ResourceKind::Backend))?;
        let names: Vec<String> = targets
            .iter()
            .map(|t| target_to_device(&t.name.clone()))
            .collect();
        Ok(names)
    }

    async fn most_recent_event(&mut self, task_id: &str) -> Result<models::EventType> {
        let job = jobs_service::get_job(&self.config, task_id, None)
            .await
            .map_err(|e| classify(e, ResourceKind::Job))?;
        // Can safely assume events is non-empty
        let event = job.events.last().unwrap().r#type;
        Ok(event)
    }
}

// e.g. Felis target    EMU:40Q:PHYSICAL_CATS
// becomes device       ab_emu_40q_physical_cats
fn target_to_device(target: &str) -> String {
    format!("ab_{}", target.replace(":", "_").to_lowercase())
}

fn device_to_target(device: &str) -> String {
    device
        .strip_prefix("ab_")
        .unwrap_or(device)
        .replacen("_", ":", 2)
        .to_uppercase()
}

#[async_trait]
impl QuantumResource for AliceBobFelis {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::AliceBobFelis)
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        // We can implement this later
        Ok(true)
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        if let Payload::AliceBobFelis {
            human_qir,
            input_params,
        } = payload
        {
            let job = create_external_job::CreateExternalJob {
                // For now Felis supports only a single input/output format
                input_data_format: Some(json!("HUMAN_QIR")),
                output_data_format: Some(json!("HISTOGRAM")),
                target: self.felis_target.clone(),
                input_params: serde_json::from_str(&input_params).unwrap(),
            };

            let external_job = jobs_service::create_job(&self.config, job, None)
                .await
                .map_err(|e| classify(e, ResourceKind::Backend))?;
            jobs_service::upload_input(&self.config, &external_job.id, human_qir, None)
                .await
                .map_err(|e| classify(e, ResourceKind::Job))?;
            // If here we can assume all went well
            Ok(external_job.id)
        } else {
            Err(QrmiError::UnsupportedPayload(format!("{payload:?}")))
        }
    }

    // task_stop seems to be expected to be idempotent
    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let event = &self.most_recent_event(task_id).await?;
        if *event != EventType::Succeeded && *event != EventType::Cancelled {
            jobs_service::cancel_job(&self.config, task_id, None)
                .await
                .map_err(|e| classify(e, ResourceKind::Job))?;
        }
        Ok(())
    }

    // Here we map Felis Events to QRMI Statuses
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let event = &self.most_recent_event(task_id).await?;
        match event {
            EventType::Created => Ok(TaskStatus::Queued),
            EventType::FetchingInput => Ok(TaskStatus::Queued),
            EventType::InputReady => Ok(TaskStatus::Queued),
            EventType::Compiling => Ok(TaskStatus::Running),
            EventType::Compiled => Ok(TaskStatus::Running),
            EventType::Transpiling => Ok(TaskStatus::Running),
            EventType::Transpiled => Ok(TaskStatus::Running),
            EventType::Executing => Ok(TaskStatus::Running),
            EventType::Succeeded => Ok(TaskStatus::Completed),
            EventType::Cancelled => Ok(TaskStatus::Cancelled),
            EventType::TimedOut => Ok(TaskStatus::Failed),
            EventType::CompilationFailed => Ok(TaskStatus::Failed),
            EventType::ExecutionFailed => Ok(TaskStatus::Failed),
            EventType::TranspilationFailed => Ok(TaskStatus::Failed),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let output_csv = jobs_service::download_output(&self.config, task_id, None)
            .await
            .map_err(|e| classify(e, ResourceKind::Job))?;
        Ok(TaskResult { value: output_csv })
    }

    #[allow(clippy::expect_fun_call)]
    async fn target(&mut self) -> Result<Target> {
        let targets = targets_service::list_targets(&self.config)
            .await
            .map_err(|e| classify(e, ResourceKind::Backend))?;

        let target = targets
            .iter()
            .find(|obj| obj.name == self.felis_target)
            .expect(&format!(
                "No matching target found {t}",
                t = self.felis_target
            ));

        Ok(Target {
            value: serde_json::to_string(&target).unwrap(),
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("felis_target".to_string(), self.felis_target.clone());
        metadata
    }
}
