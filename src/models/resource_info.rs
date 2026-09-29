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

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Default)]
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
    #[default]
    Other,
}

#[derive(Serialize, Default)]
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
    pub status: String, // quantum resource status (online, offline, maintenance etc. TBD)

    // Extra: vendor-specific, structured or raw data
    pub extra: serde_json::Value,
}
