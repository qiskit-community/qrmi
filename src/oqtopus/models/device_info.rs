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

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OqtopusDeviceStatus {
    Available,
    Unavailable,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
pub(crate) enum OqtopusDeviceType {
    #[serde(rename = "QPU")]
    Qpu,
    #[serde(rename = "simulator")]
    Simulator,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct OqtopusDeviceInfo {
    pub(crate) device_id: String,
    pub(crate) device_type: OqtopusDeviceType,
    pub(crate) status: OqtopusDeviceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) available_at: Option<String>,
    pub(crate) n_pending_jobs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) n_qubits: Option<u32>,
    pub(crate) basis_gates: Vec<String>,
    pub(crate) supported_instructions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) device_info: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) calibrated_at: Option<String>,
    pub(crate) description: String,
}
