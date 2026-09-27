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

#[cfg(feature = "pyo3")]
use {
    pyo3::prelude::*,
    pyo3_stub_gen::{define_stub_info_gatherer, derive::*},
};

/// Task Payload
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object), gen_stub_pyclass_enum)]
pub enum Payload {
    /// Payload that contains Qiskit Primitive input.
    QiskitPrimitive { input: String, program_id: String },
    /// Payload for Pasqal Cloud
    PasqalCloud { sequence: String, job_runs: i32 },
    /// Payload for Alice and Bob's Cloud API AKA. Felis
    AliceBobFelis {
        human_qir: String,
        input_params: String,
    },
    /// Payload for IQM Server
    IQMServer {
        /// IQM JSON request body
        iqmjson: String,
        /// Job type(circuit, run, sweep)
        job_type: String,
        /// submit the job to the timeslot queue instead of the default FIFO queue
        use_timeslot: Option<bool>,
        /// Optional user-defined tag associated with the job
        tag: Option<String>,
    },
    /// Payload for OQTOPUS Cloud
    Oqtopus {
        /// Job type. "sampling" | "estimation" | "multi_manual" | "sse"
        job_type: String,
        /// QASM3 (or Python script for sse) program(s). A single-element
        /// `Vec` is a single program; `sse` jobs require exactly one.
        program: Vec<String>,
        /// Number of shots. Default is 1000.
        shots: Option<u32>,
        /// Job name.
        name: Option<String>,
        /// Job description.
        description: Option<String>,
        /// Transpiler settings. JSON object string, or None
        transpiler_info: Option<String>,
        /// Simulator settings. JSON object string, or None
        simulator_info: Option<String>,
        /// Error mitigation settings. JSON object string, or None
        mitigation_info: Option<String>,
    },
}
#[cfg(feature = "pyo3")]
define_stub_info_gatherer!(stub_info);
