// This code is part of Qiskit.
//
// Copyright (C): 2026 UKRI-STFC (Hartree Centre)
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Error conditions specific to the Pasqal backends, as opposed to
//! [`crate::error::QrmiError`], which only knows about conditions that are
//! generic across every vendor. Values of this type reach callers wrapped in
//! `QrmiError::Pasqal(_)` via `?` (see the `#[from]` on that variant).

use crate::error::QrmiErrorKind;
use thiserror::Error;

/// Errors that only make sense in the context of Pasqal's backends: they
/// name Pasqal-specific concepts (device types, CUDA-Q sequence payloads)
/// that the framework-level [`crate::QrmiError`] has no business knowing
/// about.
#[derive(Error, Debug)]
pub enum PasqalError {
    /// The backend name did not match any known Pasqal Cloud device type
    /// (e.g. `FRESNEL`, `EMU_MPS`).
    #[error("{0}")]
    InvalidDeviceType(String),

    /// A CUDA-Q sequence payload could not be parsed as JSON.
    #[error("failed to parse CUDA-Q sequence payload: {0}")]
    InvalidCudaqSequence(String),
}

impl PasqalError {
    pub(crate) fn kind(&self) -> QrmiErrorKind {
        match self {
            PasqalError::InvalidDeviceType(_) => QrmiErrorKind::InvalidInput,
            PasqalError::InvalidCudaqSequence(_) => QrmiErrorKind::InvalidInput,
        }
    }
}
