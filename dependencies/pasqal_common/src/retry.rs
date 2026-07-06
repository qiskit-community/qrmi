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
//!   the stock [`RetryTransientMiddleware`] with an exponential backoff, and
//! * rate-limit / locked responses (HTTP 429 and 423), handled by
//!   [`RetryAfterMiddleware`], which honors the server's `Retry-After` header
//!   when present and otherwise falls back to the same exponential backoff.
//!
//! Both tiers are bounded by wall-clock time rather than a retry count, so a
//! request never spends longer than its tier's `max_total_duration`
//! retrying, no matter how many attempts that allows. The rate-limit tier gets
//! a larger per-attempt ceiling and a larger total budget, since a busy backend
//! wants to be given more room between polls.

use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use http::Extensions;
use log::debug;
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::policies::{ExponentialBackoff, ExponentialBackoffTimed};
use reqwest_retry::{
    default_on_request_failure, default_on_request_success, Jitter, RetryDecision, RetryPolicy,
    RetryTransientMiddleware, Retryable, RetryableStrategy,
};

const HTTP_TOO_MANY_REQUESTS: u16 = 429;
const HTTP_LOCKED: u16 = 423;

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
    /// Hard cap on the total wall-clock time spent retrying a request. Once
    /// this budget is exhausted the request fails, regardless of attempt count.
    max_total_duration: Duration,
}

impl BackoffConfig {
    /// Compile these parameters into a time-bounded exponential backoff policy.
    fn build_policy(&self) -> ExponentialBackoffTimed {
        ExponentialBackoff::builder()
            .retry_bounds(self.min_interval, self.max_interval)
            .base(self.base)
            .jitter(if self.jitter {
                Jitter::Bounded
            } else {
                Jitter::None
            })
            .build_with_total_retry_duration(self.max_total_duration)
    }
}

/// Retry policy for a Pasqal HTTP client.
///
/// This is an internal implementation detail: callers only ever get the
/// [`Default`] policy (via [`with_retry`]) or no retries at all. The knobs are
/// not exposed for per-call tuning by design — retries are toggled on/off at
/// the client level, nothing more.
#[derive(Debug, Clone)]
struct RetryConfig {
    /// Backoff for ordinary transient failures (5xx, timeouts, connection
    /// errors). Excludes 429/423, which are handled by [`Self::rate_limit`].
    transient: BackoffConfig,
    /// Backoff for rate-limit (429) and locked (423) responses.
    rate_limit: BackoffConfig,
}

impl Default for RetryConfig {
    /// Sensible production defaults: spend at most two minutes retrying
    /// ordinary transient failures, and up to five minutes on rate limits.
    fn default() -> Self {
        Self {
            transient: BackoffConfig {
                min_interval: Duration::from_secs(1),
                max_interval: Duration::from_secs(10),
                base: 2,
                jitter: true,
                max_total_duration: Duration::from_secs(120),
            },
            rate_limit: BackoffConfig {
                min_interval: Duration::from_secs(1),
                // A rate-limited backend wants more breathing room between
                // polls, so allow a higher per-attempt ceiling and a longer
                // overall budget than ordinary transient errors.
                max_interval: Duration::from_secs(60),
                base: 2,
                jitter: true,
                max_total_duration: Duration::from_secs(300),
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
/// On such a response it waits for the duration advertised by the server's
/// `Retry-After` header when present, and otherwise falls back to the
/// exponential backoff in `policy`. Retrying stops once `max_total_duration`
/// of wall-clock time has elapsed; if a `Retry-After` value would push past
/// that budget the request gives up immediately rather than sleeping
/// pointlessly. Every other outcome — success, non-429/423 status, or a
/// request-level error — is passed straight through so the transient tier can
/// handle it.
struct RetryAfterMiddleware {
    policy: ExponentialBackoffTimed,
    max_total_duration: Duration,
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
        let mut n_past_retries: u32 = 0;
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

            // `should_retry` enforces the total-duration budget and provides the
            // fallback backoff when the server sends no `Retry-After` header.
            let RetryDecision::Retry { execute_after } =
                self.policy.should_retry(start, n_past_retries)
            else {
                return Ok(response);
            };
            let now = SystemTime::now();
            let backoff = execute_after.duration_since(now).unwrap_or_default();

            let delay = match parse_retry_after(response.headers(), now) {
                Some(retry_after) => {
                    // Respect the server's signal, but never sleep past our own
                    // budget: if it would, give up now instead.
                    let elapsed = now.duration_since(start).unwrap_or_default();
                    let remaining = self.max_total_duration.saturating_sub(elapsed);
                    if retry_after > remaining {
                        debug!(
                            "Retry-After ({:?}) exceeds remaining retry budget ({:?}); giving up",
                            retry_after, remaining
                        );
                        return Ok(response);
                    }
                    retry_after
                }
                None => backoff,
            };

            debug!(
                "Received status {}; retrying after {:?} (attempt {})",
                status,
                delay,
                n_past_retries + 1
            );
            tokio::time::sleep(delay).await;
            n_past_retries += 1;
        }
    }
}

/// Attach the default retry middleware to `builder`.
///
/// Two middlewares are stacked: a [`RetryTransientMiddleware`] (with
/// `RetryStrategyExcept429`) for ordinary transient failures, and a
/// `RetryAfterMiddleware` for 429/423 responses. Each is bounded by its
/// tier's `max_total_duration`, so a request never retries beyond that budget.
pub fn with_retry(builder: ClientBuilder) -> ClientBuilder {
    let config = RetryConfig::default();
    builder
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            config.transient.build_policy(),
            RetryStrategyExcept429,
        ))
        .with(RetryAfterMiddleware {
            policy: config.rate_limit.build_policy(),
            max_total_duration: config.rate_limit.max_total_duration,
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
    fn default_config_budgets() {
        let cfg = RetryConfig::default();
        // Transient errors give up after 2 minutes; rate limits get 5 minutes.
        assert_eq!(cfg.transient.max_total_duration, Duration::from_secs(120));
        assert_eq!(cfg.rate_limit.max_total_duration, Duration::from_secs(300));
        // The rate-limit tier also gets a larger per-attempt ceiling.
        assert!(cfg.rate_limit.max_interval > cfg.transient.max_interval);
    }

    #[test]
    fn with_retry_builds_a_usable_client() {
        // Ensure the middleware stack type-checks and a client can be built.
        let builder = ClientBuilder::new(reqwest::Client::new());
        let _client = with_retry(builder).build();
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
