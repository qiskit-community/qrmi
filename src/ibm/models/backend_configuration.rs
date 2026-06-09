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

//! Shared IBM backend configuration model used by both
//! `IBMQiskitRuntimeServiceProvider` and `IBMQuantumSystemProvider`.
//!
//! The JSON format is identical between the two services.

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

fn default_n_registers() -> u64 {
    1
}
fn default_false() -> bool {
    false
}
fn empty_string_array() -> Vec<String> {
    Vec::new()
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    let v = Value::deserialize(deserializer)?;
    match v {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom("expected string or number")),
    }
}

/// Gate configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct GateConfig {
    /// The gate name as it will be referred to in QASM
    pub name: String,

    /// Variable names for the gate parameters (if any)
    pub parameters: Option<Vec<String>>,

    /// List of qubit groupings which are coupled by this gate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_map: Option<Vec<Vec<u64>>>,

    /// Definition of this gate in terms of QASM primitives U and CX
    pub qasm_def: Option<String>,

    /// This specified gate supports conditional operations (true/false).
    #[serde(default = "default_false")]
    pub conditional: bool,

    /// Register latency map
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_map: Option<Vec<Vec<u64>>>,

    /// Description of the gate operation
    pub description: Option<String>,
}

/// Processor type
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ProcessorType {
    /// Processor family indicates quantum chip architecture
    pub family: String,

    /// Revision number. Note: sometimes returned as integer from server.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub revision: String,

    /// Segment, if indicated
    pub segment: Option<String>,
}

/// Timing constraints
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct TimingConstraints {
    pub granularity: u64,
    pub min_length: u64,
    pub pulse_alignment: u64,
    pub acquire_alignment: u64,
}

/// IBM backend configuration shared between Qiskit Runtime Service and Quantum System.
///
/// The JSON format is identical between the two services.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfiguration {
    /// Backend name
    pub backend_name: String,

    /// Backend version in the form X.X.X
    pub backend_version: String,

    /// Number of qubits
    pub n_qubits: u64,

    /// List of basis gate names on the backend
    pub basis_gates: Vec<String>,

    pub gates: Vec<GateConfig>,

    /// Backend is local or remote
    pub local: bool,

    /// Backend is a simulator
    pub simulator: bool,

    /// Backend supports conditional operations
    #[serde(default = "default_false")]
    pub conditional: bool,

    /// Backend supports memory
    #[serde(default = "default_false")]
    pub memory: bool,

    /// Maximum number of shots supported
    pub max_shots: u64,

    /// Array grouping qubits that are physically coupled together
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_map: Option<Vec<Vec<u64>>>,

    /// Maximum number of experiments supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_experiments: Option<u64>,

    /// Number of register slots available for feedback
    #[serde(default = "default_n_registers")]
    pub n_registers: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_map: Option<Vec<Vec<u64>>>,

    #[serde(default = "default_false")]
    pub configurable: bool,

    #[serde(default = "default_false")]
    pub credits_required: bool,

    pub online_date: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,

    /// Range of delay times between programs (microseconds)
    pub rep_delay_range: Option<Vec<f64>>,

    pub default_rep_delay: Option<f64>,

    #[serde(default = "default_false")]
    pub dynamic_reprate_enabled: bool,

    #[serde(default = "default_false")]
    pub measure_esp_enabled: bool,

    #[serde(default = "empty_string_array")]
    pub supported_instructions: Vec<String>,

    #[serde(default = "empty_string_array")]
    pub supported_features: Vec<String>,

    pub quantum_volume: Option<u64>,
    pub processor_type: Option<ProcessorType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub qubit_lo_range: Option<Vec<Vec<f64>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meas_lo_range: Option<Vec<Vec<f64>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing_constraints: Option<TimingConstraints>,
}
