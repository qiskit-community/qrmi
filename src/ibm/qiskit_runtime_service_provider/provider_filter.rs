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

//! Filter parsing and application for [`super::IBMQiskitRuntimeServiceProvider::resources`].
//!
//! Filter string format: `key=value` pairs joined by `&`.
//!
//! Supported keys:
//! - `num_qubits=<N>`      — include only backends with `n_qubits >= N`
//! - `max_shots=<N>`       — include only backends with `max_shots >= N`
//! - `name=<glob>`         — include only backends whose name matches the glob pattern
//! - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
//! - `status=online`       — include only online backends (requires extra API call per backend)
//!
//! Example: `num_qubits=127&name=ibm_*&status=online`

use crate::ibm::models::BackendConfiguration;
use anyhow::{anyhow, Result};
use glob::Pattern;
use qiskit_runtime_client::models::BackendsResponseV2DevicesInner;

/// Whether to filter by online status.
#[derive(Debug, Default, PartialEq)]
pub(super) enum StatusFilter {
    /// No status filtering.
    #[default]
    Any,
    /// Only backends that are online (status == "active" or "online").
    Online,
}

/// Parsed, validated filter criteria.
#[derive(Debug)]
pub(super) struct BackendFilter {
    /// Minimum qubit count (inclusive). `None` means no constraint.
    pub num_qubits: Option<u32>,
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
                    let n: u32 = value.trim().parse().map_err(|_| {
                        anyhow!(
                            "Invalid value for 'num_qubits': {:?} (expected a non-negative integer)",
                            value
                        )
                    })?;
                    f.num_qubits = Some(n);
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
                    let pattern = Pattern::new(value.trim()).map_err(|e| {
                        anyhow!("Invalid glob pattern for 'name' filter {:?}: {}", value, e)
                    })?;
                    f.name_pattern = Some(pattern);
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

    /// Returns `true` if `device` satisfies the `name` glob constraint.
    /// Applied at `list_backends` stage before config fetch.
    pub fn matches_name(&self, name: &str) -> bool {
        match &self.name_pattern {
            Some(pattern) => pattern.matches(name),
            None => true,
        }
    }

    /// Returns `true` if `device` satisfies `is_simulator` constraint from list_backends.
    /// Applied at `list_backends` stage before config fetch.
    pub fn matches_device(&self, device: &BackendsResponseV2DevicesInner) -> bool {
        let simulator = device.is_simulator.unwrap_or(false);
        simulator == self.is_simulator
    }

    /// Returns `true` if `config` satisfies `num_qubits`, `max_shots` and `is_simulator`
    /// constraints from BackendConfiguration.
    /// `name` is applied at the `list_backends` stage.
    pub fn matches_config(&self, config: &BackendConfiguration) -> bool {
        // is_simulator filter.
        if config.simulator != self.is_simulator {
            return false;
        }

        // num_qubits filter.
        if let Some(min) = self.num_qubits {
            if config.n_qubits < min as u64 {
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

    /// Returns `true` if this filter requires per-backend status API calls.
    pub fn needs_status_check(&self) -> bool {
        self.status == StatusFilter::Online
    }

    /// Returns `true` if the given status string counts as "online".
    pub fn is_online_status(status: &str) -> bool {
        matches!(status.to_lowercase().as_str(), "active" | "online")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qiskit_runtime_client::models::{
        BackendsResponseV2DevicesInner, BackendsResponseV2DevicesInnerStatus,
    };

    fn make_device(name: &str, is_simulator: Option<bool>) -> BackendsResponseV2DevicesInner {
        BackendsResponseV2DevicesInner {
            name: name.to_string(),
            status: Box::new(BackendsResponseV2DevicesInnerStatus::default()),
            is_simulator,
            qubits: None,
            clops: None,
            processor_type: None,
            queue_length: 0,
            performance_metrics: None,
        }
    }

    #[test]
    fn default_filter_excludes_simulators() {
        let f = BackendFilter::parse("").unwrap();
        assert!(f.matches_device(&make_device("ibm_torino", Some(false))));
        assert!(!f.matches_device(&make_device("ibmq_qasm_simulator", Some(true))));
        assert!(f.matches_device(&make_device("ibm_unknown", None)));
    }

    #[test]
    fn name_glob_filter() {
        let f = BackendFilter::parse("name=ibm_*").unwrap();
        assert!(f.matches_name("ibm_torino"));
        assert!(f.matches_name("ibm_brisbane"));
        assert!(!f.matches_name("other_backend"));
    }

    #[test]
    fn status_filter_parsed() {
        let f = BackendFilter::parse("status=online").unwrap();
        assert!(f.needs_status_check());
    }

    #[test]
    fn is_online_status() {
        assert!(BackendFilter::is_online_status("active"));
        assert!(BackendFilter::is_online_status("online"));
        assert!(BackendFilter::is_online_status("ACTIVE"));
        assert!(!BackendFilter::is_online_status("offline"));
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
