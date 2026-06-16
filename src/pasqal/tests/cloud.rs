use super::PasqalCloud;
use crate::models::ResourceType;
use crate::pasqal::cloud_config::{
    expand_env_vars, pasqal_config_path_from_root, read_pasqal_config, PasqalConfig,
};
use crate::QuantumResource;
use pasqal_cloud_api::ClientBuilder;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::thread;

// Ensure that tests that manipulate environment variables are not run in parallel to avoid interference between them.
fn env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK.get_or_init(|| Mutex::new(()))
}

#[tokio::test]
async fn is_accessible_no_authentication() {
    // This test verifies that `is_accessible()` does not need to authenticate.
    // We set up a mock server that simulates the devices endpoint.
    // We also verify that `is_accessible()` returns true, indicating that the backend is accessible.

    // Ask for any free port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local_addr should succeed");
    // Setup Mock server with mocked responses.
    let mock_server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0_u8; 4096];
            // Read the request
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let req_lower = req.to_ascii_lowercase();

            // Verify endpoint and no authentication
            assert!(req.contains("/core-fast/api/v1/devices"));
            assert!(!req_lower.contains("authorization: bearer"));

            // Hardcode response
            let body = r#"{"data":[{"status":"UP","availability":"ACTIVE"}]}"#;

            // Write the response
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    let mut builder = ClientBuilder::new("project-id".to_string());
    builder.with_base_url(format!("http://{}", addr));
    let api_client = builder.build().expect("client build should succeed");

    // Create a PasqalCloud instance pointing to the mock server
    let mut qrmi = PasqalCloud {
        api_client,
        backend_name: "EMU_FREE".to_string(),
        task_kinds: std::collections::HashMap::new(),
    };

    let accessible = qrmi
        .is_accessible()
        .await
        .expect("is_accessible should succeed");
    mock_server.join().expect("server thread should join");

    // Verify that `is_accessible()` returns true
    assert!(accessible);
}

#[test]
fn resolve_pasqal_credentials_prefers_environment_variables() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    std::env::set_var("PASQAL_USERNAME", "env-user");
    std::env::set_var("PASQAL_PASSWORD", "env-pass");

    let cfg = PasqalConfig {
        username: Some("config-user".to_string()),
        password: Some("config-pass".to_string()),
        client_id: None,
        client_secret: None,
        token: None,
        project_id: None,
        auth_endpoint: None,
    };
    let (username, password) = cfg.credentials();

    assert_eq!(username.as_deref(), Some("env-user"));
    assert_eq!(password.as_deref(), Some("env-pass"));

    std::env::remove_var("PASQAL_USERNAME");
    std::env::remove_var("PASQAL_PASSWORD");
}

#[test]
fn resolve_pasqal_service_account_credentials_prefers_environment_variables() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    std::env::set_var("EMU_FREE_QRMI_PASQAL_CLOUD_CLIENT_ID", "env-client-id");
    std::env::set_var(
        "EMU_FREE_QRMI_PASQAL_CLOUD_CLIENT_SECRET",
        "env-client-secret",
    );

    let cfg = PasqalConfig {
        username: None,
        password: None,
        client_id: Some("config-client-id".to_string()),
        client_secret: Some("config-client-secret".to_string()),
        token: None,
        project_id: None,
        auth_endpoint: None,
    };
    let (client_id, client_secret) = cfg.service_account_credentials("EMU_FREE");

    assert_eq!(client_id.as_deref(), Some("env-client-id"));
    assert_eq!(client_secret.as_deref(), Some("env-client-secret"));

    std::env::remove_var("EMU_FREE_QRMI_PASQAL_CLOUD_CLIENT_ID");
    std::env::remove_var("EMU_FREE_QRMI_PASQAL_CLOUD_CLIENT_SECRET");
}

#[test]
fn pasqal_cloud_new_allows_missing_project_id_and_auth() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    let vars = [
        "PASQAL_USERNAME",
        "PASQAL_PASSWORD",
        "PASQAL_CONFIG_ROOT",
        "EMU_FREE_QRMI_PASQAL_CONFIG_ROOT",
        "EMU_FREE_PASQAL_CONFIG_ROOT",
        "EMU_FREE_QRMI_PASQAL_CLOUD_PROJECT_ID",
        "EMU_FREE_QRMI_PASQAL_CLOUD_AUTH_TOKEN",
        "EMU_FREE_QRMI_PASQAL_CLOUD_CLIENT_ID",
        "EMU_FREE_QRMI_PASQAL_CLOUD_CLIENT_SECRET",
    ];
    let old_vars = vars.map(|key| (key, std::env::var(key).ok()));
    let old_home = std::env::var("HOME").ok();
    let home =
        std::env::temp_dir().join(format!("qrmi_pasqal_no_auth_home_{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).expect("home dir should be created");

    for key in vars {
        std::env::remove_var(key);
    }
    std::env::set_var("HOME", &home);

    let qrmi = PasqalCloud::new("EMU_FREE").expect("PasqalCloud should build without auth");
    assert_eq!(qrmi.backend_name, "EMU_FREE");

    for (key, value) in old_vars {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
    match old_home {
        Some(home) => std::env::set_var("HOME", home),
        None => std::env::remove_var("HOME"),
    }
    let _ = fs::remove_dir_all(&home);
}

fn write_pasqal_config(root: &Path, content: &str) {
    let config_dir = root.join(".pasqal");
    fs::create_dir_all(&config_dir).expect("config dir should be created");
    fs::write(config_dir.join("config"), content).expect("config should be written");
}

#[test]
fn expand_env_vars_handles_path_expansion_cases() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    std::env::set_var("USER", "<user>");
    std::env::set_var("PROJECT_DIR", "PROJ_123");
    std::env::remove_var("UNKNOWN");

    assert_eq!(
        expand_env_vars("$USER").expect("$USER should expand"),
        "<user>"
    );
    assert_eq!(
        expand_env_vars("${USER}").expect("${USER} should expand"),
        "<user>"
    );
    assert_eq!(
        expand_env_vars("$UNKNOWN").expect("unset $UNKNOWN should expand"),
        ""
    );
    assert_eq!(
        expand_env_vars("${UNKNOWN}").expect("unset ${UNKNOWN} should expand"),
        ""
    );
    assert_eq!(
        expand_env_vars("$$USER").expect("$$ should escape $"),
        "$USER"
    );
    assert_eq!(
        expand_env_vars("/work/$PROJECT_DIR/$USER/config").expect("mixed value should expand"),
        "/work/PROJ_123/<user>/config"
    );

    assert!(expand_env_vars("${USER").is_err());
    assert!(expand_env_vars("${}").is_err());
    assert!(expand_env_vars("${USER-NAME}").is_err());
    assert_eq!(
        expand_env_vars("$").expect("bare $ should be preserved"),
        "$"
    );
    assert_eq!(expand_env_vars("$-").expect("$- should be preserved"), "$-");
}

#[test]
fn pasqal_config_path_from_root_expands_home_and_environment_variables() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    let old_home = std::env::var("HOME").ok();
    let old_user = std::env::var("USER").ok();
    std::env::set_var("HOME", "/home/aleks");
    std::env::set_var("USER", "aleks");

    assert_eq!(
        pasqal_config_path_from_root("~/configs")
            .expect("path should expand")
            .as_deref(),
        Some(Path::new("/home/aleks/configs/.pasqal/config"))
    );
    assert_eq!(
        pasqal_config_path_from_root("/work/$USER/configs")
            .expect("path should expand")
            .as_deref(),
        Some(Path::new("/work/aleks/configs/.pasqal/config"))
    );
    assert_eq!(
        pasqal_config_path_from_root("/work/${USER}/configs")
            .expect("path should expand")
            .as_deref(),
        Some(Path::new("/work/aleks/configs/.pasqal/config"))
    );
    assert!(pasqal_config_path_from_root("/work/${USER/configs").is_err());

    match old_home {
        Some(home) => std::env::set_var("HOME", home),
        None => std::env::remove_var("HOME"),
    }
    match old_user {
        Some(user) => std::env::set_var("USER", user),
        None => std::env::remove_var("USER"),
    }
}

#[test]
fn read_pasqal_config_uses_backend_config_root_env() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    let root = std::env::temp_dir().join(format!(
        "qrmi_pasqal_cfg_{}_backend_root",
        std::process::id()
    ));
    let home = std::env::temp_dir().join(format!("qrmi_pasqal_cfg_{}_home", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&home);
    write_pasqal_config(
        &root,
        "token=from-backend-root\nproject_id=project\nclient_id=client\nclient_secret=secret\n",
    );

    std::env::remove_var("PASQAL_CONFIG_ROOT");
    std::env::set_var("EMU_FREE_PASQAL_CONFIG_ROOT", &root);
    std::env::set_var("HOME", &home);

    let cfg = read_pasqal_config("EMU_FREE").expect("read_pasqal_config should not fail");

    assert_eq!(cfg.token.as_deref(), Some("from-backend-root"));
    assert_eq!(cfg.project_id.as_deref(), Some("project"));
    assert_eq!(cfg.client_id.as_deref(), Some("client"));
    assert_eq!(cfg.client_secret.as_deref(), Some("secret"));

    std::env::remove_var("EMU_FREE_PASQAL_CONFIG_ROOT");
    std::env::remove_var("HOME");
    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn read_pasqal_config_returns_default_when_config_root_file_missing() {
    let _guard = env_lock().lock().expect("env lock should not be poisoned");
    let tmp_dir = std::env::temp_dir();
    let missing_root = tmp_dir.join("qrmi_missing_pasqal_cfg");
    let missing_home = tmp_dir.join("qrmi_home_without_cfg");
    std::env::set_var("PASQAL_CONFIG_ROOT", &missing_root);
    std::env::set_var("HOME", &missing_home);

    let cfg = read_pasqal_config("EMU_FREE").expect("read_pasqal_config should not fail");
    // All config should be None since the config file is missing: the default
    assert!(cfg.username.is_none());
    assert!(cfg.password.is_none());
    assert!(cfg.client_id.is_none());
    assert!(cfg.client_secret.is_none());
    assert!(cfg.project_id.is_none());
    assert!(cfg.token.is_none());
    assert!(cfg.auth_endpoint.is_none());

    std::env::remove_var("PASQAL_CONFIG_ROOT");
    std::env::remove_var("HOME");
}

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    let mut builder = ClientBuilder::new("project-id".to_string());
    builder.with_token("opaque_token".to_string());
    let api_client = builder.build().expect("client build should succeed");

    let mut qrmi = PasqalCloud {
        api_client,
        backend_name: "EMU_FREE".to_string(),
        task_kinds: std::collections::HashMap::new(),
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

#[test]
fn detect_cudaq_payload_shape() {
    assert!(PasqalCloud::is_cudaq_sequence(
        r#"{"setup":{},"hamiltonian":{}}"#
    ));
    assert!(!PasqalCloud::is_cudaq_sequence(
        r#"{"name":"pulser-sequence"}"#
    ));
}

#[test]
fn normalize_cudaq_result_normalizes_all_supported_counter_shapes() {
    let expected = serde_json::json!({
        "counter": {
            "000": 47,
            "001": 16
        }
    });

    let cases = vec![
        serde_json::json!({
            "counter": {
                "000": 47,
                "001": 16
            }
        }),
        serde_json::json!({
            "counter": {
                "job-123": {
                    "000": 47,
                    "001": 16
                }
            }
        }),
        serde_json::json!({
            "job-123": {
                "counter": {
                    "000": 47,
                    "001": 16
                }
            }
        }),
        serde_json::json!({
            "000": 47,
            "001": 16
        }),
    ];

    for case in cases {
        let normalized = PasqalCloud::normalize_cudaq_result(&case);
        let parsed: serde_json::Value =
            serde_json::from_str(&normalized).expect("normalized result must be valid json");
        assert_eq!(parsed, expected);
        let count_000 = parsed["counter"]["000"]
            .as_i64()
            .expect("counter['000'] should be an integer");
        assert_eq!(count_000, 47);
    }
}
