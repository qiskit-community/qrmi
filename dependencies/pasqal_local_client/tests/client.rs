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

use pasqal_local_api::{Client, ClientBuilder};
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
async fn munge_token_is_bound_to_the_request() {
    let mut server = mockito::Server::new_async().await;
    // Echo the token back as the session id, to decode it below
    server
        .mock("POST", "/sessions")
        .with_body_from_request(|req| {
            let token = req.header("x-munge-cred")[0].to_str().unwrap().to_string();
            json!({ "id": token }).to_string().into()
        })
        .create_async()
        .await;

    let Ok(session) = client_for(&server).create_session(1000, "1").await else {
        eprintln!("skipped: no MUNGE daemon to sign the request");
        return;
    };
    let mut unmunge = std::process::Command::new("unmunge")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("unmunge should run where munged does");
    use std::io::Write;
    unmunge
        .stdin
        .take()
        .unwrap()
        .write_all(session.id.as_bytes())
        .unwrap();
    let decoded = String::from_utf8(unmunge.wait_with_output().unwrap().stdout).unwrap();
    assert!(
        decoded.ends_with(&format!("POST {}/sessions", server.url())),
        "{decoded}"
    );
}
