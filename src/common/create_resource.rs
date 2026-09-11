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
use crate::Result;
use std::collections::HashMap;

use crate::alice_bob::AliceBobFelis;
use crate::ibm::{IBMQiskitRuntimeService, IBMQuantumComputeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::ResourceType;
use crate::pasqal::{PasqalCloud, PasqalLocal};
use crate::QuantumResource;

pub(crate) fn create_resource(
    resource_type: &ResourceType,
    resource_id: &str,
) -> Result<Box<dyn QuantumResource + Send + Sync>> {
    Ok(match resource_type {
        ResourceType::IBMQuantumSystem => Box::new(IBMQuantumSystem::new(resource_id)?),
        ResourceType::QiskitRuntimeService => Box::new(IBMQiskitRuntimeService::new(resource_id)?),
        ResourceType::IBMQuantumComputeService => {
            Box::new(IBMQuantumComputeService::new(resource_id)?)
        }
        ResourceType::PasqalCloud => Box::new(PasqalCloud::new(resource_id)?),
        ResourceType::PasqalLocal => Box::new(PasqalLocal::new(resource_id)?),
        ResourceType::AliceBobFelis => Box::new(AliceBobFelis::new(resource_id)?),
        ResourceType::IQMServer => Box::new(IQMServer::new(resource_id)?),
    })
}

/// Same as [`create_resource`], but builds the resource from a config map
/// instead of environment variables.
pub(crate) fn create_resource_from_config(
    resource_type: &ResourceType,
    resource_id: &str,
    config: HashMap<String, String>,
) -> Result<Box<dyn QuantumResource + Send + Sync>> {
    Ok(match resource_type {
        ResourceType::IBMQuantumSystem => {
            Box::new(IBMQuantumSystem::from_config(resource_id, config)?)
        }
        ResourceType::QiskitRuntimeService => {
            Box::new(IBMQiskitRuntimeService::from_config(resource_id, config)?)
        }
        ResourceType::IBMQuantumComputeService => {
            Box::new(IBMQuantumComputeService::from_config(resource_id, config)?)
        }
        ResourceType::PasqalLocal => Box::new(PasqalLocal::from_config(resource_id, config)?),
        ResourceType::PasqalCloud => Box::new(PasqalCloud::from_config(resource_id, config)?),
        ResourceType::AliceBobFelis => Box::new(AliceBobFelis::from_config(resource_id, config)?),
        ResourceType::IQMServer => Box::new(IQMServer::from_config(resource_id, config)?),
    })
}
