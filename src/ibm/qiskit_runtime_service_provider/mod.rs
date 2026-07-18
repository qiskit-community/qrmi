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

//! [`ResourceProvider`] implementation for IBM Qiskit Runtime Service.

mod provider_filter;

use crate::ibm::models::BackendConfiguration;
use crate::ibm::IBMQiskitRuntimeService;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::future::join_all;
use log::warn;
use provider_filter::BackendFilter;
use qiskit_runtime_client::apis::{auth, backends_api, configuration};
use std::collections::HashMap;
use std::env;

use qrmi_core_api::{QuantumResource, ResourceProvider};

/// A [`ResourceProvider`] that discovers backends available through IBM Qiskit Runtime Service.
///
/// Constructed from a [`ResourceDef`] with `is_dynamic: true`.
///
/// # Config file example
///
/// ```json
/// {
///     "resources": [
///         {
///             "name": "ibm_inst1",
///             "type": "qiskit-runtime-service",
///             "is_dynamic": true,
///             "environment": {
///                 "QRMI_IBM_QRS_ENDPOINT":     "https://quantum.cloud.ibm.com/api/v1",
///                 "QRMI_IBM_QRS_IAM_ENDPOINT": "https://iam.cloud.ibm.com",
///                 "QRMI_IBM_QRS_IAM_APIKEY":   "my_apikey",
///                 "QRMI_IBM_QRS_SERVICE_CRN":  "my_instance"
///             }
///         }
///     ]
/// }
/// ```
pub struct IBMQiskitRuntimeServiceProvider {
    config: configuration::Configuration,
    api_key: String,
    iam_endpoint: String,
    provider_env: HashMap<String, String>,
}

impl IBMQiskitRuntimeServiceProvider {
    /// Constructs a new provider from an environment variable map.
    ///
    /// # Required keys
    ///
    /// - `QRMI_IBM_QRS_ENDPOINT`
    /// - `QRMI_IBM_QRS_IAM_ENDPOINT`
    /// - `QRMI_IBM_QRS_IAM_APIKEY`
    /// - `QRMI_IBM_QRS_SERVICE_CRN`
    pub fn new(environment: &HashMap<String, String>) -> Result<Self> {
        let get = |key: &str| -> Result<String> {
            environment
                .get(key)
                .cloned()
                .ok_or_else(|| anyhow!("Missing '{}' in environment map", key))
        };

        let qrs_endpoint = get("QRMI_IBM_QRS_ENDPOINT")?;
        let iam_endpoint = get("QRMI_IBM_QRS_IAM_ENDPOINT")?;
        let api_key = get("QRMI_IBM_QRS_IAM_APIKEY")?;
        let service_crn = get("QRMI_IBM_QRS_SERVICE_CRN")?;

        let mut config = configuration::Configuration::new();
        config.base_path = qrs_endpoint;
        config.bearer_access_token = None;
        config.crn = Some(service_crn);

        Ok(Self {
            config,
            api_key,
            iam_endpoint,
            provider_env: environment.clone(),
        })
    }

    fn inject_backend_env(&self, backend_name: &str) {
        for (key, value) in &self.provider_env {
            env::set_var(format!("{backend_name}_{key}"), value);
        }
    }

    async fn is_backend_online(config: &configuration::Configuration, name: &str) -> bool {
        match backends_api::get_backend_status(config, name, None).await {
            Ok(resp) => resp
                .status
                .as_deref()
                .map(BackendFilter::is_online_status)
                .unwrap_or(false),
            Err(e) => {
                warn!("Failed to get status for backend {:?}: {:?}", name, e);
                false
            }
        }
    }
}

#[async_trait]
impl ResourceProvider for IBMQiskitRuntimeServiceProvider {
    /// Returns backends available through IBM Qiskit Runtime Service,
    /// filtered and sorted by `queue_length` (ascending).
    ///
    /// # Filter string format
    ///
    /// `key=value` pairs joined by `&`. Supported keys:
    ///
    /// - `num_qubits=<N>`      — only backends with `n_qubits >= N`
    /// - `max_shots=<N>`       — only backends with `max_shots >= N`
    /// - `name=<glob>`         — only backends whose name matches the glob pattern
    /// - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
    /// - `status=online`       — only online backends
    ///
    /// Example: `Some("num_qubits=127&name=ibm_*&status=online")`
    ///
    /// # Sorting
    ///
    /// Results are sorted by `queue_length` ascending (least busy first).
    async fn resources(
        &self,
        filters: Option<String>,
    ) -> Result<Vec<Box<dyn QuantumResource + Send + Sync>>> {
        let filter = BackendFilter::parse(filters.as_deref().unwrap_or(""))?;

        let mut config = self.config.clone();
        let mut token_expiration: u64 = 0;
        let mut token_lifetime: u64 = 0;

        auth::check_token(
            &self.api_key,
            &self.iam_endpoint,
            &mut config.bearer_access_token,
            &mut token_expiration,
            &mut token_lifetime,
        )
        .await
        .map_err(|e| anyhow!("Token renewal failed: {:?}", e))?;

        let response = backends_api::list_backends(&config, Some("2025-01-01"))
            .await
            .map_err(|e| anyhow!("Failed to list backends: {:?}", e))?;

        // Apply name and is_simulator filters at list stage, keep queue_length for sorting.
        let candidates: Vec<_> = response
            .devices
            .unwrap_or_default()
            .into_iter()
            .filter(|d| filter.matches_device(d))
            .filter(|d| filter.matches_name(&d.name))
            .collect();

        // Apply async status filter if requested.
        let candidates = if filter.needs_status_check() {
            let checks: Vec<_> = candidates
                .iter()
                .map(|d| Self::is_backend_online(&config, &d.name))
                .collect();
            let results = join_all(checks).await;
            candidates
                .into_iter()
                .zip(results)
                .filter_map(|(d, online)| if online { Some(d) } else { None })
                .collect::<Vec<_>>()
        } else {
            candidates
        };

        // Fetch BackendConfiguration for each candidate in parallel.
        let config_futures: Vec<_> = candidates
            .iter()
            .map(|d| backends_api::get_backend_configuration(&config, &d.name, Some("2025-01-01")))
            .collect();

        let configs = join_all(config_futures).await;

        // Sort by queue_length ascending before constructing resources.
        let mut candidates_with_config: Vec<_> = candidates.into_iter().zip(configs).collect();

        candidates_with_config.sort_by_key(|(d, _)| d.queue_length);

        let resources: Vec<Box<dyn QuantumResource + Send + Sync>> = candidates_with_config
            .into_iter()
            .filter_map(|(device, config_result)| {
                let raw = match config_result {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Skipping backend {:?}: get_backend_configuration failed: {:?}", device.name, e);
                        return None;
                    }
                };

                let backend_config: BackendConfiguration = match serde_json::from_value(
                    serde_json::to_value(&raw).unwrap_or_default()
                ) {
                    Ok(c) => c,
                    Err(e) => {
                        warn!(
                            "Skipping backend {:?}: failed to deserialize BackendConfiguration: {:?}\nRaw: {:?}",
                            device.name, e, raw
                        );
                        return None;
                    }
                };

                if !filter.matches_config(&backend_config) {
                    return None;
                }

                self.inject_backend_env(&device.name);

                match IBMQiskitRuntimeService::new(&device.name) {
                    Ok(r) => Some(Box::new(r) as Box<dyn QuantumResource + Send + Sync>),
                    Err(e) => {
                        warn!(
                            "Skipping backend {:?}: failed to construct IBMQiskitRuntimeService: {:?}",
                            device.name, e
                        );
                        None
                    }
                }
            })
            .collect();

        Ok(resources)
    }
}
