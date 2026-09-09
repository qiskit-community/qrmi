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
use crate::models::ResourceType;
use crate::{QrmiError, QuantumResource};
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

fn valid_config() -> HashMap<String, String> {
    HashMap::from([
        ("backend_name".to_string(), "test_eagle".to_string()),
        ("endpoint".to_string(), "http://localhost:8080".to_string()),
        ("iam_api_key".to_string(), "dummy".to_string()),
        ("service_crn".to_string(), "crn:test".to_string()),
        (
            "iam_endpoint".to_string(),
            "http://localhost:8081".to_string(),
        ),
    ])
}

#[test]
fn from_config_builds_resource_from_map() {
    let qrmi = IBMQuantumSystem::from_config(valid_config()).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "test_eagle");
}

#[test]
fn from_config_missing_endpoint() {
    let mut config = valid_config();
    config.remove("endpoint");
    let err = IBMQuantumSystem::from_config(config)
        .map(|_| ())
        .unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "endpoint"));
}

#[test]
fn from_config_partial_s3_keys_are_ignored() {
    // S3 access is only enabled when all required S3 keys are present; a
    // partial set should be silently dropped rather than erroring.
    let mut config = valid_config();
    config.insert("s3_bucket".to_string(), "my-bucket".to_string());
    let qrmi = IBMQuantumSystem::from_config(config).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "test_eagle");
}

#[test]
fn from_config_full_s3_keys_are_accepted() {
    let mut config = valid_config();
    config.insert("aws_access_key_id".to_string(), "AKIA".to_string());
    config.insert("aws_secret_access_key".to_string(), "secret".to_string());
    config.insert("s3_endpoint".to_string(), "http://s3.example".to_string());
    config.insert("s3_bucket".to_string(), "my-bucket".to_string());
    config.insert("s3_region".to_string(), "eu-west-1".to_string());
    let qrmi = IBMQuantumSystem::from_config(config).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "test_eagle");
}
