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

use crate::error::{QrmiError, QrmiErrorKind};
use http::StatusCode;
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
            IbmError::UnknownProgramId(_) => QrmiErrorKind::InvalidInput,
            IbmError::InvalidSessionMode(_) => QrmiErrorKind::InvalidInput,
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
            QsError::InvalidJobInput(msg) => crate::QrmiError::InvalidInput(msg),
            other => crate::QrmiError::Other(anyhow::Error::new(other)),
        }
    }
}

/// Disambiguates a 404 from `quantum_compute_client` (used by
/// [`crate::ibm::IBMQiskitRuntimeService`] and
/// [`crate::ibm::IBMQuantumComputeService`]): on its own, the status code
/// doesn't say whether it was a job, a session, or a backend that wasn't
/// found.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceKind {
    Backend,
    Job,
    Session,
}

/// Maps a failed `quantum_compute_client` call onto [`QrmiError`].
/// `resource_kind` disambiguates a 404 (see [`ResourceKind`]).
///
/// Unlike `iqm_server_api` (see [`crate::iqm::error`]), every operation in
/// this API reuses the same generic error body model (`ListJobs400Response`
/// -- just a `trace`/`errors` pair) regardless of status, rather than a
/// distinct *named* model per status the way IQM's `Unauthorized`/
/// `JobNotFound` were. So there's nothing in the type system here to
/// confirm what a given status actually means beyond the bare status code
/// itself.
///
/// Classified conservatively as a result: 400 is consistently "the request
/// itself was malformed" (`InvalidInput` -- literally what the reused
/// model's name suggests it was originally modeled for), 401 is
/// consistently "credentials missing or rejected" (`AuthenticationFailed`),
/// and 404 maps to `ResourceNotFound`/`TaskNotFound` for a
/// [`ResourceKind::Backend`]/[`ResourceKind::Job`] respectively. A 404 for
/// [`ResourceKind::Session`] falls through to the generic case below
/// instead: a session isn't a compute resource the way a backend is, and
/// `QrmiError` doesn't have a variant that means "this session/acquisition
/// doesn't exist" -- forcing it into `ResourceNotFound` would misrepresent
/// what was actually missing, so it's left unclassified rather than
/// mislabeled. 403 shows up on almost every endpoint here too, right
/// alongside 401 -- but with no distinguishing model, and IQM having
/// already taught us not to assume 403 means "access denied" without real
/// evidence (see [`crate::iqm::error`]'s docs), it's deliberately left
/// unclassified rather than guessed at.
///
/// Falls through to [`QrmiError::Other`] for a status this function
/// doesn't otherwise recognize (403 and a `Session` 404 included), or for
/// a transport-level failure (connection, TLS, JSON decoding, ...) that
/// has no status at all. Either way `err` is kept as `source`; for the
/// former, the response body is also attached via `.context(...)`, since
/// `Error<T>`'s own `Display` only shows the bare status (e.g. `"status
/// code 403 Forbidden"`), dropping whatever the server actually said.
pub(crate) fn classify<T>(
    err: quantum_compute_client::apis::Error<T>,
    resource_kind: ResourceKind,
) -> QrmiError
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    if let quantum_compute_client::apis::Error::ResponseError(ref content) = err {
        let status = content.status;
        let body = content.content.clone();
        match (status, resource_kind) {
            (StatusCode::BAD_REQUEST, _) => return QrmiError::InvalidInput(body),
            (StatusCode::UNAUTHORIZED, _) => return QrmiError::AuthenticationFailed(body),
            (StatusCode::NOT_FOUND, ResourceKind::Backend) => {
                return QrmiError::ResourceNotFound(body)
            }
            (StatusCode::NOT_FOUND, ResourceKind::Job) => return QrmiError::TaskNotFound(body),
            _ => {
                return QrmiError::Other(
                    anyhow::Error::new(err).context(format!("response body: {status} {body}")),
                );
            }
        }
    }
    QrmiError::Other(anyhow::Error::new(err))
}
