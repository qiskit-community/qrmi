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
            IbmError::UnknownProgramId(_) => QrmiErrorKind::UnknownProgramId,
            IbmError::InvalidSessionMode(_) => QrmiErrorKind::InvalidValue,
        }
    }
}
