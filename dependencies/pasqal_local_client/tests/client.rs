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

use pasqal_local_api::{Client, ClientBuilder, ResourceState};
use serde_json::json;

fn client_for(server: &mockito::Server) -> Client {
    ClientBuilder::new(server.url())
        .build()
        .expect("client should build")
}

#[tokio::test]
async fn get_accessible_needs_no_credentials() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/accessible")
        .match_header("x-munge-cred", mockito::Matcher::Missing)
        .with_status(200)
        .with_body(json!({"is_accessible": false, "message": "QPU in maintenance"}).to_string())
        .create_async()
        .await;

    let accessible = client_for(&server)
        .get_accessible()
        .await
        .expect("get_accessible should succeed");

    mock.assert_async().await;
    assert!(!accessible.is_accessible);
    assert_eq!(accessible.message, "QPU in maintenance");
}

#[tokio::test]
async fn failed_request_reports_status_and_body() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/accessible")
        .with_status(503)
        .with_body(r#"{"detail":"QPU unreachable"}"#)
        .create_async()
        .await;

    let err = client_for(&server)
        .get_accessible()
        .await
        .expect_err("get_accessible should fail");

    let message = err.to_string();
    assert!(message.contains("503"));
    assert!(message.contains("QPU unreachable"));
}

#[tokio::test]
async fn get_status_summary_needs_no_credentials() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/status/summary")
        .match_header("x-munge-cred", mockito::Matcher::Missing)
        .with_status(200)
        .with_body(
            json!({
                "status": "paused",
                "status_reason": "calibration",
                "healthy": true,
                "busy": true,
                "pending_job_count": 3,
                "capacity": {"available_slots": 3, "max_slots": 5},
            })
            .to_string(),
        )
        .create_async()
        .await;

    let status = client_for(&server)
        .get_status_summary()
        .await
        .expect("get_status_summary should succeed");

    mock.assert_async().await;
    assert_eq!(status.status, ResourceState::Paused);
    assert_eq!(status.status_reason.as_deref(), Some("calibration"));
    assert_eq!(status.healthy, Some(true));
    assert!(status.busy);
    assert_eq!(status.pending_job_count, 3);
    let capacity = status.capacity.expect("capacity must not be None");
    assert_eq!(capacity.available_slots, 3);
    assert_eq!(capacity.max_slots, 5);
}
