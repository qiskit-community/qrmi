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

use crate::models::{ResourceStatusCode, ResourceType};
use serde::Serialize;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QubitType {
    /// Superconducting Quantum Computers
    Superconducting,
    /// Trapped-Ion Quantum Computers
    TrappedIon,
    /// Photonic Quantum Computers,
    Photonic,
    /// Neutral-Atom Quantum Computers
    NeutralAtom,
    /// Semiconductor-based Quantum Computers
    Semiconductor,
    /// Other types of Quantum Computers
    Other,
}

/// Configuration and attributes of a quantum resource, returned by
/// [`crate::QuantumResource::describe`].
#[derive(Debug, Clone, Serialize)]
pub struct QuantumResourceInfo {
    // Common
    pub resource_id: String,   // same as one can be obtained by resource_id()
    pub resource_type: String, // same as one can be obtained by resource_name()
    pub backend_display_name: Option<String>, // Human-readable name reported by the provider API; None if not reported
    pub num_qubits: Option<u32>,              // None if not reported
    pub qubit_type: QubitType,
    pub processor_name: Option<String>, // e.g. hardware generation name
    pub processor_revision: Option<String>,
    pub description: Option<String>,
    pub is_simulator: Option<bool>, // None if not reported
    pub has_queue: Option<bool>,    // None if not reported
    pub max_shots: Option<u64>,
    pub pending_job_count: Option<u64>, // number of jobs currently queued on the backend
    pub status: Option<ResourceStatusCode>, // same as status().status; None if not reported
    /// RFC 3339 timestamp of when this information was last generated or
    /// refreshed on the vendor's side (e.g. the calibration/maintenance
    /// date), if reported.
    pub last_updated: Option<String>,

    // Extra: vendor-specific, structured or raw data
    pub extra: serde_json::Value,
}

impl QuantumResourceInfo {
    /// Creates a new instance with the required fields set; optional fields are left empty.
    pub fn new(resource_id: String, resource_type: &ResourceType, qubit_type: QubitType) -> Self {
        Self {
            resource_id,
            resource_type: resource_type.as_str().to_string(),
            backend_display_name: None,
            num_qubits: None,
            qubit_type,
            processor_name: None,
            processor_revision: None,
            description: None,
            is_simulator: None,
            has_queue: None,
            max_shots: None,
            pending_job_count: None,
            status: None,
            last_updated: None,
            extra: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_serializes_with_defaults() {
        let info = QuantumResourceInfo::new(
            "fresnel".to_string(),
            &ResourceType::PasqalLocal,
            QubitType::NeutralAtom,
        );
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["resource_id"], "fresnel");
        assert_eq!(json["resource_type"], "pasqal-local");
        assert_eq!(json["qubit_type"], "neutral_atom");
        assert!(json["backend_display_name"].is_null());
        assert!(json["num_qubits"].is_null());
        assert!(json["is_simulator"].is_null());
        assert!(json["has_queue"].is_null());
        assert!(json["max_shots"].is_null());
        assert!(json["status"].is_null());
        assert!(json["last_updated"].is_null());
        assert!(json["extra"].is_null());
    }
}
