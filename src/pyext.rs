// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025, 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::alice_bob::AliceBobFelis;
use crate::ibm::IBMQiskitRuntimeServiceProvider;
use crate::ibm::{IBMDirectAccess, IBMQiskitRuntimeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::pasqal::PasqalCloud;
use crate::pasqal::PasqalLocal;
use crate::QuantumResource;
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};
use tokio::runtime::Runtime;

#[pyclass(eq, eq_int, hash, frozen)]
#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    IBMDirectAccess,
    IBMQuantumSystem,
    IBMQiskitRuntimeService,
    PasqalCloud,
    PasqalLocal,
    AliceBobFelis,
    IQMServer,
}

#[gen_stub_pyclass]
#[pyclass]
#[pyo3(name = "QuantumResource")]
pub struct PyQuantumResource {
    qrmi: Box<dyn QuantumResource + Send + Sync>,
    rt: Runtime,
}

impl PyQuantumResource {
    /// Internal constructor used by `PyResourceProvider::backends()`.
    pub(crate) fn from_inner(qrmi: Box<dyn QuantumResource + Send + Sync>) -> Self {
        Self {
            qrmi,
            rt: Runtime::new().unwrap(),
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyQuantumResource {
    #[new]
    pub fn new(resource_id: &str, resource_type: ResourceType) -> PyResult<Self> {
        crate::common::initialize();
        let qrmi: Box<dyn QuantumResource + Send + Sync> = match resource_type {
            ResourceType::IBMDirectAccess => match IBMDirectAccess::new(resource_id) {
                Ok(v) => Box::new(v),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                }
            },
            ResourceType::IBMQuantumSystem => match IBMQuantumSystem::new(resource_id) {
                Ok(v) => Box::new(v),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                }
            },
            ResourceType::IBMQiskitRuntimeService => {
                match IBMQiskitRuntimeService::new(resource_id) {
                    Ok(v) => Box::new(v),
                    Err(e) => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                    }
                }
            }
            ResourceType::PasqalCloud => match PasqalCloud::new(resource_id) {
                Ok(v) => Box::new(v),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                }
            },
            ResourceType::PasqalLocal => match PasqalLocal::new(resource_id) {
                Ok(v) => Box::new(v),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                }
            },
            ResourceType::AliceBobFelis => match AliceBobFelis::new(resource_id) {
                Ok(v) => Box::new(v),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                }
            },
            ResourceType::IQMServer => match IQMServer::new(resource_id) {
                Ok(v) => Box::new(v),
                Err(e) => {
                    return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string()));
                }
            },
        };

        Ok(Self {
            qrmi,
            rt: Runtime::new().unwrap(),
        })
    }

    fn is_accessible(&mut self) -> PyResult<bool> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.is_accessible().await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn resource_id(&mut self) -> PyResult<String> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.resource_id().await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn resource_type(&mut self) -> PyResult<ResourceType> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.resource_type().await });
        match result {
            Ok(v) => Ok(match v {
                crate::models::ResourceType::IBMDirectAccess => ResourceType::IBMDirectAccess,
                crate::models::ResourceType::IBMQuantumSystem => ResourceType::IBMQuantumSystem,
                crate::models::ResourceType::QiskitRuntimeService => {
                    ResourceType::IBMQiskitRuntimeService
                }
                crate::models::ResourceType::PasqalCloud => ResourceType::PasqalCloud,
                crate::models::ResourceType::PasqalLocal => ResourceType::PasqalLocal,
                crate::models::ResourceType::AliceBobFelis => ResourceType::AliceBobFelis,
                crate::models::ResourceType::IQMServer => ResourceType::IQMServer,
            }),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn acquire(&mut self) -> PyResult<String> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.acquire().await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn release(&mut self, id: &str) -> PyResult<()> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.release(id).await });
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_start(&mut self, payload: Payload) -> PyResult<String> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.qrmi.task_start(payload).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_stop(&mut self, task_id: &str) -> PyResult<()> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.qrmi.task_stop(task_id).await });
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_status(&mut self, task_id: &str) -> PyResult<TaskStatus> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.qrmi.task_status(task_id).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_result(&mut self, task_id: &str) -> PyResult<TaskResult> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.qrmi.task_result(task_id).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_logs(&mut self, task_id: &str) -> PyResult<String> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.qrmi.task_logs(task_id).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn target(&mut self) -> PyResult<Target> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.target().await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn metadata(&mut self) -> PyResult<std::collections::HashMap<String, String>> {
        crate::common::initialize();
        let result = self.rt.block_on(async { self.qrmi.metadata().await });
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// ResourceProvider Python bindings
// ---------------------------------------------------------------------------

/// Python wrapper for `ResourceProvider`.
///
/// Reads connection parameters from `QRMI_RESOURCE_PROVIDER_CONFIG_FILE`
/// on construction and exposes `resources(filters)` and `least_busy(filters)` to Python.
///
/// # Example (Python)
///
/// ```python
/// import os
/// from qrmi import ResourceProvider, ResourceType
///
/// os.environ["QRMI_RESOURCE_PROVIDER_CONFIG_FILE"] = "/path/to/qrmi_config.json"
///
/// provider = ResourceProvider(ResourceType.IBMQiskitRuntimeService)
///
/// # List all non-simulator resources, sorted by queue length
/// resources = provider.resources()
///
/// # With filters
/// resources = provider.resources("num_qubits=127&name=ibm_*&status=online")
///
/// # Least busy
/// resource = provider.least_busy()
///
/// for r in resources:
///     print(r.resource_id())
/// ```
#[gen_stub_pyclass]
#[pyclass]
#[pyo3(name = "ResourceProvider")]
pub struct PyResourceProvider {
    inner: Box<dyn crate::ResourceProvider>,
    rt: Runtime,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyResourceProvider {
    /// Constructs a new provider for the given resource type.
    ///
    /// Reads `QRMI_RESOURCE_PROVIDER_CONFIG_FILE` and locates the matching
    /// provider block in the config file.
    ///
    /// Currently supported resource types:
    /// - `ResourceType.IBMQiskitRuntimeService`
    #[new]
    pub fn new(resource_type: ResourceType) -> PyResult<Self> {
        crate::common::initialize();
        let inner: Box<dyn crate::ResourceProvider> = match resource_type {
            ResourceType::IBMQiskitRuntimeService => match IBMQiskitRuntimeServiceProvider::new() {
                Ok(p) => Box::new(p),
                Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            },
            _ => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Unsupported resource type",
                ))
            }
        };
        Ok(Self {
            inner,
            rt: Runtime::new().unwrap(),
        })
    }

    /// Returns available quantum resources, optionally filtered.
    ///
    /// # Arguments
    ///
    /// * `filters` - Filter string of the form `key=value&key=value`, or `None`.
    ///
    /// Supported keys:
    /// - `num_qubits=<N>`      — only backends with `qubits >= N`
    /// - `name=<glob>`         — only backends whose name matches the glob pattern
    /// - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
    /// - `status=online`       — only online backends
    ///
    /// Results are sorted by `queue_length` ascending (least busy first).
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// resources = provider.resources()
    /// resources = provider.resources("num_qubits=127&name=ibm_*")
    /// ```
    #[pyo3(signature = (filters=None))]
    pub fn resources(&self, filters: Option<&str>) -> PyResult<Vec<PyQuantumResource>> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.inner.resources(filters.map(str::to_string)).await });
        match result {
            Ok(resources) => Ok(resources
                .into_iter()
                .map(PyQuantumResource::from_inner)
                .collect()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Returns the least busy available quantum resource, optionally filtered.
    ///
    /// Equivalent to `resources(filters)[0]` but returns `None` if no resources match.
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// resource = provider.least_busy()
    /// resource = provider.least_busy("num_qubits=127&status=online")
    /// ```
    #[pyo3(signature = (filters=None))]
    pub fn least_busy(&self, filters: Option<&str>) -> PyResult<Option<PyQuantumResource>> {
        crate::common::initialize();
        let result = self
            .rt
            .block_on(async { self.inner.least_busy(filters.map(str::to_string)).await });
        match result {
            Ok(resource) => Ok(resource.map(PyQuantumResource::from_inner)),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }
}

/// A Python module implemented in Rust.
#[pymodule(name = "_core")]
fn qrmi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQuantumResource>()?;
    m.add_class::<ResourceType>()?;
    m.add_class::<crate::models::TaskStatus>()?;
    m.add_class::<crate::models::Payload>()?;
    m.add_class::<crate::models::Target>()?;
    m.add_class::<crate::models::TaskResult>()?;
    m.add_class::<PyResourceProvider>()?;
    Ok(())
}
define_stub_info_gatherer!(stub_info);
