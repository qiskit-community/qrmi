use crate::QrmiErrorKind;

use super::PasqalLocal;
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

fn required_only_config() -> HashMap<String, String> {
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
fn from_config_captures_acquisition_token_from_map() {
    let mut config = required_only_config();
    config.insert(
        "QRMI_JOB_ACQUISITION_TOKEN".to_string(),
        "session".to_string(),
    );
    let qrmi = PasqalLocal::from_config("test_backend", config).unwrap();
    assert_eq!(qrmi.session_id().unwrap(), "session");
}

#[test]
fn from_config_reports_missing_acquisition_token_as_config_key() {
    let qrmi = PasqalLocal::from_config("test_backend", required_only_config()).unwrap();
    let err = qrmi.session_id().unwrap_err();
    assert_eq!(err.kind(), QrmiErrorKind::MissingConfigKey);
    assert!(err.to_string().contains("QRMI_JOB_ACQUISITION_TOKEN"));
}
