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
use crate::ibm::IBMQuantumSystemProvider;
use crate::ibm::{IBMQiskitRuntimeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::{Payload, ResourceDef, Target, TaskResult, TaskStatus};
use crate::pasqal::PasqalCloud;
use crate::pasqal::PasqalLocal;
use crate::QuantumResource;
use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, derive::*};
use tokio::runtime::Runtime;

#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
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
    // `ManuallyDrop`, not a plain `Runtime`, so `Drop` below can take
    // ownership and call `shutdown_background()` instead of letting the
    // field's own destructor run. Existing `self.rt.block_on(...)` call
    // sites are unaffected: `ManuallyDrop<T>` derefs to `T` transparently.
    rt: std::mem::ManuallyDrop<Runtime>,
}

impl Drop for PyQuantumResource {
    fn drop(&mut self) {
        // `Runtime`'s own `Drop` blocks the current thread indefinitely
        // until every spawned task finishes. That is a problem here
        // specifically: Python drops objects while holding the GIL, and
        // if a worker thread needs the GIL to log something (e.g. an
        // in-flight HTTP request logged at `RUST_LOG=trace`) before it can
        // finish, that worker waits for the GIL forever while this thread
        // waits for that worker forever. `shutdown_background()` discards
        // the runtime without waiting for anything, which avoids the
        // deadlock entirely (in exchange for not waiting for in-flight
        // background work to finish cleanly on drop).
        //
        // SAFETY: `self.rt` is only ever taken here, in `Drop::drop`,
        // which runs at most once per instance.
        let rt = unsafe { std::mem::ManuallyDrop::take(&mut self.rt) };
        rt.shutdown_background();
    }
}

impl PyQuantumResource {
    /// Internal constructor used by `PyResourceProvider::backends()`.
    pub(crate) fn from_inner(qrmi: Box<dyn QuantumResource + Send + Sync>) -> Self {
        Self {
            qrmi,
            rt: std::mem::ManuallyDrop::new(
                Runtime::new().expect("Failed to create a new tokio runtime."),
            ),
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
            rt: std::mem::ManuallyDrop::new(
                Runtime::new().expect("Failed to create a new tokio runtime."),
            ),
        })
    }

    fn is_accessible(&mut self, py: Python<'_>) -> PyResult<bool> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.is_accessible().await }));
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn resource_id(&mut self, py: Python<'_>) -> PyResult<String> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.resource_id().await }));
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn resource_type(&mut self, py: Python<'_>) -> PyResult<ResourceType> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.resource_type().await }));
        match result {
            Ok(v) => Ok(match v {
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

    fn acquire(&mut self, py: Python<'_>) -> PyResult<String> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.acquire().await }));
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn release(&mut self, py: Python<'_>, id: &str) -> PyResult<()> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.release(id).await }));
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_start(&mut self, py: Python<'_>, payload: Payload) -> PyResult<String> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.qrmi.task_start(payload).await })
        });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_stop(&mut self, py: Python<'_>, task_id: &str) -> PyResult<()> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.qrmi.task_stop(task_id).await })
        });
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_status(&mut self, py: Python<'_>, task_id: &str) -> PyResult<TaskStatus> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.qrmi.task_status(task_id).await })
        });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_result(&mut self, py: Python<'_>, task_id: &str) -> PyResult<TaskResult> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.qrmi.task_result(task_id).await })
        });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn task_logs(&mut self, py: Python<'_>, task_id: &str) -> PyResult<String> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.qrmi.task_logs(task_id).await })
        });
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn target(&mut self, py: Python<'_>) -> PyResult<Target> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.target().await }));
        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    fn metadata(&mut self, py: Python<'_>) -> PyResult<std::collections::HashMap<String, String>> {
        crate::common::initialize();
        let result = py.detach(|| self.rt.block_on(async { self.qrmi.metadata().await }));
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// ResourceDef Python bindings
// ---------------------------------------------------------------------------

/// Python wrapper for a QRMI resource definition.
#[gen_stub_pyclass]
#[pyclass(skip_from_py_object)]
#[pyo3(name = "ResourceDef")]
#[derive(Clone)]
pub struct PyResourceDef {
    pub(crate) inner: ResourceDef,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyResourceDef {
    /// Resource name.
    #[getter]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Resource type.
    #[getter]
    pub fn resource_type(&self) -> ResourceType {
        match self.inner.r#type {
            crate::models::ResourceType::IBMQuantumSystem => ResourceType::IBMQuantumSystem,
            crate::models::ResourceType::QiskitRuntimeService => {
                ResourceType::IBMQiskitRuntimeService
            }
            crate::models::ResourceType::PasqalCloud => ResourceType::PasqalCloud,
            crate::models::ResourceType::PasqalLocal => ResourceType::PasqalLocal,
            crate::models::ResourceType::AliceBobFelis => ResourceType::AliceBobFelis,
            crate::models::ResourceType::IQMServer => ResourceType::IQMServer,
        }
    }

    /// Whether this resource definition is dynamic.
    #[getter]
    pub fn is_dynamic(&self) -> bool {
        self.inner.is_dynamic()
    }

    /// Environment variables for this resource.
    #[getter]
    pub fn environment(&self) -> std::collections::HashMap<String, String> {
        self.inner.environment.clone()
    }
}

// ---------------------------------------------------------------------------
// ResourceProvider Python bindings
// ---------------------------------------------------------------------------

/// Python wrapper for `ResourceProvider`.
///
/// # Example (Python)
///
/// ```python
/// from qrmi import Config, ResourceProvider, ResourceType
///
/// config = Config.load("/path/to/qrmi_config.json")
/// resource_def = config.resource_map["ibm_inst1"]
///
/// provider = ResourceProvider(ResourceType.IBMQiskitRuntimeService, resource_def.environment)
/// resources = provider.resources()
/// resources = provider.resources("num_qubits=127&name=ibm_*&status=online")
/// resource  = provider.least_busy()
///
/// for r in resources:
///     print(r.resource_id())
/// ```
#[gen_stub_pyclass]
#[pyclass]
#[pyo3(name = "ResourceProvider")]
pub struct PyResourceProvider {
    inner: Box<dyn crate::ResourceProvider>,
    // See `PyQuantumResource`'s `rt` field and `Drop` impl for why this is
    // `ManuallyDrop` rather than a plain `Runtime`.
    rt: std::mem::ManuallyDrop<Runtime>,
}

impl Drop for PyResourceProvider {
    fn drop(&mut self) {
        // See `PyQuantumResource`'s `Drop` impl for why.
        //
        // SAFETY: `self.rt` is only ever taken here, in `Drop::drop`,
        // which runs at most once per instance.
        let rt = unsafe { std::mem::ManuallyDrop::take(&mut self.rt) };
        rt.shutdown_background();
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyResourceProvider {
    /// Constructs a new provider from a resource type and environment variable map.
    ///
    /// Currently supported resource types:
    /// - `ResourceType.IBMQiskitRuntimeService`
    /// - `ResourceType.IBMQuantumSystem`
    #[new]
    pub fn new(
        resource_type: ResourceType,
        environment: std::collections::HashMap<String, String>,
    ) -> PyResult<Self> {
        crate::common::initialize();
        let inner: Box<dyn crate::ResourceProvider> = match resource_type {
            ResourceType::IBMQiskitRuntimeService => {
                match IBMQiskitRuntimeServiceProvider::new(&environment) {
                    Ok(p) => Box::new(p),
                    Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            }
            ResourceType::IBMQuantumSystem => match IBMQuantumSystemProvider::new(&environment) {
                Ok(p) => Box::new(p),
                Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            },
            _ => {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Unsupported resource type for dynamic resource discovery",
                ))
            }
        };
        Ok(Self {
            inner,
            rt: std::mem::ManuallyDrop::new(
                Runtime::new().expect("Failed to create a new tokio runtime."),
            ),
        })
    }

    /// Returns available quantum resources, optionally filtered.
    ///
    /// # Arguments
    ///
    /// * `filters` - Filter string of the form `key=value&key=value`, or `None`.
    ///
    /// Filter specifications (constraints) are defined by each resource provider's implementation.
    /// Results are expected to be sorted in least-busy order.
    ///
    /// # Example (Python)
    ///
    /// ```python
    /// resources = provider.resources()
    /// resources = provider.resources("num_qubits=127&name=ibm_*")
    /// ```
    #[pyo3(signature = (filters=None))]
    pub fn resources(
        &self,
        py: Python<'_>,
        filters: Option<&str>,
    ) -> PyResult<Vec<PyQuantumResource>> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.inner.resources(filters.map(str::to_string)).await })
        });
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
    pub fn least_busy(
        &self,
        py: Python<'_>,
        filters: Option<&str>,
    ) -> PyResult<Option<PyQuantumResource>> {
        crate::common::initialize();
        let result = py.detach(|| {
            self.rt
                .block_on(async { self.inner.least_busy(filters.map(str::to_string)).await })
        });
        match result {
            Ok(resource) => Ok(resource.map(PyQuantumResource::from_inner)),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }
}

// ---------------------------------------------------------------------------
// Config Python bindings
// ---------------------------------------------------------------------------

/// Python wrapper for QRMI configuration.
///
/// # Example (Python)
///
/// ```python
/// from qrmi import Config, ResourceProvider
///
/// config = Config.load("/path/to/qrmi_config.json")
///
/// # Iterate over all resource definitions
/// for name, resource_def in config.resource_map.items():
///     print(f"{name}: is_dynamic={resource_def.is_dynamic}")
///     if resource_def.is_dynamic:
///         provider = ResourceProvider(resource_def.resource_type, resource_def.environment)
///         resources = provider.resources()
/// ```
#[gen_stub_pyclass]
#[pyclass]
#[pyo3(name = "Config")]
pub struct PyConfig {
    inner: crate::models::Config,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyConfig {
    /// Loads a QRMI config file.
    #[staticmethod]
    pub fn load(path: &str) -> PyResult<Self> {
        match crate::models::Config::load(path) {
            Ok(inner) => Ok(Self { inner }),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
        }
    }

    /// Returns a dict mapping resource name to ResourceDef.
    #[getter]
    pub fn resource_map(&self) -> std::collections::HashMap<String, PyResourceDef> {
        self.inner
            .resource_map
            .iter()
            .map(|(k, v)| (k.clone(), PyResourceDef { inner: v.clone() }))
            .collect()
    }
}

/// Bridges QRMI's `log` records into Python's `logging` module.
///
/// Registered once via `crate::common::set_log_sink` from the
/// `#[pymodule]` init function below, using the same `LogSink` extension
/// point `cext::qrmi_log_callback_set` also adapts a C callback into,
/// rather than installing a second, separate logging backend. This avoids
/// ever having two loggers registered in the same process (the `log`
/// crate only allows one), which matters because `cext` is always
/// compiled in alongside `pyext` when the `pyo3` feature is enabled.
///
/// The `env_logger` filter in `common::initialize` (default `RUST_LOG`
/// level: `warn`) still runs first and decides what reaches this sink at
/// all; what changes here is only where an accepted record goes
/// afterwards. A record that passes that filter is forwarded to
/// `logging.getLogger(target)`, so Python-side configuration
/// (`logging.basicConfig()`, per-logger levels, handlers) governs it from
/// there same as any other Python log record.
///
/// This is a plain Rust closure over `common::LogSink` -- no C ABI, no raw
/// pointers, no `unsafe`. `common.rs` doesn't know or care that this
/// particular sink happens to call into Python; that's this module's
/// business alone. Compare `cext::qrmi_log_callback_set`, which adapts a
/// C function pointer into the same `LogSink` shape at its own boundary.
///
/// Uses `Python::try_attach`, not `Python::attach`: a log record can be
/// emitted from a `__del__` running during CPython interpreter
/// finalization (`Py_FinalizeEx`) -- attempting to (re-)acquire the GIL
/// in that window is a known hazard (documented on `Python::attach`
/// itself, and the subject of e.g. PyO3#5317: repeatedly attaching during
/// finalization has hung or segfaulted). `try_attach` returns `None`
/// instead in that case; we simply drop the record rather than risk
/// hanging the interpreter shutdown to deliver a log line.
fn python_log_sink(level: log::Level, target: &str, message: &str) {
    // Python's `logging` module level numbers (see Python's `logging`
    // module docs / `Lib/logging/__init__.py`: CRITICAL=50, ERROR=40,
    // WARNING=30, INFO=20, DEBUG=10, NOTSET=0). Hardcoded rather than
    // looked up via `logging.ERROR` etc. because these values are part of
    // `logging`'s documented, long-stable public API and are very unlikely
    // to change.
    // TRACE has no Python equivalent, so it is folded into DEBUG.
    let py_level: i32 = match level {
        log::Level::Error => 40,
        log::Level::Warn => 30,
        log::Level::Info => 20,
        log::Level::Debug | log::Level::Trace => 10,
    };

    Python::try_attach(|py| {
        let Ok(logging) = py.import("logging") else {
            return;
        };
        let Ok(logger) = logging.call_method1("getLogger", (target,)) else {
            return;
        };
        let _ = logger.call_method1("log", (py_level, message));
    });
}

/// Registers a user-supplied Python callable as the destination for
/// QRMI's `log` records, replacing whatever sink is currently active --
/// including the default one installed at import time (see
/// `python_log_sink` above). This is the Python-facing equivalent of the
/// C API's `qrmi_log_callback_set`.
///
/// `callback` is called as `callback(level, target, message)`, all three
/// arguments `str`. `level` is one of `"ERROR"`, `"WARN"`, `"INFO"`,
/// `"DEBUG"`, `"TRACE"`. Pass `None` to clear it, which reverts to plain
/// stderr output -- the same fallback the C API's `NULL` reverts to --
/// rather than back to the built-in `logging`-forwarding sink. Once you
/// take over logging, where it goes (including back to `logging`, if
/// that's what you want) is entirely your callback's responsibility.
///
/// # Notes
///
/// - `callback` may run on any thread, including a tokio worker thread
///   unrelated to whichever Python thread called into QRMI. This is safe
///   as long as every blocking QRMI call keeps releasing the GIL for its
///   duration (see the `py.detach(...)` calls throughout this module) --
///   that is what lets a worker thread acquire the GIL here without
///   deadlocking against a QRMI call that is still holding it.
/// - `callback` runs synchronously and holds the GIL while it runs; a slow
///   callback delays whichever QRMI call happened to trigger the log
///   record it's handling. Keep it fast -- e.g. hand off to a queue --
///   if you need to do anything slow with a record.
/// - If `callback` raises, the exception is printed (as an unhandled
///   exception would be) and otherwise discarded; it cannot propagate
///   back into the Rust code that logged in the first place.
/// - Like `python_log_sink`, this uses `Python::try_attach`: if a record
///   is emitted while the interpreter is finalizing, `callback` is simply
///   not called for it rather than risking the same hang/segfault hazard
///   `Python::attach` has in that window.
#[gen_stub_pyfunction]
#[pyfunction]
fn set_log_callback(callback: Option<Py<PyAny>>) -> PyResult<()> {
    crate::common::initialize();
    let sink: Option<crate::common::LogSink> = callback.map(|callback| {
        let sink: crate::common::LogSink =
            std::sync::Arc::new(move |level: log::Level, target: &str, message: &str| {
                Python::try_attach(|py| {
                    if let Err(err) = callback.call1(py, (level.as_str(), target, message)) {
                        err.print(py);
                    }
                });
            });
        sink
    });
    crate::common::set_log_sink(sink)
        .map_err(|()| pyo3::exceptions::PyRuntimeError::new_err("Failed to set log callback"))
}

/// Returns the version of this QRMI library as a semantic version string.
///
/// Callers can compare this against the version they were built against to detect
/// incompatibilities at runtime, for example after a system upgrade replaced the
/// native extension underneath them.
#[gen_stub_pyfunction]
#[pyfunction]
fn get_version() -> &'static str {
    crate::get_version()
}

/// A Python module implemented in Rust.
#[pymodule(name = "_core")]
fn qrmi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    crate::common::initialize();
    let _ = crate::common::set_log_sink(Some(std::sync::Arc::new(python_log_sink)));

    m.add_function(wrap_pyfunction!(set_log_callback, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    m.add_class::<PyQuantumResource>()?;
    m.add_class::<ResourceType>()?;
    m.add_class::<crate::models::TaskStatus>()?;
    m.add_class::<crate::models::Payload>()?;
    m.add_class::<crate::models::Target>()?;
    m.add_class::<crate::models::TaskResult>()?;
    m.add_class::<PyResourceDef>()?;
    m.add_class::<PyResourceProvider>()?;
    m.add_class::<PyConfig>()?;
    Ok(())
}
define_stub_info_gatherer!(stub_info);
