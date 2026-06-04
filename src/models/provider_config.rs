// This code is part of Qiskit.
//
// (C) Copyright IBM 2025-2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Config file model for `ResourceProvider` (version 2 format).

use serde::Deserialize;
use std::collections::HashMap;

/// A single provider block inside `qrmi_config.json`.
///
/// ```json
/// {
///     "type": "qiskit-runtime-service",
///     "environment": {
///         "QRMI_IBM_QRS_ENDPOINT": "https://quantum.cloud.ibm.com/api/v1",
///         "QRMI_IBM_QRS_IAM_ENDPOINT": "https://iam.cloud.ibm.com",
///         "QRMI_IBM_QRS_IAM_APIKEY": "my_apikey",
///         "QRMI_IBM_QRS_SERVICE_CRN": "my_instance"
///     }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderDef {
    /// Vendor/service type string (e.g. `"qiskit-runtime-service"`, `"iqm-server"`).
    /// Matches the same strings used by [`crate::models::ResourceType`].
    pub r#type: String,

    /// Environment variables required by this provider and the resources it returns.
    pub environment: HashMap<String, String>,
}

/// Top-level structure of `qrmi_config.json` when `"version": 2`.
///
/// ```json
/// {
///     "version": 2,
///     "providers": [
///         { "type": "qiskit-runtime-service", "environment": { ... } }
///     ]
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderConfig {
    /// Config file format version. Must be `2` for this struct.
    pub version: u32,

    /// List of provider definitions. Each `ResourceProvider` implementation
    /// selects the entry whose `type` field matches its own vendor string.
    pub providers: Vec<ProviderDef>,
}

impl ProviderConfig {
    /// Loads and parses a `ProviderConfig` from a JSON file at `path`.
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read provider config {:?}: {}", path, e))?;
        let cfg: ProviderConfig = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse provider config {:?}: {}", path, e))?;
        if cfg.version != 2 {
            anyhow::bail!(
                "Unsupported provider config version {}. Expected 2.",
                cfg.version
            );
        }
        Ok(cfg)
    }

    /// Returns the first [`ProviderDef`] whose `type` field equals `provider_type`,
    /// or `None` if no match is found.
    pub fn find(&self, provider_type: &str) -> Option<&ProviderDef> {
        self.providers.iter().find(|p| p.r#type == provider_type)
    }
}
