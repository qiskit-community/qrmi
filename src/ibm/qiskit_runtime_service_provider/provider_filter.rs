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

//! Filter parsing and application for [`super::IBMQiskitRuntimeServiceProvider::backends`].
//!
//! Filter string format: `key=value` pairs joined by `&`.
//!
//! Supported keys:
//! - `num_qubits=<N>`      — include only backends with `qubits >= N`
//! - `name=<glob>`         — include only backends whose name matches the glob pattern
//! - `is_simulator=<bool>` — include/exclude simulators (default: `false`)
//! - `status=online`       — include only online backends (requires extra API call per backend)
//!
//! Example: `num_qubits=127&name=ibm_*`

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
            name_pattern: None,
            is_simulator: false, // exclude simulators by default
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
                    let n: u32 = value.trim().parse().map_err(|_| {
                        anyhow!(
                            "Invalid value for 'num_qubits': {:?} (expected a non-negative integer)",
                            value
                        )
                    })?;
                    f.num_qubits = Some(n);
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
                        _ => return Err(anyhow!(
                            "Invalid value for 'is_simulator': {:?} (expected 'true' or 'false')",
                            value
                        )),
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
                // Unknown keys are ignored for forward compatibility.
                _ => {}
            }
        }

        Ok(f)
    }

    /// Returns `true` if `device` satisfies the synchronous constraints
    /// (`num_qubits`, `name`, `is_simulator`).
    ///
    /// The `status` filter is async and handled separately in `mod.rs`.
    pub fn matches(&self, device: &BackendsResponseV2DevicesInner) -> bool {
        // is_simulator filter (default: false — exclude simulators).
        let simulator = device.is_simulator.unwrap_or(false);
        if simulator != self.is_simulator {
            return false;
        }

        // num_qubits filter.
        if let Some(min) = self.num_qubits {
            match device.qubits {
                Some(Some(n)) if n >= 0 && (n as u32) >= min => {}
                _ => return false,
            }
        }

        // name glob filter.
        if let Some(ref pattern) = self.name_pattern {
            if !pattern.matches(&device.name) {
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

    fn make_device(
        name: &str,
        qubits: Option<i32>,
        is_simulator: Option<bool>,
    ) -> BackendsResponseV2DevicesInner {
        BackendsResponseV2DevicesInner {
            name: name.to_string(),
            status: Box::new(BackendsResponseV2DevicesInnerStatus::default()),
            is_simulator,
            qubits: qubits.map(Some),
            clops: None,
            processor_type: None,
            queue_length: 0,
            performance_metrics: None,
        }
    }

    #[test]
    fn default_filter_excludes_simulators() {
        let f = BackendFilter::parse("").unwrap();
        assert!(f.matches(&make_device("ibm_torino", Some(133), Some(false))));
        assert!(!f.matches(&make_device("ibmq_qasm_simulator", None, Some(true))));
        // is_simulator field absent — treated as false, so passes
        assert!(f.matches(&make_device("ibm_unknown", Some(127), None)));
    }

    #[test]
    fn is_simulator_true_includes_simulators() {
        let f = BackendFilter::parse("is_simulator=true").unwrap();
        assert!(f.matches(&make_device("ibmq_qasm_simulator", None, Some(true))));
        assert!(!f.matches(&make_device("ibm_torino", Some(133), Some(false))));
    }

    #[test]
    fn num_qubits_filter() {
        let f = BackendFilter::parse("num_qubits=127").unwrap();
        assert!(f.matches(&make_device("ibm_torino", Some(133), Some(false))));
        assert!(f.matches(&make_device("ibm_127q", Some(127), Some(false))));
        assert!(!f.matches(&make_device("ibm_small", Some(5), Some(false))));
        assert!(!f.matches(&make_device("unknown_qubits", None, Some(false))));
    }

    #[test]
    fn name_glob_filter() {
        let f = BackendFilter::parse("name=ibm_*").unwrap();
        assert!(f.matches(&make_device("ibm_torino", Some(133), Some(false))));
        assert!(f.matches(&make_device("ibm_brisbane", Some(127), Some(false))));
        assert!(!f.matches(&make_device("other_backend", None, Some(false))));
    }

    #[test]
    fn combined_filter() {
        let f = BackendFilter::parse("num_qubits=127&name=ibm_*").unwrap();
        assert!(f.matches(&make_device("ibm_torino", Some(133), Some(false))));
        assert!(!f.matches(&make_device("ibm_small", Some(5), Some(false))));
        assert!(!f.matches(&make_device("other_127q", Some(127), Some(false))));
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
        assert!(!BackendFilter::is_online_status("degraded"));
    }

    #[test]
    fn invalid_is_simulator_returns_error() {
        assert!(BackendFilter::parse("is_simulator=maybe").is_err());
    }

    #[test]
    fn invalid_status_returns_error() {
        assert!(BackendFilter::parse("status=offline").is_err());
    }

    #[test]
    fn invalid_num_qubits_returns_error() {
        assert!(BackendFilter::parse("num_qubits=abc").is_err());
    }

    #[test]
    fn invalid_glob_returns_error() {
        assert!(BackendFilter::parse("name=[invalid").is_err());
    }
}
