use crate::ibm::{IBMQiskitRuntimeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::{Config, ResourceType};
use crate::pasqal::{PasqalCloud, PasqalLocal};
use crate::QuantumResource;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::env;
use std::{collections::HashMap, fs};

pub struct QRMIService {
    resources: HashMap<String, Box<dyn QuantumResource + Send + Sync>>,
}

impl QRMIService {
    pub fn from_environment() -> Result<Self> {
        let mut service = Self {
            resources: HashMap::new(),
        };
        let env_var_resource_names = env::var("QRMI_JOB_QPU_RESOURCES").unwrap_or_else(|_| "<not set>".to_string());
        let env_var_resource_types = env::var("QRMI_JOB_QPU_TYPES").unwrap_or_else(|_| "<not set>".to_string());

        let resource_names: Vec<&str> = env_var_resource_names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .collect();
        let resource_types: Vec<&str> = env_var_resource_types
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect();

        for (index, resource_id) in resource_names.iter().enumerate() {
            let resource_type = resource_types.get(index).map(|value| {
                let normalised = value.to_ascii_lowercase();
                match normalised.as_str() {
                    "ibm-quantum-system" | "ibm_quantum_system" => ResourceType::IBMQuantumSystem,
                    "qiskit-runtime-service" | "qiskit_runtime_service" => {
                        ResourceType::QiskitRuntimeService
                    }
                    "pasqal-cloud" | "pasqal_cloud" => ResourceType::PasqalCloud,
                    "pasqal-local" | "pasqal_local" => ResourceType::PasqalLocal,
                    "alice-bob-felis" | "alice_bob_felis" => ResourceType::AliceBobFelis,
                    "iqm-server" | "iqm_server" => ResourceType::IQMServer,
                    _ => ResourceType::QiskitRuntimeService,
                }
            });

            let resource_type = resource_type.ok_or_else(|| {
                anyhow!("Missing or invalid resource type for: {}", resource_id)
            })?;

            service.add_resource(resource_id, resource_type)?;
        }

        Ok(service)
    }

    // Unfinished, Unsure if this is needed
    pub fn from_config(file_path: &str) -> Result<Self> {
        let config = Config::load(file_path)?;

        let mut service = Self {
            resources: HashMap::new(),
        };

        for (resource_id, resource_def) in config.resource_map {
            service.add_resource(&resource_id, resource_def.r#type)?;
        }

        Ok(service)
    }
    fn add_resource(&mut self, resource_id: &str, resource_type: ResourceType) -> Result<()> {
        let resource: Box<dyn QuantumResource + Send + Sync> = match resource_type {
            ResourceType::IBMQuantumSystem => Box::new(IBMQuantumSystem::new(resource_id)?),
            ResourceType::QiskitRuntimeService => {
                Box::new(IBMQiskitRuntimeService::new(resource_id)?)
            }
            ResourceType::PasqalCloud => Box::new(PasqalCloud::new(resource_id)?),
            ResourceType::PasqalLocal => Box::new(PasqalLocal::new(resource_id)?),
            ResourceType::AliceBobFelis => {
                return Err(anyhow!("AliceBobFelis is not supported yet"));
            }
            ResourceType::IQMServer => Box::new(IQMServer::new(resource_id)?),
        };

        self.resources.insert(resource_id.to_string(), resource);
        Ok(())
    }

    pub fn resources(&self) -> Vec<&Box<dyn QuantumResource + Send + Sync>> {
        self.resources.values().collect()
    }

    pub async fn print_resources(&mut self) -> Result<()> {
        for (name, resource) in &mut self.resources {
            let resource_id = resource.resource_id().await?;
            let resource_type = resource.resource_type().await?;

            println!("resource {} -> id={}, type={}", name, resource_id, resource_type.as_str());
        }

        Ok(())
    }
}

// Needs to be changed to use a dummy QRMI resource for testing
#[tokio::test]
async fn discovers_resources_from_job_qpu_environment() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug)]
    struct EnvFile(HashMap<String, String>);

    let json_str = fs::read_to_string("../env.json")?;
    let env: EnvFile = serde_json::from_str(&json_str)?;

    for (k, v) in env.0 {
        env::set_var(&k, &v);
    }

    let mut service = QRMIService::from_environment()?;

    let resources = service.resources();
    assert_eq!(resources.len(), 2);

    let mut resource = service
        .resources
        .values_mut()
        .next()
        .expect("expected at least one resource");

    let resource_id = resource.resource_id().await?;
    let resource_type = resource.resource_type().await?;
    let is_accessible = resource.is_accessible().await?;

    println!("Resource info: id={}, type={}, accessible={}", resource_id, resource_type.as_str(), is_accessible);

    service.print_resources().await?;
    Ok(())
}
