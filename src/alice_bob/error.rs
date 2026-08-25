// This code is part of Qiskit.
//
// (C) Copyright Alice and Bob 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Maps errors from `alice_bob_felis` -- an OpenAPI-generated client we
//! don't control or modify -- onto [`crate::QrmiError`]. Same approach as
//! [`crate::ibm::error`]'s `classify` for `quantum_compute_client` (see
//! that module's docs for the general shape); the specifics here differ
//! because this API's own OpenAPI spec differs.

use crate::QrmiError;
use http::StatusCode;

/// Disambiguates a 404 from `alice_bob_felis`: on its own, the status code
/// doesn't say whether it was a job or a target (backend) that wasn't
/// found.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceKind {
    Backend,
    Job,
}

/// Maps a failed `alice_bob_felis` call onto [`QrmiError`]. `resource_kind`
/// disambiguates a 404 (see [`ResourceKind`]).
///
/// Every operation this crate uses only ever types a single status --
/// 422 (`HttpValidationError`, FastAPI's standard request-validation
/// failure response) -- in its generated per-operation error enum;
/// nothing else has a typed model at all, typed *or* generic (unlike
/// `quantum_compute_client`'s reused `ListJobs400Response` -- see
/// [`crate::ibm::error`]'s docs). That's different from IQM's 403 case,
/// though: there, a specific status had a *documented, differently-named*
/// model proving it meant something else. Here there's simply no
/// documentation either way for 401/404, so rather than assume the API
/// deviates from standard HTTP semantics without evidence, this still
/// classifies them the ordinary way. 403 gets the same conservative
/// treatment as everywhere else in this codebase, though: with nothing to
/// confirm what it means here, it's left unclassified rather than guessed
/// at.
///
/// - 422 (and 400, in case a future spec update adds it) → `InvalidInput`.
/// - 401 → `AuthenticationFailed`.
/// - 404 → `ResourceNotFound`/`TaskNotFound` by `resource_kind`.
/// - Everything else (403 included), and any transport-level failure
///   (connection, TLS, JSON decoding, ...) → `QrmiError::Other`, with `err`
///   kept as `source` either way. For a response with an unclassified
///   status specifically, the body is also attached via `.context(...)`,
///   since `Error<T>`'s own `Display` only shows the bare status (e.g.
///   `"status code 403 Forbidden"`), dropping whatever the server actually
///   said.
pub(crate) fn classify<T>(
    err: alice_bob_felis::apis::Error<T>,
    resource_kind: ResourceKind,
) -> QrmiError
where
    T: std::fmt::Debug + Send + Sync + 'static,
{
    if let alice_bob_felis::apis::Error::ResponseError(ref content) = err {
        let status = content.status;
        let body = content.content.clone();
        match (status, resource_kind) {
            (StatusCode::BAD_REQUEST, _) | (StatusCode::UNPROCESSABLE_ENTITY, _) => {
                return QrmiError::InvalidInput(body)
            }
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
