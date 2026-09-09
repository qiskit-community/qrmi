// This code is part of Qiskit.
//
// (C) Copyright Alice and Bob 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::super::AliceBobFelis;
use crate::models::ResourceType;
use crate::{QrmiError, QuantumResource};
use std::collections::HashMap;

// Base64 of "user:pass" -- Felis API keys are basic auth credentials in disguise.
const DUMMY_API_KEY: &str = "dXNlcjpwYXNz";

fn valid_config() -> HashMap<String, String> {
    HashMap::from([
        (
            "backend_name".to_string(),
            "ab_emu_40q_physical_cats".to_string(),
        ),
        ("api_key".to_string(), DUMMY_API_KEY.to_string()),
        (
            "base_endpoint".to_string(),
            "http://localhost:8080".to_string(),
        ),
    ])
}

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "ab_emu_40q_physical_cats";
    let mut qrmi = AliceBobFelis::from_config(valid_config()).expect("from_config should succeed");

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");
    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, BACKEND_NAME);
    assert_eq!(resource_type, ResourceType::AliceBobFelis);
}

#[test]
fn from_config_builds_resource_from_map() {
    let qrmi = AliceBobFelis::from_config(valid_config()).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "ab_emu_40q_physical_cats");
    assert_eq!(qrmi.felis_target, "EMU:40Q:PHYSICAL_CATS");
}

#[test]
fn from_config_missing_api_key() {
    let mut config = valid_config();
    config.remove("api_key");
    let err = AliceBobFelis::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "api_key"));
}

#[test]
fn from_config_missing_base_endpoint() {
    let mut config = valid_config();
    config.remove("base_endpoint");
    let err = AliceBobFelis::from_config(config).map(|_| ()).unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "base_endpoint"));
}
