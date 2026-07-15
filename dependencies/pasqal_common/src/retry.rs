//
// (C) Copyright Pasqal SAS 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Retry configuration shared by the Pasqal clients.
//!
//! Retries are split into two tiers:
//!
//! * ordinary transient failures (5xx, timeouts, connection errors), handled by
//!   the stock [`RetryTransientMiddleware`] with an exponential backoff that
//!   starts at 1s, doubles, and is capped at 5s per wait, and
//! * rate-limit / locked responses (HTTP 429 and 423), handled by
//!   [`RetryAfterMiddleware`]. When the server sends a `Retry-After` header we
//!   trust it and retry exactly once, since the server has told us when it will
//!   be ready and guessing again afterwards only adds load. With no such header
//!   we fall back to an exponential backoff that starts at 5s, doubles, and is
//!   capped at 30s per wait — a rate-limited backend wants more breathing room
//!   between polls than a merely flaky one.
//!
//! Both tiers are bounded by a retry *count* rather than a wall-clock budget,
//! and share the same count so that a caller has a single number to reason
//! about (see [`with_retry`]). A count of zero disables both tiers, including
//! the `Retry-After`-guided retry.

use std::time::{Duration, SystemTime};

use anyhow::anyhow;
use async_trait::async_trait;
use http::Extensions;
use log::debug;
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, Jitter, RetryDecision, RetryPolicy,
    RetryTransientMiddleware, Retryable, RetryableStrategy,
};

const HTTP_TOO_MANY_REQUESTS: u16 = 429;
const HTTP_LOCKED: u16 = 423;

/// How many times a request is retried, absent an explicit choice by the caller.
pub const DEFAULT_MAX_RETRIES: u32 = 5;

/// Longest wait a server's `Retry-After` header can ask of us.
///
/// The header is honored as sent, up to this ceiling: a backend that asks us to
/// come back in an hour is telling us it is not coming back within the lifetime
/// of any request we care about, so the request fails with an error naming the
/// wait the server asked for instead of sleeping on it.
const RETRY_AFTER_MAX_WAIT: Duration = Duration::from_secs(300);

/// Backoff parameters for a single class of retryable responses.
#[derive(Debug, Clone)]
struct BackoffConfig {
    /// Wait before the first retry (grows exponentially from here).
    min_interval: Duration,
    /// Upper bound applied to any single wait between attempts.
    max_interval: Duration,
    /// Base of the exponential growth (typically 2).
    base: u32,
    /// Whether to apply bounded jitter to smooth out retry storms.
    jitter: bool,
    /// How many times a failed request is retried before it is given up on.
    max_retries: u32,
}

impl BackoffConfig {
    /// Compile these parameters into a count-bounded exponential backoff policy.
    fn build_policy(&self) -> ExponentialBackoff {
        ExponentialBackoff::builder()
            .retry_bounds(self.min_interval, self.max_interval)
            .base(self.base)
            .jitter(if self.jitter {
                Jitter::Bounded
            } else {
                Jitter::None
            })
            .build_with_max_retries(self.max_retries)
    }
}

/// Retry policy for a Pasqal HTTP client.
///
/// This is an internal implementation detail: the only knob callers get is the
/// retry count passed to [`with_retry`] (or no retries at all). The backoff
/// shape is not exposed for per-call tuning by design.
/// Client errors (e.g. missing/wrong auth, 401/403, are never retried)
/// See https://docs.rs/reqwest-retry/0.7.0/reqwest_retry/fn.default_on_request_success.html
#[derive(Debug, Clone)]
struct RetryConfig {
    /// Backoff for ordinary transient failures (5xx, timeouts, connection
    /// errors). Excludes 429/423, which are handled by [`Self::rate_limit`].
    transient: BackoffConfig,
    /// Fallback backoff for rate-limit (429) and locked (423) responses that
    /// carry no `Retry-After` header.
    rate_limit: BackoffConfig,
}

impl RetryConfig {
    /// Production defaults, retrying each tier at most `max_retries` times.
    fn new(max_retries: u32) -> Self {
        Self {
            transient: BackoffConfig {
                min_interval: Duration::from_secs(1),
                max_interval: Duration::from_secs(5),
                base: 2,
                jitter: true,
                max_retries,
            },
            rate_limit: BackoffConfig {
                // A rate-limited backend wants more breathing room between
                // polls, so start higher and allow a higher per-attempt ceiling
                // than ordinary transient errors.
                min_interval: Duration::from_secs(5),
                max_interval: Duration::from_secs(30),
                base: 2,
                jitter: true,
                max_retries,
            },
        }
    }
}

/// Retry every transient failure *except* rate-limit / locked responses, which
/// the dedicated [`RetryAfterMiddleware`] tier owns.
struct RetryStrategyExcept429;
impl RetryableStrategy for RetryStrategyExcept429 {
    fn handle(
        &self,
        res: &Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<Retryable> {
        match res {
            Ok(success)
                if success.status() == HTTP_TOO_MANY_REQUESTS
                    || success.status() == HTTP_LOCKED =>
            {
                None
            }
            Ok(success) => default_on_request_success(success),
            Err(error) => default_on_request_failure(error),
        }
    }
}

/// Parse a `Retry-After` header value into a delay measured from `now`.
///
/// Supports both RFC-7231 forms: delay-seconds (e.g. `Retry-After: 30`) and an
/// HTTP-date (e.g. `Retry-After: Fri, 31 Dec 1999 23:59:59 GMT`). A date in the
/// past yields a zero delay (retry immediately). Returns `None` when the header
/// is absent or unparseable.
fn parse_retry_after(headers: &http::HeaderMap, now: SystemTime) -> Option<Duration> {
    let value = headers
        .get(http::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let when = httpdate::parse_http_date(value).ok()?;
    Some(when.duration_since(now).unwrap_or(Duration::ZERO))
}

/// Middleware dedicated to rate-limit (429) and locked (423) responses.
///
/// Two behaviors, depending on what the server tells us:
///
/// * With a `Retry-After` header, we wait as instructed and retry **once**. The
///   server has stated when it expects to be ready; if it rate-limits us again
///   anyway, retrying further is guesswork, so the response goes to the caller.
///   A `Retry-After` beyond [`RETRY_AFTER_MAX_WAIT`] is treated as "not coming
///   back in time": the request fails immediately with an error explaining that
///   we chose not to wait, rather than with a bare rate-limit response.
/// * Without one, we retry up to `max_retries` times on the exponential backoff
///   in `policy`.
///
/// Every other outcome — success, a non-429/423 status, or a request-level
/// error — is passed straight through for the transient tier to handle.
struct RetryAfterMiddleware {
    policy: ExponentialBackoff,
}

#[async_trait]
impl Middleware for RetryAfterMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let start = SystemTime::now();
        // Counts only the backoff-driven retries: the single `Retry-After`
        // retry is budgeted separately, and consuming from the same counter
        // would let a server shrink its own backoff budget by sending a header.
        let mut n_past_retries: u32 = 0;
        let mut retry_after_used = false;
        loop {
            // A non-cloneable body (e.g. a stream) cannot be replayed, so we
            // can only send it once and return whatever we get.
            let Some(duplicate) = req.try_clone() else {
                return next.run(req, extensions).await;
            };

            let response = next.clone().run(duplicate, extensions).await?;
            let status = response.status().as_u16();
            if status != HTTP_TOO_MANY_REQUESTS && status != HTTP_LOCKED {
                return Ok(response);
            }

            let now = SystemTime::now();
            let delay = match parse_retry_after(response.headers(), now) {
                Some(retry_after) => {
                    if retry_after_used {
                        debug!(
                            "Received status {} again after honoring Retry-After; giving up",
                            status
                        );
                        return Ok(response);
                    }
                    if retry_after > RETRY_AFTER_MAX_WAIT {
                        // Declining to wait is our decision, not the server's,
                        // so say so: passing the bare 429 up would surface as an
                        // ordinary rate-limit error and leave the caller unable
                        // to tell that the backend had in fact told us when to
                        // come back.
                        return Err(reqwest_middleware::Error::Middleware(anyhow!(
                            "Request was rate limited (HTTP {status}) and the server asked to \
                             retry after {}s, which is longer than the maximum wait of {}s. \
                             The request was not retried; try again later.",
                            retry_after.as_secs(),
                            RETRY_AFTER_MAX_WAIT.as_secs(),
                        )));
                    }
                    retry_after_used = true;
                    retry_after
                }
                None => {
                    let RetryDecision::Retry { execute_after } =
                        self.policy.should_retry(start, n_past_retries)
                    else {
                        debug!("Received status {}; retry budget exhausted", status);
                        return Ok(response);
                    };
                    n_past_retries += 1;
                    execute_after.duration_since(now).unwrap_or_default()
                }
            };

            debug!("Received status {}; retrying after {:?}", status, delay);
            tokio::time::sleep(delay).await;
        }
    }
}

/// Attach the default retry middleware to `builder`, retrying a failed request
/// at most `max_retries` times.
///
/// Two middlewares are stacked: a [`RetryTransientMiddleware`] (with
/// `RetryStrategyExcept429`) for ordinary transient failures, and a
/// [`RetryAfterMiddleware`] for 429/423 responses. See the module docs for the
/// backoff each uses.
///
/// A `max_retries` of zero attaches nothing: every request is made exactly
/// once. In particular the single `Retry-After`-guided retry — which for a
/// nonzero count is budgeted *in addition to* `max_retries` — is not granted
/// either, since a caller asking for zero retries is asking to never sleep on
/// a response.
pub fn with_retry(builder: ClientBuilder, max_retries: u32) -> ClientBuilder {
    if max_retries == 0 {
        return builder;
    }
    let config = RetryConfig::new(max_retries);
    builder
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            config.transient.build_policy(),
            RetryStrategyExcept429,
        ))
        .with(RetryAfterMiddleware {
            policy: config.rate_limit.build_policy(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_map(value: &str) -> http::HeaderMap {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::RETRY_AFTER,
            http::HeaderValue::from_str(value).unwrap(),
        );
        headers
    }

    #[test]
    fn default_config_backoffs() {
        let cfg = RetryConfig::new(DEFAULT_MAX_RETRIES);
        // Transient failures back off from 1s to at most 5s...
        assert_eq!(cfg.transient.min_interval, Duration::from_secs(1));
        assert_eq!(cfg.transient.max_interval, Duration::from_secs(5));
        // ...while rate limits start higher and are given more room.
        assert_eq!(cfg.rate_limit.min_interval, Duration::from_secs(5));
        assert_eq!(cfg.rate_limit.max_interval, Duration::from_secs(30));
        // Both tiers double, and share the caller's retry count.
        assert_eq!(cfg.transient.base, 2);
        assert_eq!(cfg.rate_limit.base, 2);
        assert_eq!(cfg.transient.max_retries, DEFAULT_MAX_RETRIES);
        assert_eq!(cfg.rate_limit.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn policy_stops_after_max_retries() {
        let policy = RetryConfig::new(2).transient.build_policy();
        let start = SystemTime::now();
        assert!(matches!(
            policy.should_retry(start, 1),
            RetryDecision::Retry { .. }
        ));
        assert!(matches!(
            policy.should_retry(start, 2),
            RetryDecision::DoNotRetry
        ));
    }

    #[test]
    fn with_retry_builds_a_usable_client() {
        // Ensure the middleware stack type-checks and a client can be built.
        let builder = ClientBuilder::new(reqwest::Client::new());
        let _client = with_retry(builder, DEFAULT_MAX_RETRIES).build();
    }

    #[test]
    fn parse_retry_after_delay_seconds() {
        let now = SystemTime::UNIX_EPOCH;
        assert_eq!(
            parse_retry_after(&header_map("30"), now),
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn parse_retry_after_http_date() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let target = now + Duration::from_secs(100);
        let headers = header_map(&httpdate::fmt_http_date(target));
        assert_eq!(
            parse_retry_after(&headers, now),
            Some(Duration::from_secs(100))
        );
    }

    #[test]
    fn parse_retry_after_past_date_is_zero() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let past = SystemTime::UNIX_EPOCH + Duration::from_secs(500);
        let headers = header_map(&httpdate::fmt_http_date(past));
        assert_eq!(parse_retry_after(&headers, now), Some(Duration::ZERO));
    }

    #[test]
    fn parse_retry_after_absent_or_invalid() {
        let now = SystemTime::UNIX_EPOCH;
        assert_eq!(parse_retry_after(&http::HeaderMap::new(), now), None);
        assert_eq!(parse_retry_after(&header_map("not-a-date"), now), None);
    }
}
