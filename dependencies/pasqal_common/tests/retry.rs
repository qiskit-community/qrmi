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

use pasqal_common::with_retry;
use reqwest_middleware::ClientBuilder as MiddlewareClientBuilder;

fn client() -> reqwest_middleware::ClientWithMiddleware {
    with_retry(MiddlewareClientBuilder::new(reqwest::Client::new())).build()
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

/// When `Retry-After` exceeds the remaining budget the middleware gives up
/// immediately, surfacing the 429 to the caller after a single request. The
/// advertised 3600s is well beyond the default rate-limit budget (300s).
#[tokio::test]
async fn gives_up_when_retry_after_exceeds_budget() {
    let mut server = mockito::Server::new_async().await;

    let rate_limited = server
        .mock("GET", "/")
        .with_status(429)
        .with_header("retry-after", "3600")
        .expect(1)
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
