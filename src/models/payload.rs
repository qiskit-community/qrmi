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

use pyo3::prelude::*;

/// Task Payload
#[derive(Debug, Clone, PartialEq)]
#[pyclass]
pub enum Payload {
    /// Payload that contains Qiskit Primitive Unified Bloc(PUB)
    QiskitPrimitive { pubs: String, program_id: String },
    /// Payload for Pasqal Cloud
    PasqalCloud { value: String },
}
