use crate::ibm::{IBMQiskitRuntimeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::ResourceType;
use crate::pasqal::{PasqalCloud, PasqalLocal};
use crate::QuantumResource;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::env;

pub struct QRMIService {
    resources: HashMap<String, Box<dyn QuantumResource + Send + Sync>>,
}

impl QRMIService {
    pub async fn from_environment() -> Result<Self> {
        let plugin_error = env::var("QRMI_PLUGIN_ERROR").ok();
        if let Some(plugin_error) = plugin_error {
            return Err(anyhow!(plugin_error));
        }

        let mut service = Self {
            resources: HashMap::new(),
        };

        let env_var_resource_names = env::var("QRMI_JOB_QPU_RESOURCES")?;
        let env_var_resource_types = env::var("QRMI_JOB_QPU_TYPES")?;

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
            let Some(resource_type_str) = resource_types.get(index) else {
                eprintln!("Missing resource type for {}", resource_id);
                continue;
            };
            let normalised = resource_type_str.to_ascii_lowercase();

            let resource_type = match normalised.as_str() {
                "ibm-quantum-system" | "ibm_quantum_system" => ResourceType::IBMQuantumSystem,
                "qiskit-runtime-service" | "qiskit_runtime_service" => {
                    ResourceType::QiskitRuntimeService
                }
                "pasqal-cloud" | "pasqal_cloud" => ResourceType::PasqalCloud,
                "pasqal-local" | "pasqal_local" => ResourceType::PasqalLocal,
                "alice-bob-felis" | "alice_bob_felis" => ResourceType::AliceBobFelis,
                "iqm-server" | "iqm_server" => ResourceType::IQMServer,
                _ => {
                    eprintln!(
                        "Unsupported resource type: {} specified for {}",
                        resource_type_str, resource_id
                    );
                    continue;
                }
            };

            service.add_resource(resource_id, resource_type).await?;
        }

        Ok(service)
    }

    async fn add_resource(&mut self, resource_id: &str, resource_type: ResourceType) -> Result<()> {
        let mut resource: Box<dyn QuantumResource + Send + Sync> = match resource_type {
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

        match resource.is_accessible().await {
            Ok(true) => {
                self.resources.insert(resource_id.to_string(), resource);
            }
            Ok(false) => {
                eprintln!("{} is not accessible now. ignored.", resource_id);
            }
            Err(error) => {
                eprintln!(
                    "Could not determine accessibility for {}: {}. ignored.",
                    resource_id, error
                );
            }
        }
        Ok(())
    }

    pub fn resource(&self, resource_id: &str) -> Option<&(dyn QuantumResource + Send + Sync)> {
        self.resources.get(resource_id).map(Box::as_ref)
    }

    pub fn resources(&self) -> Vec<&(dyn QuantumResource + Send + Sync)> {
        self.resources.values().map(Box::as_ref).collect()
    }

    pub async fn print_resources(&mut self) -> Result<()> {
        for resource in self.resources.values_mut() {
            let resource_id = resource.resource_id().await?;
            let resource_type = resource.resource_type().await?;

            println!(
                "resource -> id={}, type={}",
                resource_id,
                resource_type.as_str()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    #[ignore = "requires real QRMI provider credentials and configured resources"]
    async fn discovers_real_resources_from_environment() -> Result<(), Box<dyn std::error::Error>> {
        let mut service = QRMIService::from_environment().await?;

        let resources = service.resources();

        assert!(!resources.is_empty());

        service.print_resources().await?;

        Ok(())
    }

    fn restore_env_var(name: &str, previous_value: Option<String>) {
        if let Some(value) = previous_value {
            env::set_var(name, value);
        } else {
            env::remove_var(name);
        }
    }

    #[tokio::test]
    #[serial]
    async fn errors_when_plugin_error_is_set() {
        let previous_plugin_error = env::var("QRMI_PLUGIN_ERROR").ok();

        env::remove_var("QRMI_PLUGIN_ERROR");
        env::set_var("QRMI_PLUGIN_ERROR", "Test plugin error");

        let result = QRMIService::from_environment().await;

        restore_env_var("QRMI_PLUGIN_ERROR", previous_plugin_error);

        match result {
            Ok(_) => panic!("expected from_environment to fail when QRMI_PLUGIN_ERROR is set"),
            Err(error) => {
                assert!(error
                    .to_string()
                    .contains("Test plugin error"));
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn errors_when_resource_names_are_missing() {
        let previous_plugin_error = env::var("QRMI_PLUGIN_ERROR").ok();
        let previous_resources = env::var("QRMI_JOB_QPU_RESOURCES").ok();
        let previous_types = env::var("QRMI_JOB_QPU_TYPES").ok();

        env::remove_var("QRMI_PLUGIN_ERROR");
        env::remove_var("QRMI_JOB_QPU_RESOURCES");
        env::set_var("QRMI_JOB_QPU_TYPES", "ibm-quantum-system");

        let result = QRMIService::from_environment().await;

        restore_env_var("QRMI_PLUGIN_ERROR", previous_plugin_error);
        restore_env_var("QRMI_JOB_QPU_RESOURCES", previous_resources);
        restore_env_var("QRMI_JOB_QPU_TYPES", previous_types);

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn errors_when_resource_types_are_missing() {
        let previous_plugin_error = env::var("QRMI_PLUGIN_ERROR").ok();
        let previous_resources = env::var("QRMI_JOB_QPU_RESOURCES").ok();
        let previous_types = env::var("QRMI_JOB_QPU_TYPES").ok();

        env::remove_var("QRMI_PLUGIN_ERROR");
        env::set_var("QRMI_JOB_QPU_RESOURCES", "qpu-a");
        env::remove_var("QRMI_JOB_QPU_TYPES");

        let result = QRMIService::from_environment().await;

        restore_env_var("QRMI_PLUGIN_ERROR", previous_plugin_error);
        restore_env_var("QRMI_JOB_QPU_RESOURCES", previous_resources);
        restore_env_var("QRMI_JOB_QPU_TYPES", previous_types);

        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn ignores_unsupported_resource_type() -> Result<(), Box<dyn std::error::Error>> {
        let previous_plugin_error = env::var("QRMI_PLUGIN_ERROR").ok();
        let previous_resources = env::var("QRMI_JOB_QPU_RESOURCES").ok();
        let previous_types = env::var("QRMI_JOB_QPU_TYPES").ok();

        env::remove_var("QRMI_PLUGIN_ERROR");
        env::set_var("QRMI_JOB_QPU_RESOURCES", "unsupported-qpu");
        env::set_var("QRMI_JOB_QPU_TYPES", "not-a-real-resource-type");

        let service = QRMIService::from_environment().await?;

        restore_env_var("QRMI_PLUGIN_ERROR", previous_plugin_error);
        restore_env_var("QRMI_JOB_QPU_RESOURCES", previous_resources);
        restore_env_var("QRMI_JOB_QPU_TYPES", previous_types);

        assert_eq!(service.resources().len(), 0);
        assert!(service.resource("unsupported-qpu").is_none());

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn ignores_resource_without_corresponding_type() -> Result<(), Box<dyn std::error::Error>>
    {
        let previous_plugin_error = env::var("QRMI_PLUGIN_ERROR").ok();
        let previous_resources = env::var("QRMI_JOB_QPU_RESOURCES").ok();
        let previous_types = env::var("QRMI_JOB_QPU_TYPES").ok();

        env::remove_var("QRMI_PLUGIN_ERROR");
        env::set_var("QRMI_JOB_QPU_RESOURCES", "qpu-a");
        env::set_var("QRMI_JOB_QPU_TYPES", "");

        let service = QRMIService::from_environment().await?;

        restore_env_var("QRMI_PLUGIN_ERROR", previous_plugin_error);
        restore_env_var("QRMI_JOB_QPU_RESOURCES", previous_resources);
        restore_env_var("QRMI_JOB_QPU_TYPES", previous_types);

        assert_eq!(service.resources().len(), 0);
        assert!(service.resource("qpu-a").is_none());

        Ok(())
    }
}
