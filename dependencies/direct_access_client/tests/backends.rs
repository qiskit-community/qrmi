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
use assert_json_diff::assert_json_include;
use direct_access_api::{models::Backend, models::BackendStatus, models::Backends, ClientBuilder};
use serde_json::json;

/// Test Client.get_backend_configuration().
/// In this test, test will be run with existing and unknown backend name.
/// Client.get_backend_configuration() should return backend configuration JSON if backend is existing, otherwise
/// it should be failed with error message.
#[tokio::test]
async fn test_backend_configuration() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "dummy": "test",
    });

    let not_found = json!({
        "status_code": 404,
        "title": "Backend unknown not found.",
        "trace": "",
        "errors":[
            {
                "code":"1216",
                "message":"Backend unknown not found.",
                "more_info":"https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    server
        .mock("GET", "/v1/backends/test/configuration")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends/unknown/configuration")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(not_found.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client
        .get_backend_configuration::<serde_json::Value>("test")
        .await
        .unwrap();
    assert_json_include!(actual: actual, expected: expected);

    let failed = client
        .get_backend_configuration::<serde_json::Value>("unknown")
        .await;
    assert!(failed.is_err());
}

/// Test Client.get_backend_properties().
/// In this test, test will be run with existing and unknown backend name.
/// Client.get_backend_properties() should return backend properties JSON if backend is existing, otherwise
/// it should be failed with error message.
#[tokio::test]
async fn test_backend_properties() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "dummy": "test",
    });

    let not_found = json!({
        "status_code": 404,
        "title": "Backend unknown not found.",
        "trace": "",
        "errors":[
            {
                "code":"1216",
                "message":"Backend unknown not found.",
                "more_info":"https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    server
        .mock("GET", "/v1/backends/test/properties")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends/unknown/properties")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(not_found.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client
        .get_backend_properties::<serde_json::Value>("test")
        .await
        .unwrap();
    assert_json_include!(actual: actual, expected: expected);

    let failed = client
        .get_backend_properties::<serde_json::Value>("unknown")
        .await;
    assert!(failed.is_err());
}

/// Test Client.get_backend_pulse_defaults().
/// In this test, test will be run with existing and unknown backend name.
/// Client.get_backend_pulse_defaults() should return the pulse defaults JSON if backend is existing, otherwise
/// it should be failed with error message.
#[tokio::test]
async fn test_backend_pulse_defaults() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "dummy": "test",
    });

    let not_found = json!({
        "status_code": 404,
        "title": "Backend unknown not found.",
        "trace": "",
        "errors":[
            {
                "code":"1216",
                "message":"Backend unknown not found.",
                "more_info":"https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    server
        .mock("GET", "/v1/backends/test/defaults")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends/unknown/defaults")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(not_found.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client
        .get_backend_pulse_defaults::<serde_json::Value>("test")
        .await
        .unwrap();
    assert_json_include!(actual: actual, expected: expected);

    let failed = client
        .get_backend_pulse_defaults::<serde_json::Value>("unknown")
        .await;
    assert!(failed.is_err());
}

/// Test Client.get_backend().
/// In this test, test will be run with existing and unknown backend name.
/// Client.get_backend() should return the backend details JSON if backend is existing, otherwise
/// it should be failed with error message.
#[tokio::test]
async fn test_backend_details() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "name": "test",
        "status": "online",
    });

    let not_found = json!({
        "status_code": 404,
        "title": "Backend unknown not found.",
        "trace": "",
        "errors":[
            {
                "code":"1216",
                "message":"Backend unknown not found.",
                "more_info":"https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            }
        ],
    });

    server
        .mock("GET", "/v1/backends/test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    server
        .mock("GET", "/v1/backends/unknown")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(not_found.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client.get_backend::<Backend>("test").await.unwrap();
    assert_eq!(actual.name, "test");
    assert_eq!(actual.status, BackendStatus::Online);

    let failed = client.get_backend::<Backend>("unknown").await;
    assert!(failed.is_err());
}

/// Test Client.list_backends().
/// This test will compare the deserialized values in Backend object with expected values.
/// All comparisons should be succeeded.
#[tokio::test]
async fn test_list_backends() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "backends": [
            {
                "name": "backend_online",
                "status": "online",
            },
            {
                "name": "backend_offline",
                "status": "offline",
            },
            {
                "name": "backend_paused",
                "status": "paused",
            },
        ],
    });

    server
        .mock("GET", "/v1/backends")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client.list_backends::<serde_json::Value>().await.unwrap();
    assert_json_include!(actual: actual, expected: expected);

    let actual_backends = client.list_backends::<Backends>().await.unwrap();
    let backends = actual_backends.backends;
    assert_eq!("backend_online", backends[0].name);
    assert_eq!(BackendStatus::Online, backends[0].status);
    assert_eq!("backend_offline", backends[1].name);
    assert_eq!(BackendStatus::Offline, backends[1].status);
    assert_eq!("backend_paused", backends[2].name);
    assert_eq!(BackendStatus::Paused, backends[2].status);
}
