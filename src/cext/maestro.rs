// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal, Alice and Bob,  2025, 2026
// (C) Copyright UKRI-STFC (Hartree Centre) 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
// Maestro payload conversion extracted from cext.rs.

//! Maestro's C payload adapter, kept separate from upstream payload dispatch.

use super::Payload;
use std::ffi::CStr;

/// Converts Maestro payloads; other variants and invalid UTF-8 return `None`.
///
/// The caller must provide valid, nul-terminated strings, as required by the
/// public C API. Submission and error classification remain in `cext.rs`.
pub(super) unsafe fn payload_from_c(payload: &Payload) -> Option<crate::models::Payload> {
    let Payload::MaestroLocal {
        input,
        job_type,
        qubits,
        simulator_type,
        simulation_method,
        observables,
        config,
    } = *payload
    else {
        return None;
    };

    Some(crate::models::Payload::MaestroLocal {
        input: CStr::from_ptr(input).to_str().ok()?.to_owned(),
        job_type: CStr::from_ptr(job_type).to_str().ok()?.to_owned(),
        qubits,
        simulator_type,
        simulation_method,
        observables: CStr::from_ptr(observables).to_str().ok()?.to_owned(),
        config: CStr::from_ptr(config).to_str().ok()?.to_owned(),
    })
}
