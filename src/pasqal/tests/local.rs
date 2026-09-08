use super::PasqalLocal;
use crate::QrmiError;
use std::collections::HashMap;

fn valid_config() -> HashMap<String, String> {
    HashMap::from([
        ("backend_name".to_string(), "FRESNEL".to_string()),
        (
            "warden_url".to_string(),
            "http://localhost:8080".to_string(),
        ),
        ("job_uid".to_string(), "42".to_string()),
        ("job_id".to_string(), "job-123".to_string()),
    ])
}

#[test]
fn from_config_builds_resource_from_map() {
    let qrmi = PasqalLocal::from_config(valid_config()).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "FRESNEL");
    assert_eq!(qrmi.job_uid, 42);
    assert_eq!(qrmi.job_id, "job-123");
}

#[test]
fn from_config_missing_backend_name() {
    let mut config = valid_config();
    config.remove("backend_name");
    let err = PasqalLocal::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "backend_name"));
}

#[test]
fn from_config_missing_warden_url() {
    let mut config = valid_config();
    config.remove("warden_url");
    let err = PasqalLocal::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "warden_url"));
}

#[test]
fn from_config_missing_job_uid() {
    let mut config = valid_config();
    config.remove("job_uid");
    let err = PasqalLocal::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "job_uid"));
}

#[test]
fn from_config_missing_job_id() {
    let mut config = valid_config();
    config.remove("job_id");
    let err = PasqalLocal::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "job_id"));
}

#[test]
fn from_config_invalid_job_uid() {
    let mut config = valid_config();
    config.insert("job_uid".to_string(), "not-a-number".to_string());
    let err = PasqalLocal::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::ParseError { name, value, .. }
        if name == "job_uid" && value == "not-a-number"));
}
