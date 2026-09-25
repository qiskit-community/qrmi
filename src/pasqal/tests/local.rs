use crate::{QrmiErrorKind, ResourceStatusCode};

use super::{status_from_warden, PasqalLocal};
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

fn accessible(is_accessible: bool, message: &str) -> pasqal_local_api::AccessibleResponse {
    serde_json::from_value(serde_json::json!({
        "is_accessible": is_accessible,
        "message": message,
    }))
    .unwrap()
}

fn slots(total: u64, used: u64) -> pasqal_local_api::QpuSlotsResponse {
    serde_json::from_value(serde_json::json!({
        "qpu_slots_total": total,
        "qpu_slots_used": used,
        "qpu_slots_available": total - used,
    }))
    .unwrap()
}

#[test]
fn status_reports_warden_qpu_slots() {
    let status = status_from_warden(accessible(true, "Warden ok."), Some(slots(10, 4)));
    assert_eq!(status.status, ResourceStatusCode::Online);
    assert_eq!(status.status_reason.as_deref(), Some("Warden ok."));
    let capacity = status.capacity.unwrap();
    assert_eq!((capacity.available_slots, capacity.max_slots), (6, 10));
    assert_eq!(status.busy, Some(false));
}

#[test]
fn status_is_busy_when_all_slots_are_claimed() {
    let status = status_from_warden(accessible(true, "Warden ok."), Some(slots(10, 10)));
    assert_eq!(status.busy, Some(true));
}

#[test]
fn status_without_slot_management_has_no_capacity() {
    let status = status_from_warden(accessible(true, ""), None);
    assert_eq!(status.status, ResourceStatusCode::Online);
    assert!(status.status_reason.is_none());
    assert!(status.capacity.is_none());
    assert!(status.busy.is_none());
}

#[test]
fn status_is_paused_during_warden_maintenance() {
    let status = status_from_warden(accessible(false, "Calibration"), Some(slots(10, 0)));
    assert_eq!(status.status, ResourceStatusCode::Paused);
    assert_eq!(status.status_reason.as_deref(), Some("Calibration"));
}

#[test]
fn from_config_accepts_acquisition_token() {
    let mut config = slots_config(None);
    config.insert(
        "QRMI_JOB_ACQUISITION_TOKEN".to_string(),
        "00000000-0000-4000-8000-000000000000".to_string(),
    );
    let qrmi = PasqalLocal::from_config("test_backend", config).unwrap();
    assert_eq!(
        qrmi.acquisition_token.as_deref(),
        Some("00000000-0000-4000-8000-000000000000")
    );
    assert!(PasqalLocal::from_config("test_backend", slots_config(None))
        .unwrap()
        .acquisition_token
        .is_none());
}
