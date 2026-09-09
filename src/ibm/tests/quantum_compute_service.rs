// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2026
// (C) Copyright UKRI-STFC (Hartree Centre) 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::super::IBMQuantumComputeService;
use crate::models::ResourceType;
use crate::{QrmiError, QuantumResource};
use quantum_compute_client::apis::configuration;
use std::collections::HashMap;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "ibm_torino";

    // Create a service instance with dummy configuration for testing
    // Note: The configuration values are not used in this test since we are only
    // testing the resource_id and resource_type methods.
    // hence their values don't matter
    let mut qrmi = IBMQuantumComputeService {
        config: configuration::Configuration::new(),
        backend_name: BACKEND_NAME.to_string(),
        session_id: None,
        calibration_id: None,
        timeout_secs: None,
        session_mode: "dedicated".to_string(),
        session_max_ttl: 28800,
        api_key: "dummy".to_string(),
        iam_endpoint: "http://127.0.0.1:8080".to_string(),
        token_expiration: 0,
        token_lifetime: 0,
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
    assert_eq!(resource_type, ResourceType::IBMQuantumComputeService);
}

fn valid_config() -> HashMap<String, String> {
    HashMap::from([
        ("backend_name".to_string(), "ibm_torino".to_string()),
        ("endpoint".to_string(), "http://localhost:8080".to_string()),
        (
            "iam_endpoint".to_string(),
            "http://localhost:8081".to_string(),
        ),
        ("iam_api_key".to_string(), "dummy".to_string()),
        ("service_crn".to_string(), "crn:test".to_string()),
    ])
}

#[test]
fn from_config_builds_resource_from_map() {
    let qrmi =
        IBMQuantumComputeService::from_config(valid_config()).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "ibm_torino");
    assert_eq!(qrmi.session_mode, "dedicated");
    assert_eq!(qrmi.session_max_ttl, 28800);
    assert_eq!(qrmi.timeout_secs, None);
    assert_eq!(qrmi.session_id, None);
}

#[test]
fn from_config_honors_optional_keys() {
    let mut config = valid_config();
    config.insert("session_mode".to_string(), "batch".to_string());
    config.insert("session_max_ttl".to_string(), "100".to_string());
    config.insert("timeout_secs".to_string(), "30".to_string());
    config.insert("session_id".to_string(), "sess-123".to_string());

    let qrmi = IBMQuantumComputeService::from_config(config).expect("from_config should succeed");
    assert_eq!(qrmi.session_mode, "batch");
    assert_eq!(qrmi.session_max_ttl, 100);
    assert_eq!(qrmi.timeout_secs, Some(30));
    assert_eq!(qrmi.session_id, Some("sess-123".to_string()));
}

#[test]
fn from_config_missing_service_crn() {
    let mut config = valid_config();
    config.remove("service_crn");
    let err = IBMQuantumComputeService::from_config(config)
        .map(|_| ())
        .unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "service_crn"));
}

#[test]
fn from_config_invalid_session_max_ttl_falls_back_to_default() {
    // Mirrors `new()`'s env-var parsing: an unparseable value is ignored
    // rather than rejected.
    let mut config = valid_config();
    config.insert("session_max_ttl".to_string(), "not-a-number".to_string());
    let qrmi = IBMQuantumComputeService::from_config(config).expect("from_config should succeed");
    assert_eq!(qrmi.session_max_ttl, 28800);
}
