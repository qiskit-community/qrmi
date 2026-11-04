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
use std::os::raw::c_char;

/// A nul-terminated UTF-8 string, or `default` for a NULL pointer.
unsafe fn string_or(pointer: *const c_char, default: Option<&str>) -> Option<String> {
    if pointer.is_null() {
        return default.map(str::to_owned);
    }
    CStr::from_ptr(pointer).to_str().ok().map(str::to_owned)
}

/// Converts Maestro payloads. Other variants, invalid UTF-8 and a NULL `input`
/// or `job_type` return `None`, which `cext.rs` refuses as invalid input before
/// contacting the server.
///
/// `observables` and `config` are unused by native requests (`job_type`
/// `"request"`), so NULL selects their defaults, `""` and `"{}"`, as the Lua
/// binding does. Any non-NULL string must be valid and nul-terminated.
/// Submission and error classification remain in `cext.rs`.
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
        input: string_or(input, None)?,
        job_type: string_or(job_type, None)?,
        qubits,
        simulator_type,
        simulation_method,
        observables: string_or(observables, Some(""))?,
        config: string_or(config, Some("{}"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{
        qrmi_get_last_error, qrmi_resource_task_start, qrmi_string_free, QuantumResource,
        ReturnCode,
    };
    use super::*;
    use crate::maestro::MaestroLocal;
    use std::sync::Arc;

    fn payload(input: *const c_char, job_type: *const c_char) -> Payload {
        Payload::MaestroLocal {
            input: input.cast_mut(),
            job_type: job_type.cast_mut(),
            qubits: 0,
            simulator_type: 0,
            simulation_method: 0,
            observables: std::ptr::null_mut(),
            config: std::ptr::null_mut(),
        }
    }

    #[test]
    fn null_optional_strings_take_their_defaults() {
        let request = c"{\"schema_version\":2}";
        let converted = unsafe { payload_from_c(&payload(request.as_ptr(), c"request".as_ptr())) };
        let Some(crate::models::Payload::MaestroLocal {
            input,
            job_type,
            observables,
            config,
            ..
        }) = converted
        else {
            panic!("A request payload with NULL unused fields was refused");
        };
        assert_eq!(input, "{\"schema_version\":2}");
        assert_eq!(job_type, "request");
        assert_eq!(observables, "");
        assert_eq!(config, "{}");
    }

    #[test]
    fn null_required_strings_are_refused_without_dereferencing() {
        let request = c"{\"schema_version\":2}".as_ptr();
        let null = std::ptr::null();
        unsafe {
            assert!(payload_from_c(&payload(null, c"request".as_ptr())).is_none());
            assert!(payload_from_c(&payload(request, null)).is_none());
        }
        // A zero-initialized payload is refused as invalid input; nothing is sent.
        let mut resource = QuantumResource {
            inner: Box::new(MaestroLocal::from_config("c_maestro", Default::default()).unwrap()),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
        };
        let zeroed = payload(null, null);
        let mut task_id = std::ptr::null_mut();
        unsafe {
            assert!(matches!(
                qrmi_resource_task_start(&mut resource, &zeroed, &mut task_id),
                ReturnCode::InvalidInputError
            ));
            qrmi_string_free(qrmi_get_last_error());
        }
        assert!(task_id.is_null());
    }
}
