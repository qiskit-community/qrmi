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

    // retry_status_codes / retry_methods arrive as JSON arrays (lists);
    // OqtopusConfig expects frozenset[int] / frozenset[str].
    let builtins = py.import("builtins")?;
    for key in ["retry_status_codes", "retry_methods"] {
        if let Some(val) = kwargs.get_item(key)? {
            let frozenset = builtins.call_method1("frozenset", (val,))?;
            kwargs.set_item(key, frozenset)?;
        }
    }

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

fn extract_user_api_error(py: Python, err: &PyErr) -> Option<(i64, String)> {
    let value = err.value(py);
    let status_code: i64 = value.getattr("status_code").ok()?.extract().ok()?;
    let message: String = value.getattr("message").ok()?.extract().ok()?;
    Some((status_code, message))
}

pub fn job_spec_cls<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    py.import("oqtopus_client")?
        .getattr("services")?
        .getattr("job_spec")?
        .getattr("OqtopusJobSpec")
}

/// Recursively normalizes a Python value for JSON serialization.
/// Prefers `model_dump(mode="json")` if available (used by e.g.
/// `JobsJobInfo`), falling back to `to_dict()` if that's what the value
/// exposes instead (used by e.g. `JobsS3TranspileResult`). Values with
/// neither are returned unchanged (plain values, dicts, and
/// `(str, Enum)` members are already JSON-friendly, and anything else
/// falls back to `json.dumps(..., default=str)` at the call site).
fn normalize_value<'py>(py: Python<'py>, value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if value.hasattr("model_dump")? {
        let kwargs = PyDict::new(py);
        kwargs.set_item("mode", "json")?;
        return value.call_method("model_dump", (), Some(&kwargs));
    }
    if value.hasattr("to_dict")? {
        return value.call_method0("to_dict");
    }
    Ok(value.clone())
}

/// Shared plumbing for a py_bridge C entry point that takes one
/// argument string plus `config_json`, and returns one output string.
/// Handles catching panics, initializing Python, running `f` under the
/// GIL, formatting `UserApiError`s and tracebacks, and writing the
/// result (or error) through `out`/`out_error`.
///
/// Every py_bridge function below shares this exact shape (decode two
/// input strings, run some Python calls, produce one output string, map
/// errors the same way, write through the same kind of out-params), so
/// this centralizes that instead of repeating it six times.
///
/// # Safety
/// `arg` and `config_json` must be valid pointers to nul-terminated
/// UTF-8 strings. `out` and `out_error` must each be either `NULL` or a
/// valid, writable pointer to a `*mut c_char`.
unsafe fn run_bridge_call(
    arg: *const c_char,
    config_json: *const c_char,
    out: *mut *mut c_char,
    out_error: *mut *mut c_char,
    f: impl FnOnce(Python, &str, &str) -> PyResult<String> + std::panic::UnwindSafe,
) -> c_int {
    let result = std::panic::catch_unwind(|| -> PyResult<String> {
        let arg = unsafe { CStr::from_ptr(arg) }
            .to_string_lossy()
            .into_owned();
        let config_json = unsafe { CStr::from_ptr(config_json) }
            .to_string_lossy()
            .into_owned();

        Python::initialize();

        Python::attach(|py| {
            f(py, &arg, &config_json).map_err(|e| {
                if let Some((status_code, message)) = extract_user_api_error(py, &e) {
                    return pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "api error: status={status_code} message={message}"
                    ));
                }
                let tb = e
                    .traceback(py)
                    .and_then(|tb| tb.format().ok())
                    .unwrap_or_default();
                pyo3::exceptions::PyRuntimeError::new_err(format!("{e}\n{tb}"))
            })
        })
    });

    match result {
        Ok(Ok(value)) => {
            let c_string = CString::new(value)
                .unwrap_or_else(|_| CString::new("(contains invalid data)").unwrap());
            unsafe { *out = c_string.into_raw() };
            0
        }
        Ok(Err(e)) => {
            unsafe { set_error(out_error, &format!("python error: {e}")) };
            -1
        }
        Err(_) => {
            unsafe { set_error(out_error, "panicked while calling python") };
            -1
        }
    }
}

/// Frees a string previously returned through an `out_*` parameter of
/// one of this crate's C entry points (allocated via
/// `CString::into_raw`). Safe to call with `NULL` (no-op).
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Safety
/// `ptr`, if non-null, must be a pointer previously returned from one of
/// this crate's `out_*` parameters, and must not already have been
/// freed (double-freeing is undefined behavior, as with any C
/// allocator).
#[no_mangle]
pub unsafe extern "C" fn py_bridge_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
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

/// Calls `oqtopus_client`'s `OqtopusClient.get_device(device_id)` and
/// returns the device's `status` string (e.g. `"available"`,
/// `"unavailable"`).
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Parameters
/// - `device_id`: the device ID to query (a nul-terminated UTF-8 string).
/// - `config_json`: a JSON object string passed as `**kwargs` to the
///   `OqtopusConfig` constructor (e.g. `{"url": "...", "api_token": "..."}`).
/// - `out_status`: on success, a pointer to the resulting status string
///   (nul-terminated, allocated via `CString::into_raw`) is written here.
///   The caller must free it with `py_bridge_free_string` once done.
/// - `out_error`: on failure (non-zero return), a pointer to an error
///   message (nul-terminated, allocated via `CString::into_raw`) is
///   written here. Passing `NULL` just logs to stderr instead and no
///   pointer is written. As with `out_status`, a non-null result must be
///   freed with `py_bridge_free_string`.
///
/// # Returns
/// `0` on success. `-1` if a Python exception was raised, or if this
/// function panicked internally.
///
/// # Safety
/// - `device_id` and `config_json` must both be valid pointers to
///   nul-terminated UTF-8 strings (invalid UTF-8 is lossily replaced by
///   `to_string_lossy` rather than causing a crash, but the result is not
///   guaranteed to be meaningful).
/// - `out_status` and `out_error` must each be either `NULL` or a valid,
///   writable pointer to a `*mut c_char`.
/// - Any string returned through `out_status` or `out_error` leaks until
///   the caller frees it. Callers must always call the matching
///   `py_bridge_free_string`.
/// - The caller must ensure the corresponding libpython has already been
///   `dlopen`'d (by `py_loader`) before this function is called.
#[no_mangle]
pub unsafe extern "C" fn get_device_status(
    device_id: *const c_char,
    config_json: *const c_char,
    out_status: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    unsafe {
        run_bridge_call(
            device_id,
            config_json,
            out_status,
            out_error,
            |py, device_id, config_json| {
                let client = build_client(py, config_json)?;
                let device = client.call_method1("get_device", (device_id,))?;
                device.getattr("status")?.extract()
            },
        )
    }
}

/// Calls `oqtopus_client`'s `OqtopusClient.cancel_job(job_id)` to cancel
/// a job.
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Parameters
/// - `job_id`: the ID of the job to cancel (a nul-terminated UTF-8 string).
/// - `config_json`: a JSON object string passed as `**kwargs` to the
///   `OqtopusConfig` constructor (e.g. `{"url": "...", "api_token": "..."}`).
/// - `out_message`: on success, a pointer to the confirmation message
///   returned by the API (nul-terminated, allocated via
///   `CString::into_raw`) is written here. The caller must free it with
///   `py_bridge_free_string` once done.
/// - `out_error`: on failure (non-zero return), a pointer to an error
///   message (nul-terminated, allocated via `CString::into_raw`) is
///   written here. For `UserApiError`-like exceptions (e.g. attempting
///   to cancel a job that has already completed), the message includes
///   the `status_code`/`message` detail. Passing `NULL` just logs to
///   stderr instead and no pointer is written. As with `out_message`, a
///   non-null result must be freed with `py_bridge_free_string`.
///
/// # Returns
/// `0` on success. `-1` if a Python exception was raised, or if this
/// function panicked internally.
///
/// # Safety
/// - `job_id` and `config_json` must both be valid pointers to
///   nul-terminated UTF-8 strings (invalid UTF-8 is lossily replaced by
///   `to_string_lossy` rather than causing a crash, but the result is not
///   guaranteed to be meaningful).
/// - `out_message` and `out_error` must each be either `NULL` or a valid,
///   writable pointer to a `*mut c_char`.
/// - Any string returned through `out_message` or `out_error` leaks until
///   the caller frees it. Callers must always call the matching
///   `py_bridge_free_string`.
/// - The caller must ensure the corresponding libpython has already been
///   `dlopen`'d (by `py_loader`) before this function is called.
#[no_mangle]
pub unsafe extern "C" fn cancel_job(
    job_id: *const c_char,
    config_json: *const c_char,
    out_message: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    unsafe {
        run_bridge_call(
            job_id,
            config_json,
            out_message,
            out_error,
            |py, job_id, config_json| {
                let client = build_client(py, config_json)?;
                let response = client.call_method1("cancel_job", (job_id,))?;
                response.getattr("message")?.extract()
            },
        )
    }
}

/// Calls `oqtopus_client`'s `OqtopusClient.get_job_status(job_id)` and
/// returns the job's `status` value (e.g. `"running"`, `"succeeded"`,
/// `"failed"`, ...). `JobsJobStatus` is a `str`-subclassed `Enum` on the
/// Python side, so it extracts directly as a plain string.
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Parameters
/// - `job_id`: the ID of the job to query (a nul-terminated UTF-8 string).
/// - `config_json`: a JSON object string passed as `**kwargs` to the
///   `OqtopusConfig` constructor (e.g. `{"url": "...", "api_token": "..."}`).
/// - `out_status`: on success, a pointer to the resulting status string
///   (nul-terminated, allocated via `CString::into_raw`) is written here.
///   The caller must free it with `py_bridge_free_string` once done.
/// - `out_error`: on failure (non-zero return), a pointer to an error
///   message (nul-terminated, allocated via `CString::into_raw`) is
///   written here. For `UserApiError`-like exceptions, the message
///   includes the `status_code`/`message` detail. Passing `NULL` just
///   logs to stderr instead and no pointer is written. As with
///   `out_status`, a non-null result must be freed with
///   `py_bridge_free_string`.
///
/// # Returns
/// `0` on success. `-1` if a Python exception was raised, or if this
/// function panicked internally.
///
/// # Safety
/// - `job_id` and `config_json` must both be valid pointers to
///   nul-terminated UTF-8 strings (invalid UTF-8 is lossily replaced by
///   `to_string_lossy` rather than causing a crash, but the result is not
///   guaranteed to be meaningful).
/// - `out_status` and `out_error` must each be either `NULL` or a valid,
///   writable pointer to a `*mut c_char`.
/// - Any string returned through `out_status` or `out_error` leaks until
///   the caller frees it. Callers must always call the matching
///   `py_bridge_free_string`.
/// - The caller must ensure the corresponding libpython has already been
///   `dlopen`'d (by `py_loader`) before this function is called.
#[no_mangle]
pub unsafe extern "C" fn get_job_status(
    job_id: *const c_char,
    config_json: *const c_char,
    out_status: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    unsafe {
        run_bridge_call(
            job_id,
            config_json,
            out_status,
            out_error,
            |py, job_id, config_json| {
                let client = build_client(py, config_json)?;
                let response = client.call_method1("get_job_status", (job_id,))?;
                // JobsJobStatus is (str, Enum), so this extracts directly.
                response.getattr("status")?.extract()
            },
        )
    }
}

/// Calls `oqtopus_client`'s `OqtopusClient.get_device(device_id)`,
/// serializes the resulting `OqtopusDevice` dataclass to a JSON string
/// (via `vars()` + `json.dumps(..., default=str)`, so any field type
/// without a direct JSON mapping falls back to its `str()`
/// representation instead of raising), and returns that string.
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Parameters
/// - `device_id`: the device ID to query (a nul-terminated UTF-8 string).
/// - `config_json`: a JSON object string passed as `**kwargs` to the
///   `OqtopusConfig` constructor (e.g. `{"url": "...", "api_token": "..."}`).
/// - `out_json`: on success, a pointer to the resulting JSON string
///   (nul-terminated, allocated via `CString::into_raw`) is written here.
///   The caller must free it with `py_bridge_free_string` once done.
/// - `out_error`: on failure (non-zero return), a pointer to an error
///   message (nul-terminated, allocated via `CString::into_raw`) is
///   written here. For `UserApiError`-like exceptions, the message
///   includes the `status_code`/`message` detail. Passing `NULL` just
///   logs to stderr instead and no pointer is written. As with
///   `out_json`, a non-null result must be freed with
///   `py_bridge_free_string`.
///
/// # Returns
/// `0` on success. `-1` if a Python exception was raised, or if this
/// function panicked internally.
///
/// # Safety
/// - `device_id` and `config_json` must both be valid pointers to
///   nul-terminated UTF-8 strings (invalid UTF-8 is lossily replaced by
///   `to_string_lossy` rather than causing a crash, but the result is not
///   guaranteed to be meaningful).
/// - `out_json` and `out_error` must each be either `NULL` or a valid,
///   writable pointer to a `*mut c_char`.
/// - Any string returned through `out_json` or `out_error` leaks until
///   the caller frees it. Callers must always call the matching
///   `py_bridge_free_string`.
/// - The caller must ensure the corresponding libpython has already been
///   `dlopen`'d (by `py_loader`) before this function is called.
#[no_mangle]
pub unsafe extern "C" fn get_device_json(
    device_id: *const c_char,
    config_json: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    unsafe {
        run_bridge_call(
            device_id,
            config_json,
            out_json,
            out_error,
            |py, device_id, config_json| {
                let client = build_client(py, config_json)?;
                let device = client.call_method1("get_device", (device_id,))?;

                // OqtopusDevice's declared dataclass fields only contain
                // `raw`; the actual data lives in the instance's
                // __dict__, so use vars() instead of dataclasses.asdict().
                let builtins = py.import("builtins")?;
                let device_dict = builtins.call_method1("vars", (device,))?;
                let device_dict = device_dict.cast::<PyDict>()?;

                let json_mod = py.import("json")?;

                // device_info is itself a JSON string; parse it so it
                // nests as a proper object instead of being embedded as
                // an escaped string. If parsing fails for any reason,
                // leave it as-is.
                if let Some(device_info_str) = device_dict.get_item("device_info")? {
                    if let Ok(device_info_str) = device_info_str.extract::<String>() {
                        if let Ok(parsed) = json_mod.call_method1("loads", (device_info_str,)) {
                            device_dict.set_item("device_info", parsed)?;
                        }
                    }
                }

                let str_fn = builtins.getattr("str")?;
                let kwargs = PyDict::new(py);
                kwargs.set_item("default", str_fn)?;

                json_mod
                    .call_method("dumps", (device_dict,), Some(&kwargs))?
                    .extract()
            },
        )
    }
}

/// Calls `oqtopus_client`'s `OqtopusClient.get_job_result(job_id)` and
/// serializes the resulting `OqtopusJobResult` (a subclass such as
/// `OqtopusSamplingJobResult`) to a JSON string.
///
/// The object stores its data under underscore-prefixed attributes
/// (e.g. `_job_id`, `_status`, ..., plus a non-serializable `_client`
/// reference). This strips the leading underscore from each attribute
/// name, drops `client` entirely, and recursively normalizes any nested
/// pydantic-model-like values (e.g. `job_info`, `transpile_result`) via
/// `model_dump(mode="json")` before encoding. Any remaining
/// non-JSON-native values (e.g. `datetime`) fall back to their `str()`
/// representation.
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Parameters
/// - `job_id`: the ID of the job to query (a nul-terminated UTF-8 string).
/// - `config_json`: a JSON object string passed as `**kwargs` to the
///   `OqtopusConfig` constructor (e.g. `{"url": "...", "api_token": "..."}`).
/// - `out_json`: on success, a pointer to the resulting JSON string
///   (nul-terminated, allocated via `CString::into_raw`) is written here.
///   The caller must free it with `py_bridge_free_string` once done.
/// - `out_error`: on failure (non-zero return), a pointer to an error
///   message (nul-terminated, allocated via `CString::into_raw`) is
///   written here. For `UserApiError`-like exceptions, the message
///   includes the `status_code`/`message` detail. Passing `NULL` just
///   logs to stderr instead and no pointer is written. As with
///   `out_json`, a non-null result must be freed with
///   `py_bridge_free_string`.
///
/// # Returns
/// `0` on success. `-1` if a Python exception was raised, or if this
/// function panicked internally.
///
/// # Safety
/// - `job_id` and `config_json` must both be valid pointers to
///   nul-terminated UTF-8 strings (invalid UTF-8 is lossily replaced by
///   `to_string_lossy` rather than causing a crash, but the result is not
///   guaranteed to be meaningful).
/// - `out_json` and `out_error` must each be either `NULL` or a valid,
///   writable pointer to a `*mut c_char`.
/// - Any string returned through `out_json` or `out_error` leaks until
///   the caller frees it. Callers must always call the matching
///   `py_bridge_free_string`.
/// - The caller must ensure the corresponding libpython has already been
///   `dlopen`'d (by `py_loader`) before this function is called.
#[no_mangle]
pub unsafe extern "C" fn get_job_result_json(
    job_id: *const c_char,
    config_json: *const c_char,
    out_json: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    unsafe {
        run_bridge_call(
            job_id,
            config_json,
            out_json,
            out_error,
            |py, job_id, config_json| {
                let client = build_client(py, config_json)?;
                let job_result = client.call_method1("get_job_result", (job_id,))?;

                let builtins = py.import("builtins")?;
                let raw_vars = builtins.call_method1("vars", (job_result,))?;
                let raw_vars = raw_vars.cast::<PyDict>()?;

                let result_dict = PyDict::new(py);
                for (key, value) in raw_vars.iter() {
                    let key_str: String = key.extract()?;
                    let name = key_str.trim_start_matches('_').to_string();
                    if name == "client" {
                        continue;
                    }
                    let normalized = normalize_value(py, &value)?;
                    result_dict.set_item(name, normalized)?;
                }

                let str_fn = builtins.getattr("str")?;
                let kwargs = PyDict::new(py);
                kwargs.set_item("default", str_fn)?;

                py.import("json")?
                    .call_method("dumps", (result_dict,), Some(&kwargs))?
                    .extract()
            },
        )
    }
}

/// Calls `oqtopus_client`'s `OqtopusClient.submit_job(OqtopusJobSpec)` and
/// returns the resulting job ID.
///
/// `job_spec_json` is a single JSON object describing the job:
/// `{"job_type": "sampling", "device_id": "...", "program": "...",
/// "shots": 1000, "name": null, "description": null,
/// "transpiler_info": null, "simulator_info": null,
/// "mitigation_info": null}`. `job_type` selects which
/// `OqtopusJobSpec.<job_type>(...)` classmethod builds the spec
/// (`sampling`, `estimation`, `multi_manual`, or `sse`); the remaining
/// keys are passed through as `**kwargs`.
///
/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// # Parameters
/// - `job_spec_json`: the job spec as described above (a nul-terminated
///   UTF-8 JSON string).
/// - `config_json`: a JSON object string passed as `**kwargs` to the
///   `OqtopusConfig` constructor (e.g. `{"url": "...", "api_token": "..."}`).
/// - `out_job_id`: on success, a pointer to the submitted job's ID
///   (nul-terminated, allocated via `CString::into_raw`) is written here.
///   The caller must free it with `py_bridge_free_string` once done.
/// - `out_error`: on failure (non-zero return), a pointer to an error
///   message (nul-terminated, allocated via `CString::into_raw`) is
///   written here. For `UserApiError`-like exceptions, the message
///   includes the `status_code`/`message` detail. Passing `NULL` just
///   logs to stderr instead and no pointer is written. As with
///   `out_job_id`, a non-null result must be freed with
///   `py_bridge_free_string`.
///
/// # Returns
/// `0` on success. `-1` if a Python exception was raised, or if this
/// function panicked internally.
///
/// # Safety
/// - `job_spec_json` and `config_json` must both be valid pointers to
///   nul-terminated UTF-8 strings (invalid UTF-8 is lossily replaced by
///   `to_string_lossy` rather than causing a crash, but the result is not
///   guaranteed to be meaningful).
/// - `out_job_id` and `out_error` must each be either `NULL` or a valid,
///   writable pointer to a `*mut c_char`.
/// - Any string returned through `out_job_id` or `out_error` leaks until
///   the caller frees it. Callers must always call the matching
///   `py_bridge_free_string`.
/// - The caller must ensure the corresponding libpython has already been
///   `dlopen`'d (by `py_loader`) before this function is called.
#[no_mangle]
pub unsafe extern "C" fn submit_job(
    job_spec_json: *const c_char,
    config_json: *const c_char,
    out_job_id: *mut *mut c_char,
    out_error: *mut *mut c_char,
) -> c_int {
    unsafe {
        run_bridge_call(
            job_spec_json,
            config_json,
            out_job_id,
            out_error,
            |py, job_spec_json, config_json| {
                let client = build_client(py, config_json)?;

                let json_mod = py.import("json")?;
                let spec_dict = json_mod.call_method1("loads", (job_spec_json,))?;
                let spec_dict = spec_dict.cast::<PyDict>()?;

                let job_type: String = spec_dict
                    .get_item("job_type")?
                    .ok_or_else(|| {
                        pyo3::exceptions::PyValueError::new_err("job_spec_json missing job_type")
                    })?
                    .extract()?;
                spec_dict.del_item("job_type")?;

                let builder = job_spec_cls(py)?.getattr(job_type.as_str())?;
                let job_spec = builder.call((), Some(spec_dict))?;

                let response = client.call_method1("submit_job", (job_spec,))?;

                // Prefer the direct attribute; fall back to to_dict() in
                // case the generated model exposes it under a different
                // name/alias.
                if let Ok(job_id) = response.getattr("job_id") {
                    if let Ok(job_id) = job_id.extract::<String>() {
                        return Ok(job_id);
                    }
                }
                let as_dict = response.call_method0("to_dict")?;
                let as_dict = as_dict.cast::<PyDict>()?;
                for key in ["job_id", "id", "jobId"] {
                    if let Some(value) = as_dict.get_item(key)? {
                        if let Ok(job_id) = value.extract::<String>() {
                            return Ok(job_id);
                        }
                    }
                }
                Err(pyo3::exceptions::PyValueError::new_err(
                    "could not find job_id in submit_job response",
                ))
            },
        )
    }
}
