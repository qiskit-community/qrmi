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

use crate::ibm::{IBMDirectAccess, IBMQiskitRuntimeService};
use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::pasqal::PasqalCloud;
use crate::QuantumResource;
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};
use tokio::runtime::Runtime;

#[pyclass(eq, eq_int, hash, frozen)]
#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    IBMDirectAccess,
    IBMQiskitRuntimeService,
    PasqalCloud,
}

#[gen_stub_pyclass]
#[pyclass]
#[pyo3(name = "QuantumResource")]
pub struct PyQuantumResource {
    qrmi: Box<dyn QuantumResource + Send + Sync>,
    rt: Runtime,
}
#[gen_stub_pymethods]
#[pymethods]
impl PyQuantumResource {
    #[new]
    pub fn new(resource_id: &str, resource_type: ResourceType) -> Self {
        let qrmi: Box<dyn QuantumResource + Send + Sync> = match resource_type {
            ResourceType::IBMDirectAccess => Box::new(IBMDirectAccess::new(resource_id)),
            ResourceType::IBMQiskitRuntimeService => {
                Box::new(IBMQiskitRuntimeService::new(resource_id))
            }
            ResourceType::PasqalCloud => Box::new(PasqalCloud::new(resource_id)),
        };

        Self {
            qrmi,
            rt: Runtime::new().unwrap(),
        }
    }

    fn is_accessible(&mut self) -> PyResult<bool> {
        let result = self.rt.block_on(async { self.qrmi.is_accessible().await });
        Ok(result)
    }

    fn acquire(&mut self) -> PyResult<String> {
        let result = self.rt.block_on(async { self.qrmi.acquire().await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn release(&mut self, id: &str) -> PyResult<()> {
        let result = self.rt.block_on(async { self.qrmi.release(id).await });
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_start(&mut self, payload: Payload) -> PyResult<String> {
        let result = self
            .rt
            .block_on(async { self.qrmi.task_start(payload).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_stop(&mut self, task_id: &str) -> PyResult<()> {
        let result = self
            .rt
            .block_on(async { self.qrmi.task_stop(task_id).await });
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_status(&mut self, task_id: &str) -> PyResult<TaskStatus> {
        let result = self
            .rt
            .block_on(async { self.qrmi.task_status(task_id).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_result(&mut self, task_id: &str) -> PyResult<TaskResult> {
        let result = self
            .rt
            .block_on(async { self.qrmi.task_result(task_id).await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn target(&mut self) -> PyResult<Target> {
        let result = self.rt.block_on(async { self.qrmi.target().await });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn metadata(&mut self) -> PyResult<std::collections::HashMap<String, String>> {
        let result = self.rt.block_on(async { self.qrmi.metadata().await });
        Ok(result)
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
    Ok(())
}
define_stub_info_gatherer!(stub_info);
