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

//! High-level entry point for discovering the QPU resources assigned to a job.

use crate::alice_bob::AliceBobFelis;
use crate::ibm::{IBMQiskitRuntimeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::ResourceType;
use crate::pasqal::{PasqalCloud, PasqalLocal};
use crate::QuantumResource;
use anyhow::{bail, Result};
use std::collections::HashMap;

/// Discovers the QPU resources assigned to the current job -- read from the
/// `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES` environment variables, or
/// their legacy `SLURM_JOB_QPU_RESOURCES` / `SLURM_JOB_QPU_TYPES`
/// equivalents -- and exposes the ones that are currently accessible as
/// [`QuantumResource`] instances.
///
/// This mirrors [`crate::resource_provider::ResourceProvider`] in spirit,
/// except that the set of resources comes from the job's environment rather
/// than from querying a vendor endpoint, and the result is discovered once
/// and cached (repeated calls to [`resource`](QRMIService::resource) return
/// the *same* instance) rather than re-fetched on every call.
///
/// # Example
///
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use qrmi::QRMIService;
///
///     let mut service = QRMIService::new().await?;
///     for resource in service.resources() {
///         println!("{}", resource.resource_id().await?);
///     }
///
///     if let Some(resource) = service.resource("ibm_torino") {
///         let token = resource.acquire().await?;
///         println!("acquisition token = {}", token);
///     }
///     Ok(())
/// }
/// ```
pub struct QRMIService {
    resources: HashMap<String, Box<dyn QuantumResource + Send + Sync>>,
}

impl QRMIService {
    /// Discovers and constructs the accessible QRMI resources for the
    /// current job.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `QRMI_PLUGIN_ERROR` is set (the QRMI plugin recorded a resource
    ///   acquisition failure there).
    /// - The job's QPU resource/type environment variables are not set, or
    ///   specify inconsistent numbers of resources and types.
    /// - Constructing or querying the accessibility of one of the specified
    ///   resources fails.
    pub async fn new() -> Result<Self> {
        crate::common::initialize();

        // If resource acquisition failed in the QRMI plugin, the error
        // reason may be available via this environment variable.
        if let Ok(plugin_error) = std::env::var("QRMI_PLUGIN_ERROR") {
            bail!(plugin_error);
        }

        let (qpus, qpu_types) = crate::common::get_job_qpu_resources_and_types()?;
        log::debug!("qpus: {:?}", qpus);
        log::debug!("qpu types: {:?}", qpu_types);

        let mut resources: HashMap<String, Box<dyn QuantumResource + Send + Sync>> =
            HashMap::new();
        for (qpu, qpu_type) in qpus.iter().zip(qpu_types.iter()) {
            let qpu = qpu.trim();
            let Some(resource_type) = ResourceType::from_qpu_type_str(qpu_type) else {
                log::warn!(
                    "Unsupported resource type: {} specified for {}",
                    qpu_type,
                    qpu
                );
                continue;
            };

            let mut resource = create_resource(&resource_type, qpu)?;
            if resource.is_accessible().await? {
                resources.insert(qpu.to_string(), resource);
            } else {
                log::debug!("{} is not accessible now. ignored.", qpu);
            }
        }

        Ok(Self { resources })
    }

    /// Returns all accessible QRMI resources.
    pub fn resources(&mut self) -> Vec<&mut (dyn QuantumResource + Send + Sync + 'static)> {
        self.resources.values_mut().map(|r| r.as_mut()).collect()
    }

    /// Returns a single resource matching the specified resource identifier,
    /// i.e. backend name for IBM Quantum, or `None` if not found.
    pub fn resource(
        &mut self,
        resource_id: &str,
    ) -> Option<&mut (dyn QuantumResource + Send + Sync + 'static)> {
        self.resources.get_mut(resource_id).map(|r| r.as_mut())
    }

    /// Consumes this service, yielding ownership of its accessible
    /// resources.
    ///
    /// Not part of the public API: callers who just want to use the
    /// resources should use [`resources`](Self::resources) or
    /// [`resource`](Self::resource) instead, which borrow rather than take
    /// ownership. This exists for the C and Python bindings
    /// (`cext::qrmi_service_resources`, `pyext::PyQRMIService`), which each
    /// need to move every resource into its own, independently owned
    /// handle/object -- mirroring how `cext::qrmi_provider_resources` and
    /// `pyext::PyResourceProvider::resources` each wrap the
    /// `Box<dyn QuantumResource>`s returned by `ResourceProvider::resources`.
    pub(crate) fn into_resource_map(self) -> HashMap<String, Box<dyn QuantumResource + Send + Sync>> {
        self.resources
    }
}

fn create_resource(
    resource_type: &ResourceType,
    resource_id: &str,
) -> Result<Box<dyn QuantumResource + Send + Sync>> {
    Ok(match resource_type {
        ResourceType::IBMQuantumSystem => Box::new(IBMQuantumSystem::new(resource_id)?),
        ResourceType::QiskitRuntimeService => Box::new(IBMQiskitRuntimeService::new(resource_id)?),
        ResourceType::PasqalCloud => Box::new(PasqalCloud::new(resource_id)?),
        ResourceType::PasqalLocal => Box::new(PasqalLocal::new(resource_id)?),
        ResourceType::AliceBobFelis => Box::new(AliceBobFelis::new(resource_id)?),
        ResourceType::IQMServer => Box::new(IQMServer::new(resource_id)?),
    })
}
