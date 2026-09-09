// This code is part of Qiskit.
//
// (C) Copyright IBM 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::super::IQMServer;
use crate::models::ResourceType;
use crate::{QrmiError, QuantumResource};
use iqm_server_api::apis::configuration;
use std::collections::HashMap;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "sirius:mock";

    // Create a service instance with dummy configuration for testing
    // Note: The configuration values are not used in this test since we are only
    // testing the resource_id and resource_type methods.
    // hence their values don't matter
    let mut qrmi = IQMServer {
        config: configuration::Configuration::new(),
        backend_name: BACKEND_NAME.to_string(),
        acquisition_token: None,
        calibration_set_id: "default".to_string(),
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
    assert_eq!(resource_type, ResourceType::IQMServer);
}

fn valid_config() -> HashMap<String, String> {
    HashMap::from([
        ("backend_name".to_string(), "sirius_mock".to_string()),
        (
            "isa_endpoint".to_string(),
            "http://localhost:8080".to_string(),
        ),
        ("isa_token".to_string(), "test-token".to_string()),
    ])
}

#[test]
fn from_config_builds_resource_from_map() {
    let qrmi = IQMServer::from_config(valid_config()).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "sirius:mock");
    assert_eq!(qrmi.calibration_set_id, "default");
    assert_eq!(qrmi.acquisition_token, None);
}

#[test]
fn from_config_honors_optional_keys() {
    let mut config = valid_config();
    config.insert("calibration_set_id".to_string(), "custom".to_string());
    config.insert("acquisition_token".to_string(), "tok-123".to_string());

    let qrmi = IQMServer::from_config(config).expect("from_config should succeed");
    assert_eq!(qrmi.calibration_set_id, "custom");
    assert_eq!(qrmi.acquisition_token, Some("tok-123".to_string()));
}

#[test]
fn from_config_missing_isa_endpoint() {
    let mut config = valid_config();
    config.remove("isa_endpoint");
    let err = IQMServer::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "isa_endpoint"));
}
