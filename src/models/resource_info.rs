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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QubitType {
    /// Superconducting Quantum Computers
    Superconducting,
    /// Trapped-Ion Quantum Computers
    TrappedIon,
    /// Photonic Quantum Computers,
    Photonic,
    /// Neural-Atom Quantum Computers
    NeuralAtom,
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
    pub backend_display_name: String, // Human-readable name reported by the provider API
    pub num_qubits: u32,
    pub qubit_type: QubitType,
    pub processor_name: Option<String>, // e.g. hardware generation name
    pub processor_revision: Option<String>,
    pub description: Option<String>,
    pub is_simulator: bool,
    pub has_queue: bool,
    pub max_shots: Option<u64>,
    pub pending_job_count: Option<u64>, // number of jobs currently queued on the backend
    pub status: Option<ResourceStatusCode>, // same as status().status; None if not reported

    // Extra: vendor-specific, structured or raw data
    pub extra: serde_json::Value,
}

impl QuantumResourceInfo {
    /// Creates a new instance with the required fields set; optional fields are left empty.
    pub fn new(resource_id: String, resource_type: &ResourceType, qubit_type: QubitType) -> Self {
        Self {
            resource_id,
            resource_type: resource_type.as_str().to_string(),
            backend_display_name: String::new(),
            num_qubits: 0,
            qubit_type,
            processor_name: None,
            processor_revision: None,
            description: None,
            is_simulator: false,
            has_queue: false,
            max_shots: None,
            pending_job_count: None,
            status: None,
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
            QubitType::NeuralAtom,
        );
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["resource_id"], "fresnel");
        assert_eq!(json["resource_type"], "pasqal-local");
        assert_eq!(json["qubit_type"], "neural_atom");
        assert!(json["max_shots"].is_null());
        assert!(json["status"].is_null());
        assert!(json["extra"].is_null());
    }
}
