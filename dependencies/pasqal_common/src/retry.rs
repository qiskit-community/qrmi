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
//! A single middleware owns the whole policy. Each failed attempt is sorted
//! into one of two tiers:
//!
//! * ordinary transient failures (5xx, timeouts, connection errors), retried
//!   on an exponential backoff that starts at 1s, doubles, and is capped at
//!   5s per wait, and
//! * rate-limit / locked responses (HTTP 429 and 423). When the server sends
//!   a `Retry-After` header we trust it and retry exactly once, since the
//!   server has told us when it will be ready and guessing again afterwards
//!   only adds load. With no such header we fall back to an exponential
//!   backoff that starts at 5s, doubles, and is capped at 30s per wait — a
//!   rate-limited backend wants more breathing room between polls than a
//!   merely flaky one.
//!
//! Both tiers draw on one shared budget of `max_retries` retries, so a caller
//! has a single number to reason about: however the attempts fail — and in
//! whatever mix of transient errors and rate limits — at most
//! `max_retries + 1` requests are made in total (see [`with_retry`]). A count
//! of zero disables retries entirely, including the `Retry-After`-guided one.

use std::time::{Duration, SystemTime};

use anyhow::anyhow;
use async_trait::async_trait;
use http::Extensions;
use log::debug;
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, Jitter, RetryDecision, RetryPolicy,
    Retryable,
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
///
/// The retry *count* is deliberately not part of the shape: it lives on
/// [`RetryConfig`], because both classes share one budget.
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Wait before the first retry (grows exponentially from here).
    pub min_interval: Duration,
    /// Upper bound applied to any single wait between attempts.
    pub max_interval: Duration,
    /// Base of the exponential growth (typically 2).
    pub base: u32,
    /// Whether to apply bounded jitter to smooth out retry storms.
    pub jitter: bool,
}

impl BackoffConfig {
    /// Compile these parameters into a count-bounded exponential backoff policy.
    fn build_policy(&self, max_retries: u32) -> ExponentialBackoff {
        ExponentialBackoff::builder()
            .retry_bounds(self.min_interval, self.max_interval)
            .base(self.base)
            .jitter(if self.jitter {
                Jitter::Bounded
            } else {
                Jitter::None
            })
            .build_with_max_retries(max_retries)
    }
}

/// Retry policy for a Pasqal HTTP client.
///
/// The only knob production callers get is the retry count passed to
/// [`with_retry`] (or no retries at all); the backoff shape is not exposed for
/// per-call tuning by design. [`with_retry_config`] accepts the full policy so
/// that tests can run on millisecond backoffs.
/// Client errors (e.g. missing/wrong auth, 401/403) are never retried.
/// See https://docs.rs/reqwest-retry/0.7.0/reqwest_retry/fn.default_on_request_success.html
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// How many times a failed request is retried before it is given up on.
    /// One budget shared by all failure classes: at most this many retries
    /// are made in total, whatever mix of outcomes drives them.
    pub max_retries: u32,
    /// Backoff for ordinary transient failures (5xx, timeouts, connection
    /// errors). Excludes 429/423, which are handled by [`Self::rate_limit`].
    pub transient: BackoffConfig,
    /// Fallback backoff for rate-limit (429) and locked (423) responses that
    /// carry no `Retry-After` header.
    pub rate_limit: BackoffConfig,
    /// Longest wait a server's `Retry-After` header can ask of us; a header
    /// beyond this fails the request instead of sleeping on it.
    pub retry_after_max_wait: Duration,
}

impl RetryConfig {
    /// Production defaults, retrying at most `max_retries` times in total.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            transient: BackoffConfig {
                min_interval: Duration::from_secs(1),
                max_interval: Duration::from_secs(5),
                base: 2,
                jitter: true,
            },
            rate_limit: BackoffConfig {
                // A rate-limited backend wants more breathing room between
                // polls, so start higher and allow a higher per-attempt ceiling
                // than ordinary transient errors.
                min_interval: Duration::from_secs(5),
                max_interval: Duration::from_secs(30),
                base: 2,
                jitter: true,
            },
            retry_after_max_wait: RETRY_AFTER_MAX_WAIT,
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

/// Whether an attempt's outcome is an ordinary transient failure (5xx, timeout,
/// connection error), classified the way reqwest-retry's defaults do. Only
/// consulted after 429/423 have been claimed by the rate-limit tier.
fn is_transient(result: &Result<reqwest::Response, reqwest_middleware::Error>) -> bool {
    let retryable = match result {
        Ok(response) => default_on_request_success(response),
        Err(error) => default_on_request_failure(error),
    };
    matches!(retryable, Some(Retryable::Transient))
}

/// The retry middleware: one budget, two backoff tiers.
///
/// Ordinary transient failures are retried on [`Self::transient_policy`].
/// Rate-limit (429) and locked (423) responses are retried on the server's
/// `Retry-After` header when it carries one — honored at most once, capped at
/// [`Self::retry_after_max_wait`] — and on [`Self::rate_limit_policy`]
/// otherwise. Both policies are built with the same retry count, so every
/// retry, however motivated, draws on the same budget.
struct RetryMiddleware {
    /// Backoff for ordinary transient failures.
    transient_policy: ExponentialBackoff,
    /// Fallback backoff for 429/423 responses without a `Retry-After` header.
    rate_limit_policy: ExponentialBackoff,
    /// Longest `Retry-After` we are willing to sleep on.
    retry_after_max_wait: Duration,
}

impl RetryMiddleware {
    fn new(config: &RetryConfig) -> Self {
        Self {
            transient_policy: config.transient.build_policy(config.max_retries),
            rate_limit_policy: config.rate_limit.build_policy(config.max_retries),
            retry_after_max_wait: config.retry_after_max_wait,
        }
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let start = SystemTime::now();
        let mut n_past_retries: u32 = 0;
        let mut retry_after_honored = false;
        loop {
            // A non-cloneable body (e.g. a stream) cannot be replayed, so we
            // can only send it once and return whatever we get.
            let Some(duplicate) = req.try_clone() else {
                return next.run(req, extensions).await;
            };

            let result = next.clone().run(duplicate, extensions).await;
            let now = SystemTime::now();

            // Sort the outcome into the tier that owns it. Anything neither
            // tier claims — success, a client error, a fatal request error —
            // goes to the caller as-is.
            let (policy, retry_after, what) = match &result {
                Ok(response)
                    if response.status().as_u16() == HTTP_TOO_MANY_REQUESTS
                        || response.status().as_u16() == HTTP_LOCKED =>
                {
                    let status = response.status().as_u16();
                    match parse_retry_after(response.headers(), now) {
                        Some(_) if retry_after_honored => {
                            // The server already told us once when it would be
                            // ready; if it rate-limits us again anyway,
                            // retrying further on its schedule is guesswork,
                            // so the response goes to the caller.
                            debug!(
                                "Received status {} again after honoring Retry-After; giving up",
                                status
                            );
                            return result;
                        }
                        Some(retry_after) if retry_after > self.retry_after_max_wait => {
                            // Declining to wait is our decision, not the
                            // server's, so say so: passing the bare 429 up
                            // would surface as an ordinary rate-limit error
                            // and leave the caller unable to tell that the
                            // backend had in fact told us when to come back.
                            return Err(reqwest_middleware::Error::Middleware(anyhow!(
                                "Request was rate limited (HTTP {status}) and the server asked to \
                                 retry after {}s, which is longer than the maximum wait of {}s. \
                                 The request was not retried; try again later.",
                                retry_after.as_secs(),
                                self.retry_after_max_wait.as_secs(),
                            )));
                        }
                        retry_after => (
                            &self.rate_limit_policy,
                            retry_after,
                            format!("status {status}"),
                        ),
                    }
                }
                outcome if is_transient(outcome) => {
                    let what = match outcome {
                        Ok(response) => format!("status {}", response.status().as_u16()),
                        Err(error) => format!("error: {error}"),
                    };
                    (&self.transient_policy, None, what)
                }
                _ => return result,
            };

            // The single shared budget. Both policies carry the same retry
            // count, so whichever one is consulted enforces the same cap. A
            // `Retry-After`-guided retry draws on it too: the budget is a
            // promise to the caller about total attempts, not per tier, and
            // exempting the header would let a busy backend stretch it.
            let RetryDecision::Retry { execute_after } = policy.should_retry(start, n_past_retries)
            else {
                debug!("Received {}; retry budget exhausted", what);
                return result;
            };
            n_past_retries += 1;

            let delay = match retry_after {
                Some(retry_after) => {
                    retry_after_honored = true;
                    retry_after
                }
                None => execute_after.duration_since(now).unwrap_or_default(),
            };

            debug!("Received {}; retrying after {:?}", what, delay);
            tokio::time::sleep(delay).await;
        }
    }
}

/// Attach the default retry middleware to `builder`, retrying a failed request
/// at most `max_retries` times.
///
/// One [`RetryMiddleware`] owns the whole policy: ordinary transient failures
/// and 429/423 responses share the single `max_retries` budget, so at most
/// `max_retries + 1` requests are ever made. See the module docs for the
/// backoff each class of failure gets.
///
/// A `max_retries` of zero attaches nothing: every request is made exactly
/// once, and no `Retry-After` header is honored either, since a caller asking
/// for zero retries is asking to never sleep on a response.
pub fn with_retry(builder: ClientBuilder, max_retries: u32) -> ClientBuilder {
    with_retry_config(builder, RetryConfig::new(max_retries))
}

/// Like [`with_retry`], with the full policy spelled out by the caller.
///
/// Exists mainly so tests can inject millisecond backoffs; production callers
/// should not need anything beyond [`with_retry`].
pub fn with_retry_config(builder: ClientBuilder, config: RetryConfig) -> ClientBuilder {
    if config.max_retries == 0 {
        return builder;
    }
    builder.with(RetryMiddleware::new(&config))
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
        // Both tiers double, and share the caller's single retry budget.
        assert_eq!(cfg.transient.base, 2);
        assert_eq!(cfg.rate_limit.base, 2);
        assert_eq!(cfg.max_retries, DEFAULT_MAX_RETRIES);
        // A server may ask us to wait at most five minutes.
        assert_eq!(cfg.retry_after_max_wait, Duration::from_secs(300));
    }

    #[test]
    fn policy_stops_after_max_retries() {
        let policy = RetryConfig::new(2).transient.build_policy(2);
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
