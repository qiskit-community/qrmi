use crate::QrmiErrorKind;

use super::PasqalLocal;
use std::collections::HashMap;

fn local_config() -> HashMap<String, String> {
    HashMap::from([
        (
            "QRMI_WARDEN_URL".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ])
}

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
fn session_id_prefers_the_given_id() {
    let mut config = local_config();
    config.insert(
        "QRMI_JOB_ACQUISITION_TOKEN".to_string(),
        "session-map".to_string(),
    );
    let qrmi = PasqalLocal::from_config("session_arg", config).unwrap();
    assert_eq!(qrmi.session_id("session-arg").unwrap(), "session-arg");
}

#[test]
fn session_id_uses_config_map_token() {
    let mut config = local_config();
    config.insert(
        "QRMI_JOB_ACQUISITION_TOKEN".to_string(),
        "session-map".to_string(),
    );
    let qrmi = PasqalLocal::from_config("session_map", config).unwrap();
    assert_eq!(qrmi.session_id("").unwrap(), "session-map");
}

#[test]
fn session_id_falls_back_to_environment() {
    let qrmi = PasqalLocal::from_config("session_env", local_config()).unwrap();
    assert!(qrmi.session_id("").is_err());
    std::env::set_var("session_env_QRMI_JOB_ACQUISITION_TOKEN", "session-env");
    let session_id = qrmi.session_id("");
    std::env::remove_var("session_env_QRMI_JOB_ACQUISITION_TOKEN");
    assert_eq!(session_id.unwrap(), "session-env");
}
