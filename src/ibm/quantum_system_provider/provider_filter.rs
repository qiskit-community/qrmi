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

//! Filter parsing and application for [`super::IBMQuantumSystemProvider::resources`].
//!
//! Filter string format: `key=value` pairs joined by `&`.
//!
//! Supported keys:
//! - `num_qubits=<N>`      — include only backends with `n_qubits >= N`
//! - `max_shots=<N>`       — include only backends with `max_shots >= N`
//! - `name=<glob>`         — include only backends whose name matches the glob pattern
//! - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
//! - `status=online`       — include only backends with `BackendStatus::Online`
//!   (applied at list_backends stage, before config fetch)
//!
//! Example: `num_qubits=127&name=ibm_*&status=online`

use crate::ibm::models::BackendConfiguration;
use anyhow::{anyhow, Result};
use glob::Pattern;
use quantum_system_api::models::{Backend, BackendStatus};

/// Whether to filter by online status.
#[derive(Debug, Default, PartialEq)]
pub(super) enum StatusFilter {
    /// No status filtering.
    #[default]
    Any,
    /// Only backends with `BackendStatus::Online`.
    Online,
}

/// Parsed, validated filter criteria.
#[derive(Debug)]
pub(super) struct BackendFilter {
    /// Minimum qubit count (inclusive). `None` means no constraint.
    pub num_qubits: Option<u64>,
    /// Minimum max_shots (inclusive). `None` means no constraint.
    pub max_shots: Option<u64>,
    /// Glob pattern for backend name. `None` means no constraint.
    pub name_pattern: Option<Pattern>,
    /// Whether to include simulators. Defaults to `false` (simulators excluded).
    pub is_simulator: bool,
    /// Whether to filter by online status.
    pub status: StatusFilter,
}

impl Default for BackendFilter {
    fn default() -> Self {
        Self {
            num_qubits: None,
            max_shots: None,
            name_pattern: None,
            is_simulator: false,
            status: StatusFilter::Any,
        }
    }
}

impl BackendFilter {
    /// Parses a filter string of the form `key=value&key=value`.
    ///
    /// Unknown keys are silently ignored to allow forward compatibility.
    /// An empty string produces a filter with default values applied.
    pub fn parse(filters: &str) -> Result<Self> {
        let mut f = BackendFilter::default();

        for pair in filters.split('&') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let (key, value) = pair.split_once('=').ok_or_else(|| {
                anyhow!("Invalid filter segment {:?}: expected 'key=value'", pair)
            })?;
            match key.trim() {
                "num_qubits" => {
                    f.num_qubits = Some(value.trim().parse::<u64>().map_err(|_| {
                        anyhow!(
                            "Invalid value for 'num_qubits': {:?} (expected a non-negative integer)",
                            value
                        )
                    })?);
                }
                "max_shots" => {
                    f.max_shots = Some(value.trim().parse::<u64>().map_err(|_| {
                        anyhow!(
                            "Invalid value for 'max_shots': {:?} (expected a non-negative integer)",
                            value
                        )
                    })?);
                }
                "name" => {
                    f.name_pattern = Some(Pattern::new(value.trim()).map_err(|e| {
                        anyhow!("Invalid glob pattern for 'name' filter {:?}: {}", value, e)
                    })?);
                }
                "is_simulator" => {
                    f.is_simulator = match value.trim() {
                        "true" => true,
                        "false" => false,
                        _ => {
                            return Err(anyhow!(
                            "Invalid value for 'is_simulator': {:?} (expected 'true' or 'false')",
                            value
                        ))
                        }
                    };
                }
                "status" => {
                    f.status = match value.trim() {
                        "online" => StatusFilter::Online,
                        _ => {
                            return Err(anyhow!(
                                "Invalid value for 'status': {:?} (supported: 'online')",
                                value
                            ))
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(f)
    }

    /// Returns `true` if `backend` satisfies the `status` constraint.
    /// Applied at `list_backends` stage before config fetch.
    pub fn matches_status(&self, backend: &Backend) -> bool {
        match self.status {
            StatusFilter::Any => true,
            StatusFilter::Online => backend.status == BackendStatus::Online,
        }
    }

    /// Returns `true` if `name` satisfies the `name` glob constraint.
    /// Applied at `list_backends` stage before config fetch.
    pub fn matches_name(&self, name: &str) -> bool {
        match &self.name_pattern {
            Some(pattern) => pattern.matches(name),
            None => true,
        }
    }

    /// Returns `true` if `config` satisfies `num_qubits`, `max_shots`,
    /// and `is_simulator` constraints.
    /// `name` and `status` are applied at the `list_backends` stage.
    pub fn matches_config(&self, config: &BackendConfiguration) -> bool {
        // is_simulator filter (default: false — exclude simulators).
        if config.simulator != self.is_simulator {
            return false;
        }

        // num_qubits filter.
        if let Some(min) = self.num_qubits {
            if config.n_qubits < min {
                return false;
            }
        }

        // max_shots filter.
        if let Some(min) = self.max_shots {
            if config.max_shots < min {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ibm::models::BackendConfiguration;
    use quantum_system_api::models::{Backend, BackendStatus};

    fn make_backend(name: &str, status: BackendStatus) -> Backend {
        Backend {
            name: name.to_string(),
            status,
            message: None,
            version: None,
        }
    }

    fn make_config(
        name: &str,
        n_qubits: u64,
        max_shots: u64,
        simulator: bool,
    ) -> BackendConfiguration {
        BackendConfiguration {
            backend_name: name.to_string(),
            backend_version: "1.0.0".to_string(),
            n_qubits,
            basis_gates: vec![],
            gates: vec![],
            local: false,
            simulator,
            conditional: false,
            memory: false,
            max_shots,
            coupling_map: None,
            max_experiments: None,
            n_registers: 1,
            register_map: None,
            configurable: false,
            credits_required: false,
            online_date: None,
            display_name: None,
            description: None,
            tags: None,
            rep_delay_range: None,
            default_rep_delay: None,
            dynamic_reprate_enabled: false,
            measure_esp_enabled: false,
            supported_instructions: vec![],
            supported_features: vec![],
            quantum_volume: None,
            processor_type: None,
            qubit_lo_range: None,
            meas_lo_range: None,
            timing_constraints: None,
        }
    }

    #[test]
    fn default_filter_excludes_simulators() {
        let f = BackendFilter::parse("").unwrap();
        assert!(f.matches_config(&make_config("ibm_torino", 133, 4096, false)));
        assert!(!f.matches_config(&make_config("simulator", 32, 4096, true)));
    }

    #[test]
    fn status_filter() {
        let f = BackendFilter::parse("status=online").unwrap();
        assert!(f.matches_status(&make_backend("ibm_torino", BackendStatus::Online)));
        assert!(!f.matches_status(&make_backend("ibm_offline", BackendStatus::Offline)));
        assert!(!f.matches_status(&make_backend("ibm_paused", BackendStatus::Paused)));
    }

    #[test]
    fn num_qubits_filter() {
        let f = BackendFilter::parse("num_qubits=127").unwrap();
        assert!(f.matches_config(&make_config("ibm_torino", 133, 4096, false)));
        assert!(f.matches_config(&make_config("ibm_127q", 127, 4096, false)));
        assert!(!f.matches_config(&make_config("ibm_small", 5, 4096, false)));
    }

    #[test]
    fn max_shots_filter() {
        let f = BackendFilter::parse("max_shots=8192").unwrap();
        assert!(f.matches_config(&make_config("ibm_torino", 133, 10000, false)));
        assert!(f.matches_config(&make_config("ibm_exact", 127, 8192, false)));
        assert!(!f.matches_config(&make_config("ibm_small", 127, 4096, false)));
    }

    #[test]
    fn combined_filter() {
        let f = BackendFilter::parse("num_qubits=127&name=ibm_*&max_shots=8192").unwrap();
        // passes both name and config filters
        assert!(f.matches_name("ibm_torino"));
        assert!(f.matches_config(&make_config("ibm_torino", 133, 10000, false)));
        // fails num_qubits
        assert!(!f.matches_config(&make_config("ibm_small", 5, 10000, false)));
        // fails name filter (checked by matches_name, not matches_config)
        assert!(!f.matches_name("other_127q"));
        // fails max_shots
        assert!(!f.matches_config(&make_config("ibm_lowshots", 127, 1024, false)));
    }

    #[test]
    fn invalid_filters_return_error() {
        assert!(BackendFilter::parse("num_qubits=abc").is_err());
        assert!(BackendFilter::parse("max_shots=abc").is_err());
        assert!(BackendFilter::parse("is_simulator=maybe").is_err());
        assert!(BackendFilter::parse("status=offline").is_err());
        assert!(BackendFilter::parse("name=[invalid").is_err());
    }
}
