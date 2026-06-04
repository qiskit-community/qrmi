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

//! Shared error type for all controllers.

/// Controller error - implements `std::error::Error` (via thiserror) so it can
/// be used as the reconciler error type with `kube::runtime::Controller`.
/// It is `Clone` so it satisfies the `Controller::run` bound.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct ControllerError {
    pub message: String,
}

impl From<anyhow::Error> for ControllerError {
    fn from(e: anyhow::Error) -> Self {
        Self {
            message: format!("{e:#}"),
        }
    }
}

impl From<kube::Error> for ControllerError {
    fn from(e: kube::Error) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, ControllerError>;
