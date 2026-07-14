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

fn slots_config(slots: Option<&str>) -> HashMap<String, String> {
    let mut config = HashMap::from([
        (
            "QRMI_WARDEN_URL".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    if let Some(slots) = slots {
        config.insert("QRMI_JOB_QPU_SLOTS".to_string(), slots.to_string());
    }
    config
}

#[test]
fn qpu_slots_default_to_one() {
    let qrmi = PasqalLocal::from_config("test_backend", slots_config(None)).unwrap();
    assert_eq!(qrmi.qpu_slots, 1);
}

#[test]
fn qpu_slots_read_from_config() {
    let qrmi = PasqalLocal::from_config("test_backend", slots_config(Some("5"))).unwrap();
    assert_eq!(qrmi.qpu_slots, 5);
}

#[test]
fn qpu_slots_reject_zero() {
    let Err(err) = PasqalLocal::from_config("test_backend", slots_config(Some("0"))) else {
        panic!("expected QRMI_JOB_QPU_SLOTS=0 to be rejected");
    };
    assert_eq!(err.kind(), QrmiErrorKind::InvalidConfig);
}

#[test]
fn qpu_slots_reject_negative() {
    assert!(PasqalLocal::from_config("test_backend", slots_config(Some("-1"))).is_err());
}

#[test]
fn qpu_slots_reject_non_integer() {
    let Err(err) = PasqalLocal::from_config("test_backend", slots_config(Some("two"))) else {
        panic!("expected QRMI_JOB_QPU_SLOTS=two to be rejected");
    };
    assert_eq!(err.kind(), QrmiErrorKind::ParseError);
}
