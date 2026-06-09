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

//! [`ResourceProvider`] implementation for IBM Quantum System.

mod provider_filter;

use crate::ibm::models::BackendConfiguration;
use crate::ibm::IBMQuantumSystem;
use crate::ResourceProvider;
use crate::QuantumResource;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::future::join_all;
use log::warn;
use provider_filter::BackendFilter;
use quantum_system_api::{AuthMethod, ClientBuilder, models::Backends};
use std::collections::HashMap;
use std::env;

/// A [`ResourceProvider`] that discovers backends available through IBM Quantum System.
///
/// Constructed from a [`ResourceDef`] with `is_dynamic: true`.
///
/// # Config file example
///
/// ```json
/// {
///     "resources": [
///         {
///             "name": "ibm_qs_prod",
///             "type": "ibm-quantum-system",
///             "is_dynamic": true,
///             "environment": {
///                 "QRMI_IBM_QS_ENDPOINT":     "https://your-quantum-system-endpoint",
///                 "QRMI_IBM_QS_IAM_ENDPOINT": "https://iam.cloud.ibm.com",
///                 "QRMI_IBM_QS_IAM_APIKEY":   "my_apikey",
///                 "QRMI_IBM_QS_SERVICE_CRN":  "my_instance"
///             }
///         }
///     ]
/// }
/// ```
pub struct IBMQuantumSystemProvider {
    client: quantum_system_api::Client,
    provider_env: HashMap<String, String>,
}

impl IBMQuantumSystemProvider {
    /// Constructs a new provider from an environment variable map.
    ///
    /// # Required keys
    ///
    /// - `QRMI_IBM_QS_ENDPOINT`
    /// - `QRMI_IBM_QS_IAM_ENDPOINT`
    /// - `QRMI_IBM_QS_IAM_APIKEY`
    /// - `QRMI_IBM_QS_SERVICE_CRN`
    pub fn new(environment: &HashMap<String, String>) -> Result<Self> {
        let get = |key: &str| -> Result<String> {
            environment.get(key).cloned().ok_or_else(|| {
                anyhow!("Missing '{}' in environment map", key)
            })
        };

        let endpoint = get("QRMI_IBM_QS_ENDPOINT")?;
        let apikey = get("QRMI_IBM_QS_IAM_APIKEY")?;
        let service_crn = get("QRMI_IBM_QS_SERVICE_CRN")?;
        let iam_endpoint_url = get("QRMI_IBM_QS_IAM_ENDPOINT")?;

        let client = ClientBuilder::new(endpoint)
            .with_auth(AuthMethod::IbmCloudIam {
                apikey,
                service_crn,
                iam_endpoint_url,
            })
            .build()
            .map_err(|e| anyhow!("Failed to build quantum system client: {:?}", e))?;

        Ok(Self {
            client,
            provider_env: environment.clone(),
        })
    }

    /// Injects `{backend_name}_KEY=VALUE` environment variables so that
    /// `IBMQuantumSystem::new(backend_name)` can find the connection parameters.
    fn inject_backend_env(&self, backend_name: &str) {
        for (key, value) in &self.provider_env {
            env::set_var(format!("{backend_name}_{key}"), value);
        }
    }
}

#[async_trait]
impl ResourceProvider for IBMQuantumSystemProvider {
    /// Returns backends available through IBM Quantum System, optionally filtered.
    ///
    /// Results are returned in the order provided by `list_backends()` (no queue-length
    /// sorting, as this system has no queue).
    ///
    /// # Filter string format
    ///
    /// `key=value` pairs joined by `&`. Supported keys:
    ///
    /// - `num_qubits=<N>`      — only backends with `n_qubits >= N`
    /// - `max_shots=<N>`       — only backends with `max_shots >= N`
    /// - `name=<glob>`         — only backends whose name matches the glob pattern
    /// - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
    /// - `status=online`       — only online backends (filtered at list stage)
    ///
    /// Example: `Some("num_qubits=127&name=ibm_*&status=online")`
    async fn resources(
        &self,
        filters: Option<String>,
    ) -> Result<Vec<Box<dyn QuantumResource + Send + Sync>>> {
        let filter = BackendFilter::parse(filters.as_deref().unwrap_or(""))?;

        let backends = self
            .client
            .list_backends::<Backends>()
            .await
            .map_err(|e| anyhow!("Failed to list backends: {:?}", e))?;

        let candidates: Vec<String> = backends
            .backends
            .into_iter()
            .filter(|b| filter.matches_status(b))
            .filter(|b| filter.matches_name(&b.name))
            .map(|b| b.name)
            .collect();

        let config_futures: Vec<_> = candidates
            .iter()
            .map(|name| self.client.get_backend_configuration::<serde_json::Value>(name))
            .collect();

        let configs = join_all(config_futures).await;

        let resources: Vec<Box<dyn QuantumResource + Send + Sync>> = candidates
            .into_iter()
            .zip(configs)
            .filter_map(|(name, config_result)| {
                let raw = match config_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Skipping backend {:?}: get_backend_configuration failed: {:?}", name, e);
                        return None;
                    }
                };

                let config: BackendConfiguration = match serde_json::from_value(raw.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!(
                            "Skipping backend {:?}: failed to deserialize BackendConfiguration: {:?}\nRaw JSON: {}",
                            name, e,
                            serde_json::to_string_pretty(&raw).unwrap_or_else(|_| raw.to_string())
                        );
                        return None;
                    }
                };

                if !filter.matches_config(&config) {
                    return None;
                }

                self.inject_backend_env(&name);

                match IBMQuantumSystem::new(&name) {
                    Ok(r) => Some(Box::new(r) as Box<dyn QuantumResource + Send + Sync>),
                    Err(e) => {
                        warn!(
                            "Skipping backend {:?}: failed to construct IBMQuantumSystem: {:?}",
                            name, e
                        );
                        None
                    }
                }
            })
            .collect();

        Ok(resources)
    }
}
