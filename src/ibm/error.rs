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

//! Error conditions specific to the IBM backends, as opposed to
//! [`crate::error::QrmiError`], which only knows about conditions that are
//! generic across every vendor. Values of this type reach callers wrapped in
//! `QrmiError::Ibm(_)` via `?` (see the `#[from]` on that variant).

use crate::error::QrmiErrorKind;
use thiserror::Error;

/// Errors that only make sense in the context of IBM's backends (IBM Quantum
/// System and IBM Qiskit Runtime Service): they name IBM-specific concepts
/// (program IDs, session modes) that the framework-level [`crate::QrmiError`]
/// has no business knowing about.
#[derive(Error, Debug)]
pub enum IbmError {
    /// A `program_id` string did not match any program/primitive this
    /// backend recognizes (e.g. `sampler`, `estimator`, `noiselearner`).
    #[error("unknown program ID: {0}")]
    UnknownProgramId(String),

    /// A `session_mode` value was not one of the modes Qiskit Runtime
    /// supports.
    #[error("invalid session mode: {0}")]
    InvalidSessionMode(String),
}

impl IbmError {
    pub(crate) fn kind(&self) -> QrmiErrorKind {
        match self {
            IbmError::UnknownProgramId(_) => QrmiErrorKind::InvalidValue,
            IbmError::InvalidSessionMode(_) => QrmiErrorKind::InvalidValue,
        }
    }
}

/// Maps errors from the `quantum_system_api` crate (used by
/// [`crate::ibm::IBMQuantumSystem`] and [`crate::ibm::IBMQuantumSystemProvider`])
/// onto [`crate::QrmiError`]. A few conditions that crate can distinguish
/// map onto specific `QrmiError` variants (a missing backend or job, a
/// rejected credential); everything else falls back to
/// [`crate::QrmiError::Other`], still carrying the original error as
/// `source` so the message and cause chain aren't lost.
///
/// This is a plain `impl From`, not a `#[from]` variant on `IbmError` or
/// `QrmiError` -- `quantum_system_api::QuantumSystemError` doesn't represent
/// an IBM-specific *condition* the way `IbmError`'s variants do, it's just
/// the error type of a dependency we're translating, so it doesn't belong
/// as a variant of either enum.
impl From<quantum_system_api::QuantumSystemError> for crate::QrmiError {
    fn from(err: quantum_system_api::QuantumSystemError) -> Self {
        use quantum_system_api::QuantumSystemError as QsError;
        match err {
            QsError::BackendNotFound(msg) => crate::QrmiError::ResourceNotFound(msg),
            QsError::JobNotFound(msg) => crate::QrmiError::TaskNotFound(msg),
            QsError::AuthenticationFailed(msg) => crate::QrmiError::AuthenticationFailed(msg),
            other => crate::QrmiError::Other(anyhow::Error::new(other)),
        }
    }
}
