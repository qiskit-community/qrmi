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

mod common;
use direct_access_api::ClientBuilder;
use serde_json::json;

/// Test Client.get_service_version().
/// Actual version text should be same as the expected value (="1.0.0").
#[tokio::test]
async fn test_version() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let body_json = json!({
        "version": "1.0.0",
    });

    let mock = server
        .mock("GET", "/version")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body_json.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let version = client.get_service_version().await.unwrap();
    assert_eq!("1.0.0", version);

    mock.assert_async().await;
}
