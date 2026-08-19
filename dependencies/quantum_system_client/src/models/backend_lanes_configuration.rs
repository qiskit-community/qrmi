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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)]
/// backend lanes configuration
pub struct LaneConfiguration {
    /// Number of the execution lanes
    pub lanes: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
/// A list of [`Backend`] available for direct access.
pub struct BackendLanesConfiguration {
    /// Lane configuration for HPC workload manager
    pub hpc_workload_manager: LaneConfiguration,

    /// Lane configuration for IBM Quantum Compute Service
    pub ibm_quantum_compute: LaneConfiguration,
}
