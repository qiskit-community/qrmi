//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
mod common;
use direct_access_api::{models::Backends, AuthMethod, ClientBuilder};
use serde_json::json;

/// Test Client.list_api_versions().
#[tokio::test]
async fn test_list_api_versions() {
    common::setup();

    let mut server = mockito::Server::new_async().await;
    let versions_body = json!({
        "versions": ["2025-08-01", "2025-08-15"]
    });

    let versions_mock = server
        .mock("GET", "/versions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(versions_body.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let mut builder = ClientBuilder::new(&base_url);
    let client = builder.build().unwrap();

    let versions = client.list_api_versions().await.unwrap();
    assert_eq!(
        versions,
        vec!["2025-08-01".to_string(), "2025-08-15".to_string()]
    );

    let token_mock = server
        .mock("POST", "/identity/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "access_token": "dummy-token",
                "token_type": "Bearer",
                "expires_in": 3600
            })
            .to_string(),
        )
        .create_async()
        .await;

    builder
        .with_api_version(versions[0].clone()) //"2025801"
        .with_auth(AuthMethod::IbmCloudIam {
            apikey: "demoapikey1".to_string(),
            service_crn: "crn:v1:local:daa_sim".to_string(),
            iam_endpoint_url: base_url,
        });

    let client_with_version = builder.build().unwrap();

    let backends_body = json!({
        "backends": [
            {
                "name": "backend_online",
                "status": "online",
            },
            {
                "name": "backend_offline",
                "status": "offline",
            },
        ],
    });

    let backends_mock = server
        .mock("GET", "/v1/backends")
        .match_header("ibm-api-version", "2025-08-01")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(backends_body.to_string())
        .create_async()
        .await;

    let backends: Backends = client_with_version
        .list_backends::<Backends>()
        .await
        .unwrap();
    assert_eq!(backends.backends.len(), 2);
    assert!(backends.backends.iter().any(|b| b.name == "backend_online"));
    assert!(backends
        .backends
        .iter()
        .any(|b| b.name == "backend_offline"));

    versions_mock.assert_async().await;
    token_mock.assert_async().await;
    backends_mock.assert_async().await;
}
