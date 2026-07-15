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

//! Integration tests for the retry middleware against a mock server.
//!
//! Every retrying client here is built through [`with_retry_config`] with
//! millisecond backoffs, so the suite exercises the real sleep paths without
//! actually waiting out production-scale intervals.

use std::time::{Duration, Instant};

use pasqal_common::{with_retry, with_retry_config, RetryConfig, DEFAULT_MAX_RETRIES};
use reqwest_middleware::ClientBuilder as MiddlewareClientBuilder;

/// Production policy shrunk to millisecond backoffs. The rate-limit tier still
/// starts an order of magnitude above the transient one, so tests can tell the
/// two apart by elapsed time.
fn fast_config(max_retries: u32) -> RetryConfig {
    let mut config = RetryConfig::new(max_retries);
    config.transient.min_interval = Duration::from_millis(1);
    config.transient.max_interval = Duration::from_millis(5);
    config.rate_limit.min_interval = Duration::from_millis(20);
    config.rate_limit.max_interval = Duration::from_millis(50);
    config
}

fn client_with(max_retries: u32) -> reqwest_middleware::ClientWithMiddleware {
    with_retry_config(
        MiddlewareClientBuilder::new(reqwest::Client::new()),
        fast_config(max_retries),
    )
    .build()
}

fn client() -> reqwest_middleware::ClientWithMiddleware {
    client_with(DEFAULT_MAX_RETRIES)
}

/// A retry count of zero means the request is made once and not retried. This
/// goes through the public [`with_retry`] entry point on purpose: zero must
/// attach no middleware at all.
#[tokio::test]
async fn zero_max_retries_makes_a_single_request() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .expect(1)
        .create_async()
        .await;

    let response = with_retry(MiddlewareClientBuilder::new(reqwest::Client::new()), 0)
        .build()
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 429);
    rate_limited.assert_async().await;
}

/// A retry count of zero disables the `Retry-After` handling too: a caller who
/// asked for zero retries gets exactly one request and no sleep.
#[tokio::test]
async fn zero_max_retries_does_not_honor_retry_after() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .with_header("retry-after", "0")
        .expect(1)
        .create_async()
        .await;

    let response = with_retry(MiddlewareClientBuilder::new(reqwest::Client::new()), 0)
        .build()
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 429);
    rate_limited.assert_async().await;
}

/// An ordinary transient failure (5xx) is retried on the transient backoff and
/// the eventual success is returned.
#[tokio::test]
async fn retries_transient_5xx_then_succeeds() {
    let mut server = mockito::Server::new_async().await;

    // Created first, so it wins while it still has unmet expectations.
    let flaky = server
        .mock("GET", "/")
        .with_status(503)
        .expect(1)
        .create_async()
        .await;
    // Created last, so it serves every request after `flaky` is satisfied.
    let ok = server
        .mock("GET", "/")
        .with_status(200)
        .with_body("ok")
        .create_async()
        .await;

    let response = client()
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "ok");
    flaky.assert_async().await;
    ok.assert_async().await;
}

/// A client error other than 429/423 is not retried at all.
#[tokio::test]
async fn does_not_retry_client_errors() {
    let mut server = mockito::Server::new_async().await;

    let not_found = server
        .mock("GET", "/")
        .with_status(404)
        .expect(1)
        .create_async()
        .await;

    let response = client()
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 404);
    not_found.assert_async().await;
}

/// A 429 carrying `Retry-After: 0` should be retried immediately and succeed
/// once the server starts returning 200.
#[tokio::test]
async fn retries_429_with_retry_after_then_succeeds() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .with_header("retry-after", "0")
        .expect(1)
        .create_async()
        .await;
    let ok = server
        .mock("GET", "/")
        .with_status(200)
        .with_body("ok")
        .create_async()
        .await;

    let response = client()
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "ok");
    rate_limited.assert_async().await;
    ok.assert_async().await;
}

/// `Retry-After` buys exactly one retry: the server said when it would be ready,
/// and if it rate-limits us again anyway we surface that rather than guess.
#[tokio::test]
async fn retries_429_with_retry_after_only_once() {
    let mut server = mockito::Server::new_async().await;

    // Two requests: the original and the single Retry-After-guided retry. A
    // third would fail this expectation.
    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .with_header("retry-after", "0")
        .expect(2)
        .create_async()
        .await;

    let response = client()
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 429);
    rate_limited.assert_async().await;
}

/// Without a `Retry-After` header the middleware falls back to the rate-limit
/// backoff — noticeably slower than the transient one — and stops after
/// `max_retries` retries.
#[tokio::test]
async fn retries_429_without_retry_after_on_backoff() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .expect(2) // the original request plus the one retry allowed below
        .create_async()
        .await;

    let started = Instant::now();
    let response = client_with(1)
        .get(server.url())
        .send()
        .await
        .expect("request should complete");
    let elapsed = started.elapsed();

    assert_eq!(response.status(), 429);
    rate_limited.assert_async().await;
    assert!(
        elapsed >= Duration::from_millis(20),
        "the retry should have waited out the rate-limit base backoff of 20ms, \
         but took only {elapsed:?}"
    );
}

/// Once the budget is spent, the last response is returned to the caller.
#[tokio::test]
async fn budget_exhaustion_returns_the_last_response() {
    let mut server = mockito::Server::new_async().await;

    let broken = server
        .mock("GET", "/")
        .with_status(503)
        .expect(3) // the original request plus the two retries allowed below
        .create_async()
        .await;

    let response = client_with(2)
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 503);
    broken.assert_async().await;
}

/// Transient failures and rate limits draw on ONE shared budget: a backend
/// alternating 503 and 429 gets `max_retries` retries in total, not a
/// per-tier allowance that multiplies. With `max_retries = 3` exactly four
/// requests go out, and the fourth outcome is returned as-is.
#[tokio::test]
async fn mixed_transient_and_rate_limit_failures_share_one_budget() {
    let mut server = mockito::Server::new_async().await;

    // Each mock serves exactly one request, in creation order.
    let first_503 = server
        .mock("GET", "/")
        .with_status(503)
        .expect(1)
        .create_async()
        .await;
    let first_429 = server
        .mock("GET", "/")
        .with_status(429)
        .expect(1)
        .create_async()
        .await;
    let second_503 = server
        .mock("GET", "/")
        .with_status(503)
        .expect(1)
        .create_async()
        .await;
    let second_429 = server
        .mock("GET", "/")
        .with_status(429)
        .expect(1)
        .create_async()
        .await;
    // Guard: a fifth request would land here and fail the expectation.
    let overflow = server
        .mock("GET", "/")
        .with_status(200)
        .expect(0)
        .create_async()
        .await;

    let response = client_with(3)
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 429);
    first_503.assert_async().await;
    first_429.assert_async().await;
    second_503.assert_async().await;
    second_429.assert_async().await;
    overflow.assert_async().await;
}

/// A non-cloneable (streaming) body cannot be replayed, so it is sent exactly
/// once and the outcome — here a retryable 503 — goes to the caller unretried.
#[tokio::test]
async fn streaming_body_is_sent_exactly_once() {
    let mut server = mockito::Server::new_async().await;

    let broken = server
        .mock("POST", "/")
        .with_status(503)
        .expect(1)
        .create_async()
        .await;

    let stream = futures_util::stream::iter(vec![Ok::<_, std::io::Error>("chunk")]);
    let response = client()
        .post(server.url())
        .body(reqwest::Body::wrap_stream(stream))
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 503);
    broken.assert_async().await;
}

/// A `Retry-After` far in the future means the backend is not coming back within
/// the lifetime of this request, so we decline to wait. Declining is our own
/// decision, so the failure has to explain itself: a bare 429 would look like an
/// ordinary rate-limit error and hide the fact that the server told us when to
/// come back.
#[tokio::test]
async fn explains_itself_when_retry_after_exceeds_the_maximum_wait() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .with_header("retry-after", "3600")
        .expect(1)
        .create_async()
        .await;

    let error = client()
        .get(server.url())
        .send()
        .await
        .expect_err("declining to wait should fail the request");

    let message = error.to_string();
    for expected in ["429", "3600", "300", "not retried"] {
        assert!(
            message.contains(expected),
            "the error should mention {expected}, but was: {message}"
        );
    }
    rate_limited.assert_async().await;
}
