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

//! Integration tests for the rate-limit retry middleware against a mock server.

use std::time::{Duration, Instant};

use pasqal_common::{with_retry, DEFAULT_MAX_RETRIES};
use reqwest_middleware::ClientBuilder as MiddlewareClientBuilder;

fn client_with(max_retries: u32) -> reqwest_middleware::ClientWithMiddleware {
    with_retry(
        MiddlewareClientBuilder::new(reqwest::Client::new()),
        max_retries,
    )
    .build()
}

fn client() -> reqwest_middleware::ClientWithMiddleware {
    client_with(DEFAULT_MAX_RETRIES)
}

/// A 429 carrying `Retry-After: 0` should be retried immediately and succeed
/// once the server starts returning 200.
#[tokio::test]
async fn retries_429_with_retry_after_then_succeeds() {
    let mut server = mockito::Server::new_async().await;

    // Created first, so it wins while it still has unmet expectations.
    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .with_header("retry-after", "0")
        .expect(1)
        .create_async()
        .await;
    // Created last, so it serves every request after `rate_limited` is satisfied.
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

/// Without a `Retry-After` header the middleware falls back to its exponential
/// backoff, which starts at 5 seconds, and stops after `max_retries` attempts.
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
        elapsed >= Duration::from_secs(5),
        "the retry should have waited out the 5s base backoff, but took only {elapsed:?}"
    );
}

/// A retry count of zero means the request is made once and not retried.
#[tokio::test]
async fn zero_max_retries_makes_a_single_request() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .expect(1)
        .create_async()
        .await;

    let response = client_with(0)
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 429);
    rate_limited.assert_async().await;
}

/// A retry count of zero disables the `Retry-After` tier too: even though that
/// tier normally budgets its single retry separately from the backoff count, a
/// caller who asked for zero retries gets exactly one request and no sleep.
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

    let response = client_with(0)
        .get(server.url())
        .send()
        .await
        .expect("request should complete");

    assert_eq!(response.status(), 429);
    rate_limited.assert_async().await;
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
