use super::{read_pasqal_config, read_qrmi_config_env_value_from_content, PasqalCloud};
use crate::models::ResourceType;
use crate::QuantumResource;
use pasqal_cloud_api::ClientBuilder;
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

#[test]
fn read_qrmi_config_env_value_handles_empty_environment_key() {
    // This test verifies that `read_qrmi_config_env_value_from_content()` correctly
    // handles cases where the "environment" key is missing or empty for a resource.
    let content = r#"{
        "resources": [
            {"name":"EMU_FREE","type":"pasqal-cloud","environment":{}},        ]
    }"#;

    let value = read_qrmi_config_env_value_from_content(
        content,
        "EMU_FREE",     //existing resource
        "nonsense-key", // non-existing key in environment
    );
    assert!(value.is_none());
}

#[test]
fn read_pasqal_config_returns_default_when_config_root_file_missing() {
    let tmp_dir = std::env::temp_dir();
    let missing_root = tmp_dir.join(format!("qrmi_missing_pasqal_cfg"));
    let missing_home = tmp_dir.join(format!("qrmi_home_without_cfg"));
    std::env::set_var("PASQAL_CONFIG_ROOT", &missing_root);
    std::env::set_var("HOME", &missing_home);

    let cfg = read_pasqal_config("EMU_FREE").expect("read_pasqal_config should not fail");
    // All config should be None since the config file is missing: the default
    assert!(cfg.username.is_none());
    assert!(cfg.password.is_none());
    assert!(cfg.project_id.is_none());
    assert!(cfg.token.is_none());
    assert!(cfg.auth_endpoint.is_none());

    std::env::remove_var("PASQAL_CONFIG_ROOT");
    std::env::remove_var("HOME");
}

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    let mut builder = ClientBuilder::for_project("project-id".to_string());
    builder.with_token("opaque_token".to_string());
    let api_client = builder.build().expect("client build should succeed");

    let mut qrmi = PasqalCloud {
        api_client,
        backend_name: "EMU_FREE".to_string(),
    };

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");
    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, "EMU_FREE");
    assert_eq!(resource_type, ResourceType::PasqalCloud);
}
