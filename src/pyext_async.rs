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

//! asyncio-native counterpart to [`crate::pyext`], built on
//! `pyo3-async-runtimes`.
//!
//! Cross-cutting pieces (`to_py_err`, the `QrmiError_` exception
//! hierarchy, `ResourceType` and its conversions) live in
//! [`crate::pyext::common`], `pub(crate)`, specifically so this module can
//! reuse them instead of duplicating them.

use std::sync::Arc;

use pyo3::prelude::*;
use pyo3_stub_gen::derive::*;
use tokio::sync::Mutex;

use crate::models::Payload;
use crate::pyext::common::{to_py_err, ResourceType};
use crate::QuantumResource;

/// asyncio-native counterpart to `pyext::PyQuantumResource`.
///
/// Every I/O method returns a Python coroutine (via
/// `pyo3_async_runtimes::tokio::future_into_py`) instead of blocking the
/// calling thread, so this is meant to be awaited from an asyncio event
/// loop: `await resource.is_accessible()`, etc.
///
/// Unlike `PyQuantumResource`, this type does not own a per-instance
/// `tokio::runtime::Runtime`: `future_into_py` schedules work on the single
/// process-wide runtime `pyo3-async-runtimes` lazily creates on first use
/// (`pyo3_async_runtimes::tokio::get_runtime()`), so the two bindings can
/// coexist without either owning the other's runtime.
#[gen_stub_pyclass]
#[pyclass]
#[pyo3(name = "AsyncQuantumResource")]
pub struct PyAsyncQuantumResource {
    // `Arc<Mutex<..>>`, not a bare `Box`, because every method below has to
    // hand `future_into_py` a `'static` future: the future must own (or
    // share ownership of) the resource rather than borrow `&mut self`,
    // since `self` does not outlive the call that creates the coroutine.
    // `tokio::sync::Mutex`, not `std::sync::Mutex`, because its guard is
    // `Send`, which is required since the guard is held across the `.await`
    // points inside the trait's own async methods.
    qrmi: Arc<Mutex<Box<dyn QuantumResource + Send + Sync>>>,
}

impl PyAsyncQuantumResource {
    /// Internal constructor used by `PyAsyncQRMIService::create()`.
    pub(crate) fn from_inner(qrmi: Box<dyn QuantumResource + Send + Sync>) -> Self {
        Self {
            qrmi: Arc::new(Mutex::new(qrmi)),
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyAsyncQuantumResource {
    #[new]
    pub fn new(resource_id: &str, resource_type: ResourceType) -> PyResult<Self> {
        crate::common::initialize();
        let qrmi = crate::common::create_resource(&resource_type.into(), resource_id)
            .map_err(to_py_err)?;
        Ok(Self::from_inner(qrmi))
    }

    fn is_accessible<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.is_accessible().await.map_err(to_py_err)
        })
    }

    fn resource_id<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.resource_id().await.map_err(to_py_err)
        })
    }

    fn resource_type<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.resource_type()
                .await
                .map(ResourceType::from)
                .map_err(to_py_err)
        })
    }

    fn acquire<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.acquire().await.map_err(to_py_err)
        })
    }

    fn release<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.release(&id).await.map_err(to_py_err)
        })
    }

    fn task_start<'py>(&self, py: Python<'py>, payload: Payload) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.task_start(payload).await.map_err(to_py_err)
        })
    }

    fn task_stop<'py>(&self, py: Python<'py>, task_id: String) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.task_stop(&task_id).await.map_err(to_py_err)
        })
    }

    fn task_status<'py>(&self, py: Python<'py>, task_id: String) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.task_status(&task_id).await.map_err(to_py_err)
        })
    }

    fn task_result<'py>(&self, py: Python<'py>, task_id: String) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.task_result(&task_id).await.map_err(to_py_err)
        })
    }

    fn task_logs<'py>(&self, py: Python<'py>, task_id: String) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.task_logs(&task_id).await.map_err(to_py_err)
        })
    }

    fn target<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            qrmi.target().await.map_err(to_py_err)
        })
    }

    fn metadata<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let qrmi = self.qrmi.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut qrmi = qrmi.lock().await;
            Ok::<_, PyErr>(qrmi.metadata().await)
        })
    }
}
