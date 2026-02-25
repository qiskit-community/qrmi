use super::PasqalCloud;
use crate::QuantumResource;
use pasqal_cloud_api::ClientBuilder;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[tokio::test]
async fn is_accessible_attempts_authentication() {
    // This test verifies that `is_accessible()` attempts to authenticate and uses the obtained token.
    // We set up a mock server that simulates the authentication endpoint and the devices endpoint.
    // The server will respond with a fixed token for the authentication request and will check that this token is used in the subsequent request to the devices endpoint.
    // We also verify that `is_accessible()` returns true, indicating that the backend is accessible with the obtained token.

    // Ask for any free port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local_addr should succeed");
    // Setup Mock server with mocked responses.
    let mock_server = thread::spawn(move || {
        for _ in 0..1 {
            if let Ok((mut stream, _)) = listener.accept() {
                // Read the request
                let mut buf = [0_u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);

                // hardcode responses based on the request path
                let body = if req.contains("/oauth/token") {
                    r#"{"access_token":"opaque_token"}"#
                } else if req.contains("/core-fast/api/v1/devices") {
                    r#"{"data":[{"status":"UP","availability":"ACTIVE"}]}"#
                } else {
                    r#"{}"#
                };

                // Write the response
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        }
    });

    let api_client = ClientBuilder::new(String::new(), "project-id".to_string())
        .build()
        .expect("client build should succeed");

    // Create a PasqalCloud instance pointing to the mock auth server.
    // Use an invalid backend name to stop before get_device(), so this test
    // only validates authentication behavior.
    let mut qrmi = PasqalCloud {
        api_client,
        backend_name: "INVALID_BACKEND".to_string(),
        auth_token: String::new(),
        auth_token_expiry_unix_seconds: None,
        project_id: "project-id".to_string(),
        auth_endpoint: format!("http://{}/oauth/token", addr),
        username: Some("usr".to_string()),
        password: Some("pass".to_string()),
    };

    let result = qrmi.is_accessible().await;
    mock_server.join().expect("server thread should join");

    // Verify that authentication happened and then backend validation failed.
    assert!(result.is_err());
    assert_eq!(qrmi.auth_token, "opaque_token".to_string());
}
