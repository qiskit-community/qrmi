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

//! [`ResourceProvider`] implementation for IBM Qiskit Runtime Service.

mod provider_filter;

use crate::ibm::IBMQiskitRuntimeService;
use crate::models::{ProviderConfig, ProviderDef};
use crate::resource_provider::ResourceProvider;
use crate::QuantumResource;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::future::join_all;
use log::warn;
use provider_filter::BackendFilter;
use qiskit_runtime_client::apis::{auth, backends_api, configuration};
use qiskit_runtime_client::models::BackendsResponseV2DevicesInner;
use std::collections::HashMap;
use std::env;

/// Provider type string used in `qrmi_config.json`.
const PROVIDER_TYPE: &str = "qiskit-runtime-service";

/// Env var that holds the path to the provider config file.
const CONFIG_FILE_ENV: &str = "QRMI_RESOURCE_PROVIDER_CONFIG_FILE";

/// A [`ResourceProvider`] that discovers backends available through IBM Qiskit Runtime Service.
///
/// On construction, reads `QRMI_RESOURCE_PROVIDER_CONFIG_FILE` to locate
/// `qrmi_config.json`, then finds the `"qiskit-runtime-service"` provider block
/// and uses its `environment` map for all API calls.
///
/// # Config file example
///
/// ```json
/// {
///     "version": 2,
///     "providers": [
///         {
///             "type": "qiskit-runtime-service",
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
    /// Pre-built reqwest configuration pointing at the QRS endpoint.
    config: configuration::Configuration,
    /// IAM API key used for token refresh.
    api_key: String,
    /// IAM endpoint URL used for token refresh.
    iam_endpoint: String,
    /// Raw environment values from the provider block, kept so we can inject
    /// `{backend_name}_*` variables before constructing each `IBMQiskitRuntimeService`.
    provider_env: HashMap<String, String>,
}

impl IBMQiskitRuntimeServiceProvider {
    /// Constructs a new provider.
    ///
    /// Reads `QRMI_RESOURCE_PROVIDER_CONFIG_FILE`, parses the JSON, and finds the
    /// `"qiskit-runtime-service"` provider block. All required connection parameters
    /// are taken from that block's `environment` map.
    pub fn new() -> Result<Self> {
        let config_path = env::var(CONFIG_FILE_ENV)
            .map_err(|_| anyhow!("{CONFIG_FILE_ENV} environment variable is not set"))?;

        let cfg = ProviderConfig::load(&config_path)?;

        let provider_def: &ProviderDef = cfg.find(PROVIDER_TYPE).ok_or_else(|| {
            anyhow!(
                "No '{}' provider block found in {:?}",
                PROVIDER_TYPE,
                config_path
            )
        })?;

        Self::from_provider_def(provider_def)
    }

    fn from_provider_def(def: &ProviderDef) -> Result<Self> {
        let get = |key: &str| -> Result<String> {
            def.environment.get(key).cloned().ok_or_else(|| {
                anyhow!(
                    "Missing '{}' in '{}' provider environment block",
                    key,
                    PROVIDER_TYPE
                )
            })
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
            provider_env: def.environment.clone(),
        })
    }

    /// Injects `{backend_name}_KEY=VALUE` environment variables so that
    /// `IBMQiskitRuntimeService::new(backend_name)` can find the connection parameters.
    fn inject_backend_env(&self, backend_name: &str) {
        for (key, value) in &self.provider_env {
            env::set_var(format!("{backend_name}_{key}"), value);
        }
    }

    /// Checks whether a backend is online by calling `get_backend_status()`.
    /// Returns `true` if the status is "active" or "online".
    async fn is_backend_online(
        config: &configuration::Configuration,
        name: &str,
    ) -> bool {
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
    /// - `num_qubits=<N>`      — only backends with `qubits >= N`
    /// - `name=<glob>`         — only backends whose name matches the glob pattern
    /// - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
    /// - `status=online`       — only backends that are online (calls `get_backend_status()`
    ///                           per backend in parallel)
    ///
    /// Example: `"num_qubits=127&name=ibm_*&status=online"`
    ///
    /// # Sorting
    ///
    /// Results are sorted by `queue_length` ascending (least busy first).
    async fn backends(&self, filters: String) -> Result<Vec<Box<dyn QuantumResource + Send + Sync>>> {
        // Parse filter string upfront so we fail fast on bad input.
        let filter = BackendFilter::parse(&filters)?;

        // Clone config so we can mutate the bearer token locally for this call.
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

        // Apply synchronous filters (is_simulator, num_qubits, name).
        let mut devices: Vec<BackendsResponseV2DevicesInner> = response
            .devices
            .unwrap_or_default()
            .into_iter()
            .filter(|d| filter.matches(d))
            .collect();

        // Apply async status filter if requested — check all backends in parallel.
        if filter.needs_status_check() {
            let checks: Vec<_> = devices
                .iter()
                .map(|d| Self::is_backend_online(&config, &d.name))
                .collect();

            let results = join_all(checks).await;

            devices = devices
                .into_iter()
                .zip(results)
                .filter_map(|(d, online)| if online { Some(d) } else { None })
                .collect();
        }

        // Sort by queue_length ascending (least busy first).
        devices.sort_by_key(|d| d.queue_length);

        // Construct a QuantumResource for each backend that passed the filter.
        let resources: Vec<Box<dyn QuantumResource + Send + Sync>> = devices
            .into_iter()
            .filter_map(|device| {
                self.inject_backend_env(&device.name);

                match IBMQiskitRuntimeService::new(&device.name) {
                    Ok(r) => Some(Box::new(r) as Box<dyn QuantumResource + Send + Sync>),
                    Err(e) => {
                        warn!(
                            "Skipping backend {:?}: failed to construct \
                             IBMQiskitRuntimeService: {:?}",
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
