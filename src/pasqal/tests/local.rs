use crate::{QrmiErrorKind, ResourceStatusCode};

use super::{status_from_warden, PasqalLocal};
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

#[tokio::test]
async fn from_config_without_job_identity_cannot_acquire() {
    use crate::QuantumResource;

    let config = HashMap::from([(
        "QRMI_WARDEN_URL".to_string(),
        "http://localhost:8006".to_string(),
    )]);
    let mut qrmi = PasqalLocal::from_config("test_backend", config).unwrap();
    let Err(err) = qrmi.acquire().await else {
        panic!("expected acquire() without QRMI_JOB_UID/QRMI_JOB_ID to fail");
    };
    assert_eq!(err.kind(), QrmiErrorKind::InvalidConfig);
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
fn qpu_slots_read_from_backend_env() {
    std::env::set_var("SLOTS_ENV_QRMI_WARDEN_URL", "http://localhost:8006");
    std::env::set_var("SLOTS_ENV_QRMI_JOB_QPU_SLOTS", "4");
    let qrmi = PasqalLocal::new("SLOTS_ENV").unwrap();
    assert_eq!(qrmi.qpu_slots, 4);
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
