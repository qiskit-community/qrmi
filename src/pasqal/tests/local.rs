use crate::{QrmiErrorKind, QuantumResource};

use super::{parse_device_specs, PasqalLocal};
use crate::models::{QuantumResourceInfo, QubitType, ResourceType};
use std::collections::HashMap;

#[test]
fn from_config_accepts_env_style_keys_without_backend_prefix() {
    let config = HashMap::from([
        (
            "QRMI_WARDEN_URL".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    assert!(PasqalLocal::from_config("test_backend", config).is_ok());
}

#[test]
fn from_config_accepts_env_style_keys_without_backend_prefix_lowercased() {
    let config = HashMap::from([
        (
            "qrmi_warden_url".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("qrmi_job_uid".to_string(), "1000".to_string()),
        ("qrmi_job_id".to_string(), "1".to_string()),
    ]);
    assert!(PasqalLocal::from_config("test_backend", config).is_ok());
}

#[test]
fn from_config_errors_on_missing_key() {
    let config = HashMap::from([
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    assert!(PasqalLocal::from_config("test_backend", config).is_err());
}

#[test]
fn job_uid_parsing_fail_raises_qrmi_error() {
    let config = HashMap::from([
        (
            "QRMI_WARDEN_URL".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("QRMI_JOB_UID".to_string(), "abcd".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    let expected = QrmiErrorKind::ParseError;
    let res = PasqalLocal::from_config("backend_name", config);
    let Err(err) = res else {
        panic!("expected an error passing 'abcd' to QRMI_JOB_UID, but got OK");
    };
    assert_eq!(expected, err.kind())
}

#[test]
fn parse_device_specs_maps_pulser_device() {
    let specs = r#"{"name": "FRESNEL", "max_atom_num": 100, "max_runs": 1000, "channels": []}"#;
    let target = serde_json::json!([{"device_type": "FRESNEL", "specs": specs}]).to_string();
    let mut info = QuantumResourceInfo::new(
        "fresnel".to_string(),
        &ResourceType::PasqalLocal,
        QubitType::NeutralAtom,
    );
    parse_device_specs(&mut info, &target).unwrap();
    assert_eq!(info.backend_display_name, Some("FRESNEL".to_string()));
    assert_eq!(info.num_qubits, Some(100));
    assert_eq!(info.max_shots, Some(1000));
    assert_eq!(info.has_queue, Some(true));
}

#[tokio::test]
async fn describe_succeeds_when_warden_unreachable() {
    // Bind then drop to get a port nothing listens on.
    let addr = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let config = HashMap::from([
        ("QRMI_WARDEN_URL".to_string(), format!("http://{addr}")),
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    let mut qrmi = PasqalLocal::from_config("FRESNEL", config).unwrap();
    let info = qrmi.describe().await.unwrap();
    assert_eq!(info.resource_id, "FRESNEL");
    assert_eq!(info.num_qubits, None);
    assert_eq!(info.status, None);
}

#[test]
fn parse_device_specs_rejects_invalid_specs() {
    let target = serde_json::json!([{"device_type": "X", "specs": "not json"}]).to_string();
    let mut info = QuantumResourceInfo::new(
        "x".to_string(),
        &ResourceType::PasqalLocal,
        QubitType::NeutralAtom,
    );
    assert!(parse_device_specs(&mut info, &target).is_err());
}
