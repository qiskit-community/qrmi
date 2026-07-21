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

use anyhow::{anyhow, Result};
use std::collections::HashMap;

use qrmi_core_api::{ResourceProvider, ResourceType};

/// Creates a [`ResourceProvider`] from a [`ResourceType`] and environment variable map.
///
/// This factory function allows creating a provider without knowing the concrete type
/// at compile time, which is useful when the type is read from a config file.
///
/// # Example
///
/// ```no_run
/// use qrmi::models::Config;
/// use qrmi::resource_provider::create_provider;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load("/path/to/qrmi_config.json")?;
/// let resource_def = &config.resource_map["ibm_inst1"];
/// let provider = create_provider(&resource_def.r#type, &resource_def.environment)?;
/// # Ok(())
/// # }
/// ```
pub fn create_provider(
    resource_type: &ResourceType,
    environment: &HashMap<String, String>,
) -> Result<Box<dyn ResourceProvider>> {
    match resource_type {
        ResourceType::QiskitRuntimeService => Ok(Box::new(
            crate::ibm::IBMQiskitRuntimeServiceProvider::new(environment)?,
        )),
        ResourceType::IBMQuantumSystem => Ok(Box::new(crate::ibm::IBMQuantumSystemProvider::new(
            environment,
        )?)),
        _ => Err(anyhow!(
            "Unsupported resource type for dynamic resource discovery: {}",
            resource_type.as_str()
        )),
    }
}
