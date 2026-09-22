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

//! This crate is built as a cdylib with PyO3's `extension-module` feature,
//! which tells PyO3's build script to *not* link against libpython at
//! build time. That means this .so/.dylib can be built and shipped even on
//! machines that don't have libpython available.
//!
//! It is not meant to be loaded directly by the OS loader as a normal
//! dependency (that would fail: it has unresolved `Py_*` symbols). Instead
//! it is meant to be `dlopen`'d at runtime (see the `py_loader` crate),
//! *after* libpython has already been loaded into the process with
//! `RTLD_GLOBAL`, so the dynamic linker can resolve those symbols lazily
//! against the already-loaded libpython.

//! This crate is built as a cdylib with PyO3's `extension-module` feature,
//! which tells PyO3's build script to *not* link against libpython at
//! build time. That means this .so/.dylib can be built and shipped even on
//! machines that don't have libpython available.
//!
//! It is not meant to be loaded directly by the OS loader as a normal
//! dependency (that would fail: it has unresolved `Py_*` symbols). Instead
//! it is meant to be `dlopen`'d at runtime (see the `py_loader` crate),
//! *after* libpython has already been loaded into the process with
//! `RTLD_GLOBAL`, so the dynamic linker can resolve those symbols lazily
//! against the already-loaded libpython.
//!
//! No special setup is needed here to make the interpreter find its
//! stdlib and site-packages: `PYTHONHOME` is a standard env var that
//! CPython itself reads directly during `Py_Initialize`, as long as it's
//! present in the process environment (inherited from the shell that
//! launched `app`, or set programmatically before `app`/`py_loader` runs
//! this code) -- see `py_loader`'s use of `PYTHONHOME` to find the
//! `.so` itself, which is the same value CPython needs here.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::os::raw::c_int;

unsafe fn set_error(out_error: *mut *mut c_char, msg: &str) {
    if out_error.is_null() {
        eprintln!("[py_bridge] {msg}");
        return;
    }
    let c_string =
        CString::new(msg).unwrap_or_else(|_| CString::new("error (invalid utf8)").unwrap());
    unsafe { *out_error = c_string.into_raw() };
}

pub fn build_client<'py>(py: Python<'py>, config_json: &str) -> PyResult<Bound<'py, PyAny>> {
    let oqtopus = py.import("oqtopus_client")?;

    let json_mod = py.import("json")?;
    let config_dict = json_mod.call_method1("loads", (config_json,))?;
    let kwargs = config_dict.cast::<PyDict>()?;

    let config_cls = oqtopus
        .getattr("services")?
        .getattr("config")?
        .getattr("OqtopusConfig")?;
    let config = config_cls.call((), Some(kwargs))?;

    let client_cls = oqtopus
        .getattr("services")?
        .getattr("client")?
        .getattr("OqtopusClient")?;
    client_cls.call1((config,))
}

pub fn job_spec_cls<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    py.import("oqtopus_client")?
        .getattr("services")?
        .getattr("job_spec")?
        .getattr("OqtopusJobSpec")
}

/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// Returns 0 on success, -1 on failure (Python exception or panic).
#[no_mangle]
pub extern "C" fn test() -> c_int {
    let result = std::panic::catch_unwind(|| -> PyResult<()> {
        Python::initialize();

        Python::attach(|py| {
            py.run(
                c"import sys; print(f'[py_bridge] hello from Python {sys.version}')\n\
                 print('[py_bridge] sys.path =', sys.path)",
                None,
                None,
            )
        })
    });

    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            eprintln!("[py_bridge] python error: {e}");
            -1
        }
        Err(_) => {
            eprintln!("[py_bridge] panicked while calling python");
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_device_status(
    device_id: *const c_char,
    config_json: *const c_char,
    out_status: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    let result = std::panic::catch_unwind(|| -> PyResult<String> {
        let device_id = unsafe { CStr::from_ptr(device_id) }
            .to_string_lossy()
            .into_owned();
        let config_json = unsafe { CStr::from_ptr(config_json) }
            .to_string_lossy()
            .into_owned();

        Python::initialize();

        Python::attach(|py| {
            let inner_result: PyResult<String> = (|| {
                let client = build_client(py, &config_json)?;
                let device = client.call_method1("get_device", (device_id.clone(),))?;
                let status: String = device.getattr("status")?.extract()?;
                Ok(status)
            })();

            inner_result.map_err(|e| {
                let tb = e
                    .traceback(py)
                    .and_then(|tb| tb.format().ok())
                    .unwrap_or_default();
                pyo3::exceptions::PyRuntimeError::new_err(format!("{e}\n{tb}"))
            })
        })
    });

    match result {
        Ok(Ok(status)) => {
            let c_string = CString::new(status)
                .unwrap_or_else(|_| CString::new("(status contains invalid data)").unwrap());
            unsafe { *out_status = c_string.into_raw() };
            0
        }
        Ok(Err(e)) => {
            set_error(out_error, &format!("python error: {e}"));
            -1
        }
        Err(_) => {
            set_error(out_error, "panicked while calling python");
            -1
        }
    }
}
