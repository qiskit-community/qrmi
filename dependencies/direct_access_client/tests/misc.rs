//
// (C) Copyright IBM 2024
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
#![cfg(feature = "api_version")]

mod common;
use direct_access_api::ClientBuilder;
use serde_json::json;

/// Test ClientBuilder.with_api_version().
#[tokio::test]
async fn test_api_version_header() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "dummy": "test",
    });

    server
        .mock("GET", "/v1/backends/test/configuration")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .match_header("ibm-api-version", "1970-01-01")
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url)
        .with_api_version("1970-01-01")
        .build()
        .unwrap();
    let result = client
        .get_backend_configuration::<serde_json::Value>("test")
        .await;
    assert!(result.is_ok());
}
