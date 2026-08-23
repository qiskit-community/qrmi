//
// (C) Copyright IBM 2024-2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Error type for this crate.
//!
//! Every public `Client`/`PrimitiveJob` method returns [`Result<T>`], where
//! the error is [`QuantumSystemError`] rather than a bare `anyhow::Error`, so
//! callers can match on *why* a call failed (a network-level failure vs. an
//! API error response vs. a job that isn't in the right state yet) instead
//! of parsing message text.
//!
//! One exception: [`crate::middleware::auth`]'s `Middleware` trait impl
//! must return `reqwest_middleware::Result<T>`, and that crate's own
//! `reqwest_middleware::Error::Middleware` variant specifically requires an
//! `anyhow::Error` -- that's an external constraint we don't control, so
//! that one file continues to build errors with `anyhow!` rather than
//! `QuantumSystemError`.

use http::StatusCode;
use thiserror::Error;

/// What kind of resource a request was about. On its own, a 404 doesn't say
/// *what* wasn't found -- this lets [`QuantumSystemError::from_response`]
/// turn that into [`QuantumSystemError::BackendNotFound`] or
/// [`QuantumSystemError::JobNotFound`] instead of a generic message.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceKind {
    Backend,
    Job,
    /// Endpoints where a 404 doesn't correspond to one specific missing
    /// resource (list endpoints, service-level metadata, ...), or where a
    /// 404 isn't expected in practice. Falls back to the generic
    /// [`QuantumSystemError::Api`].
    Other,
}

/// Errors returned by this crate.
#[derive(Error, Debug)]
pub enum QuantumSystemError {
    /// The HTTP request could not be completed at all -- no response was
    /// received from the server (connection failure, TLS error, access
    /// token refresh failure, retry policy exhausted, ...).
    #[error("request failed: {0}")]
    Request(String),

    /// HTTP 401: the request's credentials were missing or rejected.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// HTTP 403: the caller is authenticated but not permitted to perform
    /// this operation (e.g. the backend is reserved and this job is outside
    /// of the reservation).
    #[error("access denied: {0}")]
    AccessDenied(String),

    /// HTTP 404 for a request about a specific backend: the named backend
    /// does not exist.
    #[error("backend not found: {0}")]
    BackendNotFound(String),

    /// HTTP 404 for a request about a specific job: the named job does not
    /// exist (or has already been deleted).
    #[error("job not found: {0}")]
    JobNotFound(String),

    /// HTTP 408: the server timed out reading the request.
    #[error("request timeout: {0}")]
    RequestTimeout(String),

    /// HTTP 400, 409, 413, or 422: the job input was rejected -- malformed,
    /// too large, conflicting with an existing job ID, or otherwise
    /// well-formed but invalid. `body` is the API's own description of
    /// what was wrong.
    #[error("invalid job input: {0}")]
    InvalidJobInput(String),

    /// HTTP 429: no execution lane is currently available (e.g. the
    /// per-backend concurrent job limit has been reached).
    #[error("execution lanes full: {0}")]
    ExecutionLanesFull(String),

    /// HTTP 5xx: the server reported an internal error.
    #[error("server error: {0}")]
    ServerError(String),

    /// A non-success HTTP status that doesn't map to any of the more
    /// specific variants above.
    #[error("API error ({status}): {body}")]
    Api { status: StatusCode, body: String },

    /// The requested operation isn't valid for the job's current state
    /// (e.g. results or logs requested before the job reached a final
    /// state, or a timeout while waiting for one).
    #[error("{0}")]
    JobNotReady(String),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Catch-all for errors from a lower-level SDK (e.g. `aws-sdk-s3`) that
    /// don't warrant their own variant. Still carries the original error as
    /// `source` for anyone inspecting the chain (`std::error::Error::source`).
    #[error("{message}")]
    Other {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
}

impl QuantumSystemError {
    /// Wraps an arbitrary error (typically from a lower-level SDK such as
    /// `aws-sdk-s3`, whose error types are awkward to name individually
    /// since they're generic over the operation) with a short description
    /// of what was being attempted.
    pub(crate) fn other(
        context: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        let context = context.into();
        QuantumSystemError::Other {
            message: format!("{context}: {source}"),
            source: Some(Box::new(source)),
        }
    }

    /// Builds a [`QuantumSystemError`] from a failed `reqwest_middleware`
    /// request (the request never got a response from the quantum system
    /// API itself). `reqwest_middleware::Error` has two variants:
    /// `Reqwest`, a lower-level transport failure (DNS, TLS, connection
    /// refused, ...), which becomes the generic
    /// [`QuantumSystemError::Request`]; and `Middleware`, which -- in this
    /// crate -- is only ever constructed by [`crate::middleware::auth`]
    /// when it fails to obtain or refresh an access token, so it's always
    /// classified as [`QuantumSystemError::AuthenticationFailed`].
    ///
    /// This distinction relies on the retry middleware
    /// ([`crate::middleware::retry::TransparentRetryMiddleware`]) *not*
    /// rewrapping errors on its way out -- unlike
    /// `reqwest_retry::RetryTransientMiddleware`, which unconditionally
    /// rewraps whatever error it receives (even after zero retries) into
    /// an opaque `RetryError`, collapsing this exact distinction. See that
    /// module's docs for the history of why we don't use it.
    /// [`crate::Client::explain_reqwest_middleware_error`] is used to
    /// unwrap the underlying reason out of the middleware chain for the
    /// message text in every case.
    pub(crate) fn from_middleware_error(err: &reqwest_middleware::Error) -> Self {
        let msg = crate::Client::explain_reqwest_middleware_error(err);
        match err {
            reqwest_middleware::Error::Middleware(_) => {
                QuantumSystemError::AuthenticationFailed(msg)
            }
            reqwest_middleware::Error::Reqwest(_) => QuantumSystemError::Request(msg),
        }
    }

    /// Builds the appropriate `QuantumSystemError` variant from a
    /// non-success HTTP response, mapping well-known status codes (401,
    /// 403, 404, 408, 429, 5xx, and 400/409/413/422) onto a specific
    /// variant and falling back to the generic [`QuantumSystemError::Api`]
    /// for anything else. `resource_kind` disambiguates a 404 between
    /// [`QuantumSystemError::BackendNotFound`] and
    /// [`QuantumSystemError::JobNotFound`] -- the status code alone doesn't
    /// say which. In every case, `body` holds whatever level of detail
    /// could be extracted from the response (a structured JSON error, a
    /// plain text message, or a fallback description if the body was
    /// neither). Used by every API call site that gets as far as a
    /// response but finds it isn't a success, so each one doesn't have to
    /// repeat this status-code mapping itself.
    pub(crate) async fn from_response(
        status: StatusCode,
        resp: reqwest::Response,
        url: &str,
        resource_kind: ResourceKind,
    ) -> Self {
        let body = Self::extract_body(status, resp, url).await;
        match status {
            StatusCode::UNAUTHORIZED => QuantumSystemError::AuthenticationFailed(body),
            StatusCode::FORBIDDEN => QuantumSystemError::AccessDenied(body),
            StatusCode::NOT_FOUND => match resource_kind {
                ResourceKind::Backend => QuantumSystemError::BackendNotFound(body),
                ResourceKind::Job => QuantumSystemError::JobNotFound(body),
                ResourceKind::Other => QuantumSystemError::Api { status, body },
            },
            StatusCode::REQUEST_TIMEOUT => QuantumSystemError::RequestTimeout(body),
            StatusCode::BAD_REQUEST
            | StatusCode::CONFLICT
            | StatusCode::PAYLOAD_TOO_LARGE
            | StatusCode::UNPROCESSABLE_ENTITY => QuantumSystemError::InvalidJobInput(body),
            StatusCode::TOO_MANY_REQUESTS => QuantumSystemError::ExecutionLanesFull(body),
            s if s.is_server_error() => QuantumSystemError::ServerError(body),
            _ => QuantumSystemError::Api { status, body },
        }
    }

    /// Extracts whatever level of detail is available from a non-success
    /// response body: a structured JSON error, a plain text message, or --
    /// if the body couldn't be parsed as either -- a fallback description
    /// naming the status and request URL.
    async fn extract_body(status: StatusCode, resp: reqwest::Response, url: &str) -> String {
        use crate::models::errors::ExtendedErrorResponse;
        match resp.json::<ExtendedErrorResponse>().await {
            Ok(ExtendedErrorResponse::Json(error)) => {
                let body = serde_json::to_value(&error)
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| format!("{error:?}"));
                log::error!("{body}");
                body
            }
            Ok(ExtendedErrorResponse::Text(message)) => {
                log::error!("{message}");
                message
            }
            Err(_) => {
                log::error!("{status} {url}");
                format!("(no error body available; request URL: {url})")
            }
        }
    }
}

/// Result type used throughout this crate.
pub type Result<T> = std::result::Result<T, QuantumSystemError>;
