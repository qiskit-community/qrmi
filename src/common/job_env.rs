// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025-2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
use crate::error::QrmiError;

/// Reads `name` (falling back to `legacy_name` if `name` is unset) and
/// splits it into a list using the delimiter specified by the
/// `QRMI_LIST_DELIMITER` environment variable (default: `,`).
///
/// Returns an empty `Vec` if the resolved value is an empty string, and an
/// error if neither `name` nor `legacy_name` is set.
fn job_env_list(name: &str, legacy_name: &str) -> Result<Vec<String>, QrmiError> {
    let values = match std::env::var(name).or_else(|_| std::env::var(legacy_name)) {
        Ok(v) => v,
        Err(_) => {
            return Err(QrmiError::EnvVarNotSet(name.to_string()));
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
pub(crate) fn get_job_qpu_resources_and_types() -> Result<(Vec<String>, Vec<String>), QrmiError> {
    let qpus = job_env_list("QRMI_JOB_QPU_RESOURCES", "SLURM_JOB_QPU_RESOURCES")?;
    let qpu_types = job_env_list("QRMI_JOB_QPU_TYPES", "SLURM_JOB_QPU_TYPES")?;
    if qpus.len() != qpu_types.len() {
        return Err(QrmiError::InvalidConfig(format!(
            "inconsistent number of QPU resources and types: {qpus:?} vs {qpu_types:?}"
        )));
    }
    if qpus.is_empty() {
        log::warn!("No QPU resources or types specified.");
    }
    Ok((qpus, qpu_types))
}
