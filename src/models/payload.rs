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

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Task Payload
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub enum Payload {
    /// Payload that contains Qiskit Primitive input.
    QiskitPrimitive { input: String, program_id: String },
    /// Payload for Pasqal Cloud
    PasqalCloud { sequence: String, job_runs: i32 },
}
