//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

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

/// Gate configuration
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct GateConfig {
    /// The gate name as it will be referred to in QASM
    pub name: String,

    /// variable names for the gate parameters (if any)
    pub parameters: Vec<String>,

    /// List of qubit groupings which are coupled by this gate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_map: Option<Vec<Vec<u64>>>,

    /// Definition of this gate in terms of QASM primitives U and CX
    pub qasm_def: String,

    /// This specified gate supports conditional operations (true/false). If this is not specified, then the gate inherits the conditional property of the backend.
    pub conditional: bool,

    /// An array of dimension len(coupling_map) X n_registers that specifies (1 - fast, 0 - slow) the register latency conditional operations on the gate
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

    /// Revision number reflects design variants within a given processor family. Is typically a semantic versioning value without the patch value, eg., \"1.0\".
    pub revision: String,

    /// Segment, if indicated, is used to distinguish different subsets of the qubit fabric/chip
    pub segment: Option<String>,
}

/// Timing constraints
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct TimingConstraints {
    /// Waveform memory data chunk size
    pub granularity: u64,

    /// Minimum number of samples required to define a pulse
    pub min_length: u64,

    /// Instruction triggering time resolution of pulse channel in units of dt
    pub pulse_alignment: u64,

    /// Instruction triggering time resolution of acquisition channel in units of dt
    pub acquire_alignment: u64,
}

/// Qiskit device backend configuration as Rust struct
///
/// # Example
///
/// ```no_run
/// use anyhow::Result;
/// use direct_access_api::{AuthMethod, ClientBuilder};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = ClientBuilder::new("http://localhost:8080")
///         .with_auth(AuthMethod::IbmCloudIam {
///             apikey: "your_iam_apikey".to_string(),
///             service_crn: "your_service_crn".to_string(),
///             iam_endpoint_url: "iam_endpoint_url".to_string(),
///         })
///         .build()
///         .unwrap();
///     let config = client
///         .get_backend_configuration::<direct_access_api::models::BackendConfiguration>("ibm_brisbane")
///         .await?;
///     println!("{:#?}", config);
///     Ok(())
/// }
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfiguration {
    /// Backend name
    pub backend_name: String,

    /// Backend version in the form X.X.X
    pub backend_version: String,

    /// Number of qubits
    pub n_qubits: u64,

    /// List of basis gates names on the backend
    pub basis_gates: Vec<String>,

    pub gates: Vec<GateConfig>,

    /// Backend is local or remote (true/false)
    pub local: bool,

    /// Backend is a simulator (true/false)
    pub simulator: bool,

    /// Backend supports conditional operations (true/false)
    pub conditional: bool,

    /// Backend supports memory (true/false)
    pub memory: bool,

    /// Maximum number of shots supported
    pub max_shots: u64,

    /// Array grouping qubits that are physically coupled together on the backend
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coupling_map: Option<Vec<Vec<u64>>>,

    /// Maximum number of experiments supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_experiments: Option<u64>,

    /// Number of register slots available for feedback (if conditional is true)
    #[serde(default = "default_n_registers")]
    pub n_registers: u64,

    /// An array of dimension n_qubits X n_registers that specifies whether a qubit can store a measurement in a certain register slot
    pub register_map: Option<Vec<Vec<u64>>>,

    /// Backend is configurable, if the backend is a simulator (true/false)
    #[serde(default = "default_false")]
    pub configurable: bool,

    /// Backend requires credits to run a job (true/false)
    #[serde(default = "default_false")]
    pub credits_required: bool,

    /// Date the backend went online
    pub online_date: Option<String>,

    /// Alternate name field for the backend
    pub display_name: Option<String>,

    /// Description of the backend
    pub description: Option<String>,

    /// Tags
    pub tags: Option<Vec<String>>,

    /// Range of delay times between programs (microseconds) allowed by backend.
    pub rep_delay_range: Option<Vec<Vec<f64>>>,

    /// Default rep delay.
    pub default_rep_delay: Option<f64>,

    /// Whether delay between programs can be set dynamically using 'rep_delay').
    #[serde(default = "default_false")]
    pub dynamic_reprate_enabled: bool,

    /// Whether ESP readout is supported by the backend.
    #[serde(default = "default_false")]
    pub measure_esp_enabled: bool,

    /// Instructions supported by the backend.
    #[serde(default = "empty_string_array")]
    pub supported_instructions: Vec<String>,

    /// Array of features supported by the backend such as qobj, qasm3, etc.
    #[serde(default = "empty_string_array")]
    pub supported_features: Vec<String>,

    /// Backend quantum volume
    pub quantum_volume: Option<u64>,

    /// Processor type
    pub processor_type: Option<ProcessorType>,

    /// Frequency range for the qubit LO
    pub qubit_lo_range: Option<Vec<Vec<f64>>>,

    /// Frequency range for the measurement LO
    pub meas_lo_range: Option<Vec<Vec<f64>>>,

    pub timing_constraints: Option<TimingConstraints>,
}
