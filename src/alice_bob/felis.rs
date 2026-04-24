
use crate::models::{ResourceType, Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use async_trait::async_trait;
use std::env;
use std::collections::HashMap;
use uuid::Uuid;
use alice_bob_felis::apis::{configuration, targets_service, jobs_service};
use alice_bob_felis::models::{EventType, create_external_job};
use alice_bob_felis::models;
use serde_json::json;
use base64::{engine::general_purpose, Engine};
use anyhow::{anyhow, bail, Result};

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
    /// * QRMI_FELIS_API_KEY: API key obtained from the Felis web console
    /// * QRMI_AB_FELIS_BASE_ENDPOINT: URL for Felis API base endpoint
    /// 
    /// These may be optionally be prefixed by the backend name
    pub fn new(backend_name: &str) -> Result<Self> {
        // Handle environment variables
        let api_key =
            env::var(format!("{backend_name}_QRMI_AB_FELIS_API_KEY"))
            .or(env::var("QRMI_AB_FELIS_API_KEY"))
            .map_err(|_| {
                anyhow!(
                    "QRMI_AB_FELIS_API_KEY environment variable is not set"
                )
            })?;
        let endpoint =
            env::var(format!("{backend_name}_QRMI_AB_FELIS_BASE_ENDPOINT"))
            .or(env::var("QRMI_AB_FELIS_BASE_ENDPOINT"))
            .map_err(|_| {
                anyhow!(
                    "QRMI_AB_FELIS_BASE_ENDPOINT environment variable is not set"
                )
            })?;
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
        let targets = targets_service::list_targets(&self.config).await?;
        let names: Vec<String> = targets.iter().map(|t| target_to_device(&t.name.clone())).collect();
        Ok(names)
    }

    async fn most_recent_event(&mut self, task_id: &str) -> Result<models::EventType> {
        let job = jobs_service::get_job(&self.config, &task_id, None).await?;
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
    device.strip_prefix("ab_").unwrap_or(&device).replacen("_", ":", 2).to_uppercase()
}

// Felis API keys are basic auth credentials in disguise.
// We need to decode them to pass them to our client in the format it expects.
// The client will then reencode.
fn decode_api_key(encoded: &str) -> Result<Option<(String, Option<String>)>, Box<dyn std::error::Error>> {
    // Decode from Base64
    let decoded_bytes = general_purpose::STANDARD.decode(encoded)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;

    // Split at the first ':'
    let mut parts = decoded_str.splitn(2, ':');

    let username = parts
        .next()
        .ok_or("missing username part")?
        .to_string();

    let password = parts
        .next()
        .ok_or("missing password part")?
        .to_string();

    Ok(Some((username, Some(password))))
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

    async fn acquire(&mut self) -> Result<String> {
        // Felis has no such concept, so we return a random UUID
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        if let Payload::AliceBobFelis { human_qir, input_params } = payload {
            let job = create_external_job::CreateExternalJob {
                // For now Felis supports only a single input/output format
                input_data_format: Some(json!("HUMAN_QIR")),
                output_data_format: Some(json!("HISTOGRAM")),
                target: self.felis_target.clone(),
                input_params: serde_json::from_str(&input_params).unwrap(),
            };
            
            let external_job = jobs_service::create_job(&self.config, job, None).await?;
            jobs_service::upload_input(&self.config, &external_job.id, human_qir, None).await?;
            // If here we can assume all went well
            Ok(external_job.id)

        } else {
            bail!(format!("Payload type {:?} is not supported by Felis.", payload))
        }
    }

    // task_stop seems to be expected to be idempotent
    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let event = &self.most_recent_event(task_id).await?;
        if *event != EventType::Succeeded &&
           *event != EventType::Cancelled {
            jobs_service::cancel_job(&self.config, task_id, None).await?;
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
        let output_csv = jobs_service::download_output(&self.config, &task_id, None).await?;
        Ok(TaskResult { value: output_csv })
    }

    async fn task_logs(&mut self, _task_id: &str) -> Result<String> {
        Ok("Logging not implemented for this QuantumResource".to_string())
    }

    async fn target(&mut self) -> Result<Target> {
        let targets = targets_service::list_targets(&self.config).await?;

        let target = targets
            .iter()
            .find(|obj| obj.name == self.felis_target)
            .expect(&format!("No matching target found {t}", t=self.felis_target));

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
