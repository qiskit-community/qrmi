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

//! Pieces shared between the synchronous (`pyext`) and asyncio-native
//! (`pyext_async`) Python bindings: the `QrmiError` -> `PyErr` exception
//! hierarchy and its conversion function, and the `ResourceType`
//! conversions between the Rust-native (`crate::models::ResourceType`) and
//! pyo3-facing (`ResourceType`, defined here) enums.
//!
//! Pulled out of `pyext.rs` into its own module specifically so a second
//! binding (`pyext_async`) can reuse it via `pub(crate)` instead of
//! duplicating it, as it originally had to.

use pyo3::prelude::*;
use pyo3_stub_gen::{create_exception, derive::*};

use crate::error::{QrmiError, QrmiErrorKind};

create_exception!(
    qrmi._core,
    QrmiError_,
    pyo3::exceptions::PyRuntimeError,
    "Base class for all QRMI-specific errors. Catching this catches any \
     error QRMI itself raises (as opposed to errors surfaced verbatim from \
     an underlying vendor library)."
);
create_exception!(
    qrmi._core,
    EnvVarNotSetError,
    QrmiError_,
    "A required environment variable was not set."
);
create_exception!(
    qrmi._core,
    ConfigError,
    QrmiError_,
    "A configuration value was missing, could not be parsed, or was \
     otherwise invalid (covers `QrmiError::ParseError`, \
     `QrmiError::MissingConfigKey`, and `QrmiError::InvalidConfig`)."
);
create_exception!(
    qrmi._core,
    UnsupportedResourceTypeError,
    QrmiError_,
    "Dynamic discovery was requested for an unsupported resource type."
);
create_exception!(
    qrmi._core,
    UnsupportedPayloadError,
    QrmiError_,
    "The payload (or a value within it, such as a program ID) is not \
     supported by this backend."
);
create_exception!(
    qrmi._core,
    TaskNotReadyError,
    QrmiError_,
    "The task is not in a state that allows the requested operation \
     (e.g. its result was requested while it is still running)."
);
create_exception!(
    qrmi._core,
    InvalidInputError,
    QrmiError_,
    "A value QRMI was given was invalid, whether QRMI itself rejected it \\
     locally (e.g. a malformed `filters` string, JSON, or UTF-8) or a \\
     vendor's API rejected the resulting request after receiving it."
);
create_exception!(
    qrmi._core,
    ResourceNotFoundError,
    QrmiError_,
    "The named resource (e.g. a backend) does not exist."
);
create_exception!(
    qrmi._core,
    TaskNotFoundError,
    QrmiError_,
    "The named task (e.g. a job) does not exist, or has already been removed."
);
create_exception!(
    qrmi._core,
    AuthenticationFailedError,
    QrmiError_,
    "The request's credentials were missing or rejected by the vendor's API."
);

/// Converts a [`QrmiError`] into the [`PyErr`] subclass matching its kind,
/// so Python code can `except qrmi.TaskNotReadyError` instead of parsing
/// `RuntimeError` message text. See `QrmiError::kind` for the mapping.
///
/// `pub(crate)`, not private, specifically so both `pyext` and
/// `pyext_async` can call it without duplicating this match.
pub(crate) fn to_py_err(err: QrmiError) -> PyErr {
    let msg = err.to_string();
    match err.kind() {
        QrmiErrorKind::EnvVarNotSet => EnvVarNotSetError::new_err(msg),
        QrmiErrorKind::ParseError
        | QrmiErrorKind::MissingConfigKey
        | QrmiErrorKind::InvalidConfig => ConfigError::new_err(msg),
        QrmiErrorKind::UnsupportedResourceType => UnsupportedResourceTypeError::new_err(msg),
        QrmiErrorKind::UnsupportedPayload => UnsupportedPayloadError::new_err(msg),
        QrmiErrorKind::TaskNotReady => TaskNotReadyError::new_err(msg),
        QrmiErrorKind::InvalidInput => InvalidInputError::new_err(msg),
        QrmiErrorKind::ResourceNotFound => ResourceNotFoundError::new_err(msg),
        QrmiErrorKind::TaskNotFound => TaskNotFoundError::new_err(msg),
        QrmiErrorKind::AuthenticationFailed => AuthenticationFailedError::new_err(msg),
        QrmiErrorKind::Other => QrmiError_::new_err(msg),
    }
}

#[pyclass(eq, eq_int, hash, frozen, from_py_object)]
#[gen_stub_pyclass_enum]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    IBMQuantumSystem,
    IBMQiskitRuntimeService,
    IBMQuantumComputeService,
    PasqalCloud,
    PasqalLocal,
    AliceBobFelis,
    IQMServer,
}

impl From<ResourceType> for crate::models::ResourceType {
    fn from(value: ResourceType) -> Self {
        match value {
            ResourceType::IBMQuantumSystem => crate::models::ResourceType::IBMQuantumSystem,
            ResourceType::IBMQiskitRuntimeService => {
                crate::models::ResourceType::QiskitRuntimeService
            }
            ResourceType::IBMQuantumComputeService => {
                crate::models::ResourceType::IBMQuantumComputeService
            }
            ResourceType::PasqalCloud => crate::models::ResourceType::PasqalCloud,
            ResourceType::PasqalLocal => crate::models::ResourceType::PasqalLocal,
            ResourceType::AliceBobFelis => crate::models::ResourceType::AliceBobFelis,
            ResourceType::IQMServer => crate::models::ResourceType::IQMServer,
        }
    }
}

// The reverse direction. Previously this exact match was inlined by hand
// in both `pyext::PyQuantumResource::resource_type` and
// `pyext_async::PyAsyncQuantumResource::resource_type` -- now there is
// exactly one copy, reusable via `.into()` from both.
impl From<crate::models::ResourceType> for ResourceType {
    fn from(value: crate::models::ResourceType) -> Self {
        match value {
            crate::models::ResourceType::IBMQuantumSystem => ResourceType::IBMQuantumSystem,
            crate::models::ResourceType::QiskitRuntimeService => {
                ResourceType::IBMQiskitRuntimeService
            }
            crate::models::ResourceType::IBMQuantumComputeService => {
                ResourceType::IBMQuantumComputeService
            }
            crate::models::ResourceType::PasqalCloud => ResourceType::PasqalCloud,
            crate::models::ResourceType::PasqalLocal => ResourceType::PasqalLocal,
            crate::models::ResourceType::AliceBobFelis => ResourceType::AliceBobFelis,
            crate::models::ResourceType::IQMServer => ResourceType::IQMServer,
        }
    }
}
