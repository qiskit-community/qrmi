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

//! QRMI-specific error type.
//!
//! Vendor backends construct [`QrmiError`] for conditions that are specific to
//! QRMI itself (missing configuration, unsupported payloads, task state, ...).
//! Because [`QrmiError`] implements [`std::error::Error`], it converts to
//! [`anyhow::Error`] automatically via `?`, so callers that still use
//! `anyhow::Result` (the public `QuantumResource` / `ResourceProvider` trait
//! methods) do not need to change.
//!
//! Conditions that only make sense for one vendor (an IBM program ID, a
//! Pasqal device type, ...) do NOT get their own framework-level variant --
//! that would leak vendor domain knowledge into a type every backend shares.
//! Instead each vendor module defines its own error type (e.g.
//! [`crate::ibm::error::IbmError`]), and `QrmiError` holds one onto it per
//! vendor (`QrmiError::Ibm`, `QrmiError::Pasqal`, ...). Adding a new failure
//! mode to an existing vendor never touches this file; only adding a new
//! vendor does.
//!
//! Deliberately NOT present here: a blanket `impl<E: std::error::Error> From<E>
//! for QrmiError`. It's tempting (it would let `?` work on any foreign error
//! type without a `.context()` call), but it would conflict with the `#[from]`
//! on `QrmiError::Ibm` / `QrmiError::Pasqal` below -- Rust doesn't allow two
//! `From<X>` impls for the same target type, and a generic "any Error" impl
//! and a concrete "this specific vendor error" impl for that same type both
//! count. So the small number of genuinely generic foreign errors (JSON,
//! UTF-8) get their own named variant with `#[from]` instead, and everything
//! vendor-API-specific goes through `.context("...")?`, converting to
//! `QrmiError::Other` via `anyhow::Error`.

use thiserror::Error;

/// Errors raised by QRMI itself, as opposed to errors bubbled up from a
/// vendor's HTTP/API client (those are wrapped with [`anyhow::Context`] at
/// the call site instead, since they already carry their own error type).
#[derive(Error, Debug)]
pub enum QrmiError {
    /// A required environment variable was not set.
    #[error("{0} environment variable is not set")]
    EnvVarNotSet(String),

    /// A value read from configuration (usually an environment variable)
    /// could not be parsed into the type it was needed as.
    #[error("failed to parse {name} value {value:?}")]
    ParseError {
        name: String,
        value: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },

    /// [`crate::resource_provider::create_provider`] was called with a
    /// resource type that does not support dynamic discovery.
    #[error("unsupported resource type for dynamic resource discovery: {0}")]
    UnsupportedResourceType(String),

    /// The [`crate::models::Payload`] variant is not supported by this backend.
    #[error("payload type is not supported: {0}")]
    UnsupportedPayload(String),

    /// The task is not in a state that allows the requested operation
    /// (e.g. asking for the result of a task that is still running).
    #[error("unable to retrieve result for task {task_id}: {reason}")]
    TaskNotReady { task_id: String, reason: String },

    /// A required key was missing from a provider's environment variable map
    /// (as opposed to [`QrmiError::EnvVarNotSet`], which is for real OS
    /// environment variables).
    #[error("missing '{0}' in environment map")]
    MissingConfigKey(String),

    /// The named resource (e.g. a backend) does not exist.
    #[error("resource not found: {0}")]
    ResourceNotFound(String),

    /// The named task (e.g. a job) does not exist, or has already been
    /// removed.
    #[error("task not found: {0}")]
    TaskNotFound(String),

    /// The request's credentials were missing or rejected by the vendor's
    /// API.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// A `filters` string passed to [`crate::ResourceProvider::resources`] was
    /// malformed or contained an invalid value.
    #[error("invalid filter: {0}")]
    InvalidFilter(String),

    /// A payload, or a piece of it, was not valid JSON.
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// A response body was expected to be valid UTF-8 text and wasn't.
    #[error("invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    /// A condition specific to IBM's backends. See
    /// [`crate::ibm::error::IbmError`].
    #[error(transparent)]
    Ibm(#[from] crate::ibm::error::IbmError),

    /// A condition specific to Pasqal's backends. See
    /// [`crate::pasqal::error::PasqalError`].
    #[error(transparent)]
    Pasqal(#[from] crate::pasqal::error::PasqalError),

    /// Catch-all for errors that don't need their own variant (yet). Any
    /// `anyhow::Error` converts into this via `?` or `.into()`.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl QrmiError {
    /// A stable, C/Python-friendly tag for this error's variant, independent
    /// of the human-readable message. `cext.rs` maps this to a numeric
    /// `QrmiReturnCode`; `pyext.rs` maps it to a specific Python exception
    /// class. Kept as a method here (rather than duplicated match statements
    /// at each binding) so a new `QrmiError` variant only needs a tag added
    /// in one place.
    pub fn kind(&self) -> QrmiErrorKind {
        match self {
            QrmiError::EnvVarNotSet(_) => QrmiErrorKind::EnvVarNotSet,
            QrmiError::ParseError { .. } => QrmiErrorKind::ParseError,
            QrmiError::UnsupportedResourceType(_) => QrmiErrorKind::UnsupportedResourceType,
            QrmiError::UnsupportedPayload(_) => QrmiErrorKind::UnsupportedPayload,
            QrmiError::TaskNotReady { .. } => QrmiErrorKind::TaskNotReady,
            QrmiError::MissingConfigKey(_) => QrmiErrorKind::MissingConfigKey,
            QrmiError::ResourceNotFound(_) => QrmiErrorKind::ResourceNotFound,
            QrmiError::TaskNotFound(_) => QrmiErrorKind::TaskNotFound,
            QrmiError::AuthenticationFailed(_) => QrmiErrorKind::AuthenticationFailed,
            QrmiError::InvalidFilter(_) => QrmiErrorKind::InvalidFilter,
            QrmiError::InvalidJson(_) => QrmiErrorKind::InvalidValue,
            QrmiError::InvalidUtf8(_) => QrmiErrorKind::InvalidValue,
            QrmiError::Ibm(e) => e.kind(),
            QrmiError::Pasqal(e) => e.kind(),
            QrmiError::Other(_) => QrmiErrorKind::Other,
        }
    }
}

/// Variant tag for [`QrmiError`] (including vendor-specific errors nested
/// inside `QrmiError::Ibm` / `QrmiError::Pasqal`), used by the C and Python
/// bindings to expose something more actionable than a formatted string.
/// Deliberately shared across the framework and every vendor module: it's a
/// flat classification label, not a place to encode vendor knowledge, so
/// vendor error types are expected to map their variants onto existing tags
/// here (see [`crate::ibm::error::IbmError::kind`]) rather than adding new
/// ones for every vendor-specific condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum QrmiErrorKind {
    /// A required environment variable was not set.
    EnvVarNotSet,
    /// A configuration value could not be parsed.
    ParseError,
    /// Dynamic discovery was requested for an unsupported resource type.
    UnsupportedResourceType,
    /// The payload variant is not supported by this backend.
    UnsupportedPayload,
    /// The task is not in a state that allows the requested operation.
    TaskNotReady,
    /// A required key was missing from a provider's environment variable map.
    MissingConfigKey,
    /// The named resource (e.g. a backend) does not exist.
    ResourceNotFound,
    /// The named task (e.g. a job) does not exist, or has already been
    /// removed.
    TaskNotFound,
    /// The request's credentials were missing or rejected.
    AuthenticationFailed,
    /// A `filters` string was malformed or contained an invalid value.
    InvalidFilter,
    /// A value was invalid for a reason not covered by a more specific kind.
    InvalidValue,
    /// Everything else (vendor API failures, I/O, ...).
    Other,
}

/// Reads a required environment variable, returning a [`QrmiError::EnvVarNotSet`]
/// with the variable's name if it isn't set. This replaces the repeated
/// `env::var(name).map_err(|_| anyhow!("{name} environment variable is not set"))?`
/// pattern that shows up throughout the vendor backends.
pub(crate) fn required_env(name: impl Into<String>) -> Result<String, QrmiError> {
    let name = name.into();
    std::env::var(&name).map_err(|_| QrmiError::EnvVarNotSet(name))
}
