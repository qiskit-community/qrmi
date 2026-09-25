// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::super::IBMQuantumSystem;
use crate::error::QrmiError;
use crate::models::ResourceType;
use crate::QuantumResource;
use quantum_system_api::ClientBuilder;
use std::collections::HashMap;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "test_eagle";
    let api_client = ClientBuilder::new("http://127.0.0.1:8080")
        .build()
        .expect("client build should succeed");
    let mut qrmi = IBMQuantumSystem {
        api_client,
        backend_name: BACKEND_NAME.to_string(),
        task_settings: Default::default(),
    };

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");
    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, BACKEND_NAME);
    assert_eq!(resource_type, ResourceType::IBMQuantumSystem);
}

fn required_only_config() -> HashMap<String, String> {
    HashMap::from([
        (
            "QRMI_IBM_QS_ENDPOINT".to_string(),
            "http://localhost".to_string(),
        ),
        ("QRMI_IBM_QS_IAM_APIKEY".to_string(), "dummy".to_string()),
        ("QRMI_IBM_QS_SERVICE_CRN".to_string(), "dummy".to_string()),
        (
            "QRMI_IBM_QS_IAM_ENDPOINT".to_string(),
            "http://localhost".to_string(),
        ),
    ])
}

#[test]
fn from_config_works_without_s3_keys() {
    assert!(IBMQuantumSystem::from_config("test_eagle", required_only_config()).is_ok());
}

#[test]
fn from_config_accepts_env_style_keys_lowercased() {
    let config: HashMap<String, String> = required_only_config()
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    assert!(IBMQuantumSystem::from_config("test_eagle", config).is_ok());
}

#[test]
fn from_config_requires_all_s3_keys_together() {
    let mut config = required_only_config();
    // Only one of the five S3 keys set -- should be treated as "no S3", not an error.
    config.insert("QRMI_IBM_QS_S3_BUCKET".to_string(), "my-bucket".to_string());
    assert!(IBMQuantumSystem::from_config("test_eagle", config).is_ok());
}

#[test]
fn from_config_missing_required_key_errors() {
    let config = HashMap::from([(
        "QRMI_IBM_QS_ENDPOINT".to_string(),
        "http://localhost".to_string(),
    )]);
    assert!(IBMQuantumSystem::from_config("test_eagle", config).is_err());
}

#[test]
fn from_config_captures_task_settings_from_map() {
    let mut config = required_only_config();
    config.insert("QRMI_JOB_TIMEOUT_SECONDS".to_string(), "600".to_string());
    for (key, value) in [
        ("QRMI_IBM_QS_S3_BUCKET", "bucket"),
        ("QRMI_IBM_QS_S3_ENDPOINT", "http://localhost:9000"),
        ("QRMI_IBM_QS_AWS_ACCESS_KEY_ID", "id"),
        ("QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY", "secret"),
        ("QRMI_IBM_QS_S3_REGION", "us-east"),
    ] {
        config.insert(key.to_string(), value.to_string());
    }
    let qrmi = IBMQuantumSystem::from_config("test_eagle", config).unwrap();

    assert_eq!(qrmi.task_settings.timeout_secs().unwrap(), 600);
    assert_eq!(qrmi.task_settings.s3().unwrap().bucket, "bucket");
}

#[test]
fn from_config_reports_missing_task_settings_as_config_keys() {
    let qrmi = IBMQuantumSystem::from_config("test_eagle", required_only_config()).unwrap();

    assert!(matches!(
        qrmi.task_settings.timeout_secs(),
        Err(QrmiError::MissingConfigKey(key)) if key == "QRMI_JOB_TIMEOUT_SECONDS"
    ));
    assert!(matches!(
        qrmi.task_settings.s3(),
        Err(QrmiError::MissingConfigKey(key)) if key == "QRMI_IBM_QS_S3_BUCKET"
    ));
}

#[test]
fn from_config_rejects_invalid_timeout() {
    let mut config = required_only_config();
    config.insert("QRMI_JOB_TIMEOUT_SECONDS".to_string(), "soon".to_string());
    assert!(matches!(
        IBMQuantumSystem::from_config("test_eagle", config),
        Err(QrmiError::ParseError { .. })
    ));
}
