// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! QRMI implementations for Pasqal Cloud Services and Pasqal Local

mod cloud;
mod cloud_config;
pub mod error;
mod local;

pub use self::cloud::PasqalCloud;
pub use self::local::PasqalLocal;

// Pasqal common utils
use crate::models::QuantumResourceInfo;
use crate::Result;

/// Fills `info` from the Pulser device specs returned by `target()`,
/// i.e. `[{"device_type": ..., "specs": "<Pulser device JSON>"}]`.
fn parse_device_specs(info: &mut QuantumResourceInfo, target: &str) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct Entry {
        specs: String,
    }
    let [entry]: [Entry; 1] = serde_json::from_str(target)?;
    let specs: serde_json::Value = serde_json::from_str(&entry.specs)?;
    info.backend_display_name = specs["name"].as_str().map(str::to_string);
    info.num_qubits = specs["max_atom_num"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok());
    info.max_shots = specs["max_runs"].as_u64();
    info.has_queue = Some(true);
    Ok(())
}
