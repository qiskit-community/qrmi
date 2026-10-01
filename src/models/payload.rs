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
        /// Job spec, as a JSON object string matching the keyword
        /// arguments of Python's `OqtopusJobSpec` dataclass, minus
        /// `device_id` (QRMI fills that in automatically from the
        /// resource itself). Required keys: `job_type` (string: one of
        /// "sampling", "estimation", "multi_manual", "sse") and
        /// `program` (array of strings; a single program is a
        /// single-element array; `sse` jobs require exactly one).
        /// Optional keys: `shots` (integer; omit the key entirely to use
        /// OQTOPUS's own default of 1000 — sending `null` fails, since
        /// `OqtopusJobSpec.shots` is a plain `int` field, not
        /// `Optional[int]`), `name`, `description`, `transpiler_info`,
        /// `simulator_info`, `mitigation_info`, and `operator` (for
        /// estimation jobs).
        job_spec: String,
    },
}
#[cfg(feature = "pyo3")]
define_stub_info_gatherer!(stub_info);
