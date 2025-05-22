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
use crate::pasqal::PasqalCloud;
use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use pyo3::prelude::*;

#[pyclass(eq, eq_int, hash, frozen)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    IBMDirectAccess,
    IBMQiskitRuntimeService,
    PasqalCloud,
}

#[pyclass]
#[pyo3(name = "QuantumResource")]
pub struct PyQuantumResource {
    qrmi: Box<dyn QuantumResource + Send + Sync>,
}
#[pymethods]
impl PyQuantumResource {
    #[new]
    pub fn new(resource_id: &str, resource_type: ResourceType) -> Self {

        let qrmi: Box<dyn QuantumResource + Send + Sync>;
        match resource_type {
            ResourceType::IBMDirectAccess => {
                qrmi = Box::new(IBMDirectAccess::new(resource_id));
            }
            ResourceType::IBMQiskitRuntimeService => {
                qrmi = Box::new(IBMQiskitRuntimeService::new(resource_id));
            }
            ResourceType::PasqalCloud => {
                qrmi = Box::new(PasqalCloud::new(resource_id));
            }
        }

        Self {
            qrmi,
        }
    }

    fn is_accessible(&mut self) -> PyResult<bool> {
        Ok(self.qrmi.is_accessible())
    }

    fn acquire(&mut self) -> PyResult<String> {
        match self.qrmi.acquire() {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn release(&mut self, id: &str) -> PyResult<()> {
        match self.qrmi.release(id) {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_start(&mut self, payload: Payload) -> PyResult<String> {
        match self.qrmi.task_start(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_stop(&mut self, task_id: &str) -> PyResult<()> {
        match self.qrmi.task_stop(task_id) {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_status(&mut self, task_id: &str) -> PyResult<TaskStatus> {
        match self.qrmi.task_status(task_id) {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_result(&mut self, task_id: &str) -> PyResult<TaskResult> {
        match self.qrmi.task_result(task_id) {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn target(&mut self) -> PyResult<Target> {
        match self.qrmi.target() {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn metadata(&mut self) -> PyResult<std::collections::HashMap<String, String>> {
        Ok(self.qrmi.metadata())
    }
}

/// A Python module implemented in Rust.
#[pymodule] 
fn qrmi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyQuantumResource>()?;
    m.add_class::<ResourceType>()?;
    m.add_class::<crate::models::TaskStatus>()?;
    m.add_class::<crate::models::Payload>()?;
    m.add_class::<crate::models::Target>()?;
    m.add_class::<crate::models::TaskResult>()?;
    Ok(())
}
