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

use crate::error::{QrmiError, QrmiErrorKind};
use http::StatusCode;
use pasqal_cloud_api::{ApiError, AuthError};
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

/// What a Pasqal Cloud request was about, to tell a missing device from a
/// missing task when the API answers 404.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceKind {
    Device,
    Task,
}

/// Converts an error from the Pasqal Cloud client into the `QrmiError`
/// variant matching its HTTP status, so callers get a specific error kind
/// instead of `Other`. Errors without a status keep their full chain in
/// `QrmiError::Other`.
pub(crate) fn classify(err: anyhow::Error, resource_kind: ResourceKind) -> QrmiError {
    if err.downcast_ref::<AuthError>().is_some() {
        return QrmiError::AuthenticationFailed(err.to_string());
    }
    let Some(api_err) = err.downcast_ref::<ApiError>() else {
        return QrmiError::Other(err);
    };
    let body = api_err.body.clone();
    match (api_err.status, resource_kind) {
        (StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY, _) => {
            QrmiError::InvalidInput(body)
        }
        (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN, _) => {
            QrmiError::AuthenticationFailed(body)
        }
        (StatusCode::NOT_FOUND, ResourceKind::Device) => QrmiError::ResourceNotFound(body),
        (StatusCode::NOT_FOUND, ResourceKind::Task) => QrmiError::TaskNotFound(body),
        _ => QrmiError::Other(err),
    }
}
