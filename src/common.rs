// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal, UKRI-STFC (Hartree Centre) 2025 - 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
use anyhow::{bail, Result};
use std::io::Write;
use std::sync::{Arc, Once, RwLock};

use crate::alice_bob::AliceBobFelis;
use crate::ibm::{IBMQiskitRuntimeService, IBMQuantumComputeService, IBMQuantumSystem};
use crate::iqm::IQMServer;
use crate::models::ResourceType;
use crate::pasqal::{PasqalCloud, PasqalLocal};
use crate::QuantumResource;

static INIT: Once = Once::new();
static LOG_SINK: RwLock<Option<LogSink>> = RwLock::new(None);

/// A destination for `log` records, in plain Rust terms: no C ABI, no
/// pointers. `cext` and `pyext` each register their own adapter here --
/// `cext`'s wraps a C function pointer and does the `CString` conversion
/// at dispatch time (see `cext::qrmi_log_callback_set`); `pyext`'s calls
/// straight into Python's `logging` module, or into a user-supplied
/// Python callable (see `pyext::set_log_callback`). Neither shape leaks
/// into this module.
///
/// `Arc` rather than `Box`: `dispatch_to_sink` below clones this handle
/// and releases `LOG_SINK`'s lock *before* calling it, since a sink may
/// call arbitrary code (a user's own Python callback in particular) that
/// could be slow or could itself log again on the same thread --
/// `std::sync::RwLock` does not guarantee safe recursive read-locking, so
/// still holding the lock across that call would risk a deadlock or worse.
pub(crate) type LogSink = Arc<dyn Fn(log::Level, &str, &str) + Send + Sync>;

/// Registers `sink` as the destination for future `log` records, replacing
/// any previously registered sink. `None` clears it, reverting to plain
/// stderr output.
pub(crate) fn set_log_sink(sink: Option<LogSink>) -> Result<(), ()> {
    LOG_SINK
        .write()
        .map(|mut current| *current = sink)
        .map_err(|_| ())
}

fn dispatch_to_sink(record: &log::Record<'_>) -> bool {
    let sink = {
        let Ok(guard) = LOG_SINK.read() else {
            return false;
        };
        guard.clone()
    };
    let Some(sink) = sink else {
        return false;
    };
    sink(record.level(), record.target(), &record.args().to_string());
    true
}

/// Called once before using the API library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.
///
/// Uses `try_init()`, not `init()`: `init()` panics if another library in
/// the same process already registered a `log` logger first (only one can
/// ever be registered process-wide), and -- because that failure happens
/// inside this `Once::call_once` closure -- a panic here poisons `INIT`,
/// so every later call to `initialize()` (this runs at the top of nearly
/// every QRMI entry point) panics too, for the rest of the process's
/// lifetime. `try_init()` fails quietly instead: if we lose that race,
/// QRMI's `log` output is simply not registered (records fall through to
/// whichever logger did win), rather than taking down every subsequent
/// call into QRMI.
pub(crate) fn initialize() {
    INIT.call_once(|| {
        let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format(|buf, record| {
                if dispatch_to_sink(record) {
                    Ok(())
                } else {
                    writeln!(
                        buf,
                        "[{} {} {}] {}",
                        buf.timestamp(),
                        record.level(),
                        record.target(),
                        record.args()
                    )
                }
            })
            .try_init();
    });
}

/// Reads `name` (falling back to `legacy_name` if `name` is unset) and
/// splits it into a list using the delimiter specified by the
/// `QRMI_LIST_DELIMITER` environment variable (default: `,`).
///
/// Returns an empty `Vec` if the resolved value is an empty string, and an
/// error if neither `name` nor `legacy_name` is set.
pub(crate) fn job_env_list(name: &str, legacy_name: &str) -> Result<Vec<String>> {
    let values = match std::env::var(name).or_else(|_| std::env::var(legacy_name)) {
        Ok(v) => v,
        Err(_) => {
            bail!(
                "The environment variable `{}` is not set and as such configuration \
                 could not be loaded.",
                name
            );
        }
    };
    if values.is_empty() {
        return Ok(Vec::new());
    }
    let sep = std::env::var("QRMI_LIST_DELIMITER").unwrap_or_else(|_| ",".to_string());
    Ok(values.split(sep.as_str()).map(str::to_string).collect())
}

/// Returns the QPU resources and types assigned to the current job.
///
/// Reads `QRMI_JOB_QPU_RESOURCES`/`QRMI_JOB_QPU_TYPES`, falling back to the
/// legacy `SLURM_JOB_QPU_RESOURCES`/`SLURM_JOB_QPU_TYPES` names. Fails if the
/// two lists have different lengths.
pub(crate) fn get_job_qpu_resources_and_types() -> Result<(Vec<String>, Vec<String>)> {
    let qpus = job_env_list("QRMI_JOB_QPU_RESOURCES", "SLURM_JOB_QPU_RESOURCES")?;
    let qpu_types = job_env_list("QRMI_JOB_QPU_TYPES", "SLURM_JOB_QPU_TYPES")?;
    if qpus.len() != qpu_types.len() {
        bail!(
            "Inconsistent specifications of QPU resources and types. {:?} vs {:?}",
            qpus,
            qpu_types
        );
    }
    if qpus.is_empty() {
        log::warn!("No QPU resources or types specified.");
    }
    Ok((qpus, qpu_types))
}

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
