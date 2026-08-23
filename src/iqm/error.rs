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

//! Maps errors from `iqm_server_api` -- an OpenAPI-generated client we don't
//! control or modify -- onto [`crate::QrmiError`].
//!
//! Unlike `quantum_system_client` (which we own, and so gave its own
//! `thiserror`-based error type with the same kind of classification baked
//! directly into it), `iqm_server_api` is generated code: every operation
//! returns `Result<T, iqm_server_api::apis::Error<E>>`, where `E` is a
//! *different, per-operation* enum (`GetJobV1Error`, `CancelJobV1Error`,
//! ...) of named response models (`Unauthorized`, `JobNotFound`,
//! `RateLimitExceeded`, ...). That per-operation typing is nice for anyone
//! matching on a single specific call, but it means there's no one type to
//! hang a shared classifier off of the way `QuantumSystemError::from_response`
//! does. [`classify`] instead works off of
//! [`iqm_server_api::apis::Error::ResponseError`]'s `status` field, which
//! every operation's error carries regardless of `E` -- the same
//! status-code-based approach, just implemented here (the consumer) instead
//! of there (the generated crate).

use crate::QrmiError;
use http::StatusCode;

/// Disambiguates a 404 from the IQM Server API: on its own, the status code
/// doesn't say whether it was a job or a quantum computer that wasn't
/// found.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceKind {
    Backend,
    Job,
}

/// Maps a failed `iqm_server_api` call onto [`QrmiError`]. `resource_kind`
/// disambiguates a 404 (see [`ResourceKind`]).
///
/// Deliberately conservative about which statuses get their own variant:
/// 401 is uniformly "the API token was missing or rejected" across every
/// endpoint (`Unauthorized`, generated the same way everywhere), so it maps
/// to [`QrmiError::AuthenticationFailed`]. 400 is likewise consistently
/// "the request itself was malformed" (`InvalidInput`, or -- for
/// `job_submit` specifically -- one of `InvalidJobPayload` /
/// `UnsupportedJobType` / `InvalidInput`), so it maps to
/// [`QrmiError::InvalidInput`]. 403, by contrast, means something
/// different per endpoint in this API -- e.g. `cancel_job_v1`'s 403 is
/// `IllegalJobStatus` (the job's current state doesn't allow cancelling
/// it), which has nothing to do with authentication -- so it's deliberately
/// left unclassified (falls through to [`QrmiError::Other`]) rather than
/// guessing.
///
/// Transport-level failures (connection, TLS, JSON decoding, ...) have no
/// status to classify by and always fall through to
/// [`QrmiError::Other`] too, same as a status this function doesn't
/// otherwise recognize -- still carrying the original error as `source` (it
/// implements `std::error::Error` via `iqm_server_api`'s own generated
/// impl), so nothing is lost for anyone inspecting the chain.
pub(crate) fn classify<T>(
    err: iqm_server_api::apis::Error<T>,
    resource_kind: ResourceKind,
) -> QrmiError
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    if let iqm_server_api::apis::Error::ResponseError(ref content) = err {
        let body = content.content.clone();
        match content.status {
            StatusCode::BAD_REQUEST => return QrmiError::InvalidInput(body),
            StatusCode::UNAUTHORIZED => return QrmiError::AuthenticationFailed(body),
            StatusCode::NOT_FOUND => {
                return match resource_kind {
                    ResourceKind::Backend => QrmiError::ResourceNotFound(body),
                    ResourceKind::Job => QrmiError::TaskNotFound(body),
                };
            }
            _ => {}
        }
    }
    QrmiError::Other(anyhow::Error::new(err))
}
