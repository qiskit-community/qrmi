use super::AliceBobFelis;
use std::collections::HashMap;

#[test]
fn from_config_builds_resource_from_map() {
    let config = HashMap::from([
        (
            "QRMI_AB_FELIS_API_KEY".to_string(),
            "aWQ6c2VjcmV0".to_string(),
        ),
        (
            "QRMI_AB_FELIS_BASE_ENDPOINT".to_string(),
            "http://localhost:8080".to_string(),
        ),
    ]);
    assert!(AliceBobFelis::from_config("ab_emu_40q_physical_cats", config).is_ok());
}

#[test]
fn from_config_accepts_env_style_keys_lowercased() {
    let config = HashMap::from([
        (
            "qrmi_ab_felis_api_key".to_string(),
            "aWQ6c2VjcmV0".to_string(),
        ),
        (
            "qrmi_ab_felis_base_endpoint".to_string(),
            "http://localhost:8080".to_string(),
        ),
    ]);
    assert!(AliceBobFelis::from_config("ab_emu_40q_physical_cats", config).is_ok());
}

#[test]
fn from_config_missing_required_key_errors() {
    let config = HashMap::from([("QRMI_AB_FELIS_API_KEY".to_string(), "id:secret".to_string())]);
    assert!(AliceBobFelis::from_config("ab_emu_40q_physical_cats", config).is_err());
}
