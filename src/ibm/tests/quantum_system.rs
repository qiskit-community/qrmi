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
use super::s3_env;
use crate::error::QrmiError;
use crate::models::{Payload, ResourceType};
use crate::QuantumResource;
use std::collections::HashMap;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "test_eagle";
    let mut qrmi = IBMQuantumSystem::from_config(BACKEND_NAME, required_only_config())
        .expect("construction should succeed");

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

fn sampler_payload() -> Payload {
    Payload::QiskitPrimitive {
        input: "{}".to_string(),
        program_id: "sampler".to_string(),
    }
}

#[tokio::test]
async fn task_start_reads_job_timeout_from_config() {
    let mut qrmi = IBMQuantumSystem::from_config("test_eagle", required_only_config())
        .expect("construction should succeed");
    // Fails before any network access.
    let err = qrmi.task_start(sampler_payload()).await.unwrap_err();
    assert!(
        matches!(err, QrmiError::MissingConfigKey(ref key) if key == "QRMI_JOB_TIMEOUT_SECONDS")
    );
}

#[tokio::test]
async fn task_start_rejects_malformed_job_timeout() {
    let mut config = required_only_config();
    config.insert("QRMI_JOB_TIMEOUT_SECONDS".to_string(), "abc".to_string());
    // Construction doesn't read the timeout, so it still succeeds.
    let mut qrmi =
        IBMQuantumSystem::from_config("test_eagle", config).expect("construction should succeed");
    let err = qrmi.task_start(sampler_payload()).await.unwrap_err();
    assert!(
        matches!(err, QrmiError::ParseError { ref name, .. } if name == "QRMI_JOB_TIMEOUT_SECONDS")
    );
}

#[test]
fn s3_env_reads_from_config() {
    let mut config = required_only_config();
    for (key, value) in [
        ("QRMI_IBM_QS_S3_BUCKET", "my-bucket"),
        ("QRMI_IBM_QS_S3_ENDPOINT", "http://localhost:9000"),
        ("QRMI_IBM_QS_AWS_ACCESS_KEY_ID", "dummy"),
        ("QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY", "dummy"),
        ("QRMI_IBM_QS_S3_REGION", "us-east-1"),
    ] {
        config.insert(key.to_string(), value.to_string());
    }
    let qrmi =
        IBMQuantumSystem::from_config("test_eagle", config).expect("construction should succeed");
    let s3 = s3_env(&qrmi.settings).expect("S3 settings should be read from config");
    assert_eq!(s3.bucket, "my-bucket");
}

#[tokio::test]
async fn task_result_reads_s3_settings_from_config() {
    let mut qrmi = IBMQuantumSystem::from_config("test_eagle", required_only_config())
        .expect("construction should succeed");
    // Fails before any network access.
    let err = qrmi
        .task_result("some-task")
        .await
        .err()
        .expect("task_result should fail");
    assert!(matches!(err, QrmiError::MissingConfigKey(ref key) if key == "QRMI_IBM_QS_S3_BUCKET"));
}
