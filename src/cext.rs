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
#![allow(dead_code)]
use crate::error::{QrmiError, QrmiErrorKind};
use crate::ibm::IBMQiskitRuntimeServiceProvider;
use crate::ibm::IBMQuantumComputeServiceProvider;
use crate::ibm::IBMQuantumSystemProvider;
use crate::models::{Config, ResourceType, TaskStatus};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Arc;

/// Integer return codes returned to C.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReturnCode {
    /// Success.
    Success = 0,
    /// Error. Generic/uncategorized failure -- see `qrmi_get_last_error()`
    /// for the message and `qrmi_get_last_error_kind()` for a more specific
    /// machine-readable reason when available.
    Error = 100,
    /// Unexpected null pointer.
    NullPointerError = 101,
    /// A required environment variable was not set.
    EnvVarNotSetError = 102,
    /// A configuration value could not be parsed.
    ParseError = 103,
    /// Dynamic discovery was requested for an unsupported resource type.
    UnsupportedResourceTypeError = 104,
    /// The requested operation is not supported by this resource.
    UnsupportedFunctionError = 105,
    /// The payload variant is not supported by this backend.
    UnsupportedPayloadError = 106,
    /// The task is not in a state that allows the requested operation.
    TaskNotReadyError = 107,
    /// A required key was missing from a provider's environment variable map.
    MissingConfigKeyError = 108,
    /// A value was invalid, whether QRMI itself rejected it locally or a
    /// vendor's API rejected the resulting request after receiving it.
    InvalidInputError = 109,
    /// The named resource (e.g. a backend) does not exist.
    ResourceNotFoundError = 111,
    /// The named task (e.g. a job) does not exist, or has already been
    /// removed.
    TaskNotFoundError = 112,
    /// The request's credentials were missing or rejected.
    AuthenticationFailedError = 113,
    /// A configuration value (or combination of values) was invalid.
    InvalidConfigError = 114,
}

impl From<QrmiErrorKind> for ReturnCode {
    fn from(kind: QrmiErrorKind) -> Self {
        match kind {
            QrmiErrorKind::EnvVarNotSet => ReturnCode::EnvVarNotSetError,
            QrmiErrorKind::ParseError => ReturnCode::ParseError,
            QrmiErrorKind::UnsupportedResourceType => ReturnCode::UnsupportedResourceTypeError,
            QrmiErrorKind::UnsupportedPayload => ReturnCode::UnsupportedPayloadError,
            QrmiErrorKind::UnsupportedFunction => ReturnCode::UnsupportedFunctionError,
            QrmiErrorKind::TaskNotReady => ReturnCode::TaskNotReadyError,
            QrmiErrorKind::MissingConfigKey => ReturnCode::MissingConfigKeyError,
            QrmiErrorKind::InvalidConfig => ReturnCode::InvalidConfigError,
            QrmiErrorKind::ResourceNotFound => ReturnCode::ResourceNotFoundError,
            QrmiErrorKind::TaskNotFound => ReturnCode::TaskNotFoundError,
            QrmiErrorKind::AuthenticationFailed => ReturnCode::AuthenticationFailedError,
            QrmiErrorKind::InvalidInput => ReturnCode::InvalidInputError,
            QrmiErrorKind::Other => ReturnCode::Error,
        }
    }
}

/// C ABI type for `qrmi_log_callback_set`. This is a C-facing detail: it
/// does not appear anywhere in `common.rs`, which only knows about plain
/// Rust closures (see `common::LogSink`). `qrmi_log_callback_set` below
/// adapts one of these into a `LogSink` at registration time.
pub type QrmiLogCallback = Option<
    unsafe extern "C" fn(level: *const c_char, target: *const c_char, message: *const c_char),
>;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    /// Payload that contains Qiskit Primitive input.
    QiskitPrimitive {
        /// Primitive input
        input: *mut c_char,
        /// "estimator" or "sampler"
        program_id: *mut c_char,
    },
    /// Payload for Pasqal Cloud and Pasqal Local resources
    PasqalCloud {
        /// Pulser sequence
        sequence: *mut c_char,
        /// Number of job runs
        job_runs: i32,
    },
    AliceBobFelis {
        /// Human-readable QIR input
        human_qir: *mut c_char,
        /// Input parameters in JSON format
        input_params: *mut c_char,
    },
    /// Payload for IQM Server
    IQMServer {
        /// IQM JSON request body
        iqmjson: *mut c_char,
        /// Job type(circuit, run, sweep)
        job_type: *mut c_char,
        /// submit the job to the timeslot queue instead of the default FIFO queue.
        /// 0 = false, 1 = true
        use_timeslot: c_int,
        /// Optional user-defined tag associated with the job
        tag: *mut c_char,
    },
    /// Payload for OQTOPUS Cloud
    Oqtopus {
        /// Job spec, as a JSON object string matching the keyword
        /// arguments of Python's `OqtopusJobSpec` dataclass, minus
        /// `device_id` (QRMI fills that in automatically). Required
        /// keys: `job_type` (one of "sampling", "estimation",
        /// "multi_manual", "sse") and `program` (array of strings; a
        /// single program is a single-element array; `sse` jobs require
        /// exactly one). Optional keys: `shots` (integer; omit the key
        /// entirely to use OQTOPUS's default of 1000 — sending `null`
        /// fails), `name`, `description`, `transpiler_info`,
        /// `simulator_info`, `mitigation_info`, and `operator`.
        job_spec: *mut c_char,
    },
}

/// A key-value pair
#[repr(C)]
#[derive(Debug)]
pub struct KeyValue {
    /// key
    key: *mut c_char,
    /// value
    value: *mut c_char,
}

/// A set of environment variables
#[repr(C)]
#[derive(Debug)]
pub struct EnvironmentVariables {
    /// Ptr to the first key-value pair in the list
    variables: *mut KeyValue,
    /// Number of key-value pairs included in the list
    length: usize,
}

/// Resource definition in QRMI configuration file
#[repr(C)]
#[derive(Debug)]
pub struct ResourceDef {
    /// resource identifier, e.g. `ibm_kingston`
    name: *mut c_char,
    /// Resource Type
    r#type: ResourceType,
    /// Whether this is a dynamic resource (used with ResourceProvider)
    is_dynamic: bool,
    /// environment variables for this resource
    environments: EnvironmentVariables,
}

/// Type alias for the C ResourceDef struct (used in qrmi_provider_new).
type CResourceDef = ResourceDef;

/// Type alias for key-value map
pub type ConfigMap = EnvironmentVariables;

/// Converts a C `EnvironmentVariables` struct to a Rust `HashMap<String, String>`.
unsafe fn envvars_to_hashmap(
    envvars: &EnvironmentVariables,
) -> anyhow::Result<std::collections::HashMap<String, String>> {
    let mut map = std::collections::HashMap::new();
    for i in 0..envvars.length {
        let kv = &*envvars.variables.add(i);
        if !kv.key.is_null() && !kv.value.is_null() {
            let key = CStr::from_ptr(kv.key)
                .to_str()
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in env key: {}", e))?
                .to_string();
            let value = CStr::from_ptr(kv.value)
                .to_str()
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in env value: {}", e))?
                .to_string();
            map.insert(key, value);
        }
    }
    Ok(map)
}

/// Rebuilds a Rust `models::ResourceDef` from a C `ResourceDef` struct.
unsafe fn rebuild_resource_def(def: &ResourceDef) -> anyhow::Result<crate::models::ResourceDef> {
    let name = if def.name.is_null() {
        String::new()
    } else {
        CStr::from_ptr(def.name)
            .to_str()
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in resource name: {}", e))?
            .to_string()
    };

    let environment = envvars_to_hashmap(&def.environments)?;

    Ok(crate::models::ResourceDef {
        name,
        r#type: def.r#type.clone(),
        is_dynamic: def.is_dynamic,
        environment,
    })
}

/// Quantum resource status
#[derive(Debug)]
pub struct ResourceStatus {
    inner: crate::models::ResourceStatus,
}

/// Quantum resource metadata
#[derive(Debug)]
pub struct ResourceMetadata {
    inner: std::collections::HashMap<String, String>,
}

/// Quantum resource handle
pub struct QuantumResource {
    inner: Box<dyn crate::QuantumResource + Send + Sync>,
    runtime: Arc<tokio::runtime::Runtime>,
}

// Last error
thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
    static LAST_ERROR_KIND: RefCell<QrmiErrorKind> = const { RefCell::new(QrmiErrorKind::Other) };
}

/// Set last error message text with the generic error kind
fn _set_last_error(msg: String) {
    _set_last_error_with_kind(msg, QrmiErrorKind::Other);
}

/// Set last error message text and its machine-readable kind together, so
/// that a message never pairs with the kind of an earlier error.
fn _set_last_error_with_kind(msg: String, kind: QrmiErrorKind) {
    log::error!("{}", msg);
    LAST_ERROR_KIND.with(|cell| *cell.borrow_mut() = kind);
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() =
            Some(CString::new(msg).unwrap_or_else(|_| {
                CString::new("Failed to generate a C-compatible string").unwrap()
            }));
    });
}

/// Records `err` as the last error -- both its message (via `_set_last_error`,
/// retrievable through `qrmi_get_last_error()`) and its machine-readable kind
/// (retrievable through `qrmi_get_last_error_kind()`) -- and returns the
/// `ReturnCode` matching that kind. Centralizing this means a new `QrmiError`
/// variant automatically gets consistent handling everywhere it's used,
/// instead of each call site choosing what to record.
fn _fail(err: QrmiError) -> ReturnCode {
    let kind = err.kind();
    _set_last_error_with_kind(err.to_string(), kind);
    ReturnCode::from(kind)
}

/// Same recording as `_fail`, for call sites that return a pointer (`NULL`
/// on failure) rather than a `ReturnCode` and so can't use `_fail`'s return
/// value directly.
fn _record_error(err: QrmiError) {
    _set_last_error_with_kind(err.to_string(), err.kind());
}

/// Borrows a C string argument as UTF-8, recording which argument was
/// invalid as the last error when it is NULL or not valid UTF-8.
///
/// # Safety
///
/// `ptr` must be NULL or point to a nul-terminated string.
unsafe fn _utf8_arg<'a>(ptr: *const c_char, name: &str) -> Option<&'a str> {
    if ptr.is_null() {
        _set_last_error_with_kind(format!("{} is NULL", name), QrmiErrorKind::InvalidInput);
        return None;
    }
    let value = CStr::from_ptr(ptr).to_str().ok();
    if value.is_none() {
        _set_last_error_with_kind(
            format!("{} is not a valid UTF-8 string", name),
            QrmiErrorKind::InvalidInput,
        );
    }
    value
}

/// Converts a string to be returned to C, recording `error` as the last
/// error when the string contains a NUL byte. `error` never includes the
/// value itself, which may be a token or a large payload.
fn _c_string(value: String, error: &str) -> Option<CString> {
    let value = CString::new(value).ok();
    if value.is_none() {
        _set_last_error(error.to_string());
    }
    value
}

/// Hands `values` to C as a string array to be freed with
/// `qrmi_string_array_free()`. If any value contains a NUL byte, allocates
/// nothing, records `error` as the last error and sets `*count` to 0 and
/// `*array` to NULL.
///
/// # Safety
///
/// `count` and `array` must be non-null.
unsafe fn _c_string_array<'a>(
    values: impl Iterator<Item = &'a String>,
    error: &str,
    count: *mut usize,
    array: *mut *mut *mut c_char,
) -> ReturnCode {
    let Ok(values) = values
        .map(|v| CString::new(v.as_str()))
        .collect::<Result<Vec<_>, _>>()
    else {
        *count = 0;
        *array = std::ptr::null_mut();
        _set_last_error(error.to_string());
        return ReturnCode::Error;
    };
    let raw: Box<[*mut c_char]> = values.into_iter().map(CString::into_raw).collect();
    *count = raw.len();
    *array = Box::into_raw(raw) as *mut *mut c_char;
    ReturnCode::Success
}

/// Returns `$ret` from the enclosing function if `$ptr` is NULL, recording
/// which argument was NULL as the last error.
macro_rules! null_pointer_check {
    ($ptr:ident, $ret:expr) => {
        if $ptr.is_null() {
            _set_last_error(format!("{} is NULL", stringify!($ptr)));
            return $ret;
        }
    };
}

/// Converts a Rust string into a `CString` suitable for handing across the
/// C ABI, stripping any embedded NUL bytes first (a `&str` may contain
/// them; a C string cannot). Used only at the C boundary -- see
/// `qrmi_log_callback_set`'s adapter closure -- because this is where
/// Rust's log records get converted to the C-callable log callback's
/// pointer arguments.
fn sanitized_cstring(value: &str) -> CString {
    CString::new(
        value
            .as_bytes()
            .iter()
            .copied()
            .filter(|byte| *byte != 0)
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default()
}

/// @ingroup Qrmi
/// Registers a QRMI log callback for C hosts.
///
/// Once registered, all log records produced by this library (subject to
/// the active `RUST_LOG` filter, default level `warn`) are routed to
/// `callback` instead of stderr.
///
/// Pass NULL to clear the current callback and revert to the default stderr writer.
///
/// # Safety
///
/// * If `callback` is non-null, it must remain a valid, callable function
///   pointer for as long as it might still be invoked. Because a
///   currently-executing log call may have already captured the previous
///   callback pointer, do not unload code backing a callback immediately
///   after replacing or clearing it — a small number of in-flight calls
///   may still land on the old pointer.
///
/// # Example
///
/// @code
///   void my_log_cb(const char *level, const char *target, const char *message) {
///     fprintf(stderr, "[%s] %s: %s\n", level, target, message);
///   }
///   QrmiReturnCode rc = qrmi_log_callback_set(my_log_cb);
/// @endcode
///
/// @param (callback) [in] Callback function, or NULL to clear.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.20.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_log_callback_set(callback: QrmiLogCallback) -> ReturnCode {
    let sink: Option<crate::common::LogSink> = callback.map(|f| {
        let adapter: crate::common::LogSink = std::sync::Arc::new(move |level, target, message| {
            let level = sanitized_cstring(level.as_str());
            let target = sanitized_cstring(target);
            let message = sanitized_cstring(message);
            // SAFETY: `f` is a C function pointer supplied by the caller of
            // `qrmi_log_callback_set`, which documents the same validity
            // requirement this closure now carries: `f` must remain valid
            // and callable for as long as it might still be invoked.
            unsafe {
                f(level.as_ptr(), target.as_ptr(), message.as_ptr());
            }
        });
        adapter
    });
    let result = crate::common::set_log_sink(sink);
    crate::common::initialize();
    match result {
        Ok(()) => ReturnCode::Success,
        Err(()) => {
            _set_last_error("failed to set the log callback".to_string());
            ReturnCode::Error
        }
    }
}

/// @ingroup Qrmi
/// Free a string allocated by C API
///
/// # Safety
///
/// * `ptr` must be one returned by the related C API such as `qrmi_resource_target()`.
///
/// # Example
///
/// @code
///   char *target = NULL;
///   QrmiReturnCode rc;
///   rc = qrmi_resource_target(qrmi, &target);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("target = %s\n", target);
///     qrmi_string_free(target);
///   }
/// @endcode
///
/// @param (ptr) [in] pointer to the memory to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_string_free(ptr: *mut c_char) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
    ReturnCode::Success
}

/// @ingroup Qrmi
/// Free a string array allocated by C API
///
/// # Safety
///
/// * `size` and `array` must be ones returned by the related C API such as `qrmi_config_resource_names_get()`.
///
/// * Specifying an incorrect `size` value must lead to memory leaks or memory corruption.
///
/// # Example
///
/// @code
///   size_t num_names = 0;
///   char **names = NULL;
///   QrmiReturnCode rc = qrmi_config_resource_names_get(cnf, &num_names, &names);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     for (int i = 0; i < num_names; i++) {
///       printf("[%s]\n", names[i]);
///     }
///     qrmi_string_array_free(num_names, names);
///   }
/// @endcode
///
/// @param (size) [in] number of strings in a string array
/// @param (array) [in] a pointer to a string array to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
/// cbindgen:ptrs-as-arrays=[[array; ]]
pub unsafe extern "C" fn qrmi_string_array_free(
    size: usize,
    array: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    if array.is_null() {
        return ReturnCode::NullPointerError;
    }

    unsafe {
        for i in 0..size {
            let ptr = *array.add(i);
            if !ptr.is_null() {
                let _ = CString::from_raw(ptr);
            }
        }
        let _ = Box::from_raw(std::ptr::slice_from_raw_parts_mut(array, size));
    }
    ReturnCode::Success
}

/// @ingroup QrmiConfig
/// Loads qrmi_config.json and returns it as Config.
///
/// # Safety
///
/// * The memory pointed to by `filename` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `filename`
///
/// # Example
///
/// @code
///   QrmiConfig *cnf = qrmi_config_load("/etc/slurm/qrmi_config.json");
/// @endcode
///
/// @param (filename) [in] qrmi_config.json file path
/// @return A QrmiConfig if succeeded, otherwise NULL. Must call qrmi_config_free() to free if no longer used.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_load(filename: *const c_char) -> *mut Config {
    crate::common::initialize();
    null_pointer_check!(filename, std::ptr::null_mut());

    if let Some(file) = _utf8_arg(filename, "filename") {
        let result = Box::new(Config::load(file));
        match *result {
            Ok(v) => {
                return Box::into_raw(Box::new(v));
            }
            Err(err) => {
                _set_last_error(format!("{:?}", err));
            }
        }
    }
    std::ptr::null_mut()
}

/// @ingroup QrmiConfig
/// Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_config_load() or related functions. Otherwise, or if `ptr` has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_config_load().
///
/// # Example
///
/// @code
///   QrmiConfig *cnf = qrmi_config_load("/etc/slurm/qrmi_config.json");
///   qrmi_config_free(cnf);
/// @endcode
///
/// @param (ptr) a pointer to Config to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_free(ptr: *mut Config) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    ReturnCode::Success
}

/// @ingroup QrmiConfig
/// Returns the resource definition for the specified resource.
///
/// # Safety
///
/// * The memory pointed to by `resource_id` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `resource_id`
///
/// # Example
///
/// @code
///   QrmiConfig *cnf = qrmi_config_load(argv[1]);
///   if (!cnf) {
///     QrmiResourceDef* res = qrmi_config_resource_def_get(cnf, "your_resource_id");
///     if (res != NULL) {
///       printf("%s %d\n", res->name, res->type);
///       QrmiEnvironmentVariables envvars = res->environments;
///       for (int j = 0; j < envvars.length; j++) {
///         QrmiKeyValue envvar = envvars.variables[j];
///         printf("%s = %s\n", envvar.key, envvar.value);
///       }
///     }
///   }
/// @endcode
///
/// @param (config) [in] a Config handle
/// @param (resource_id) [in] resource identifier
/// @return A QrmiResourceDef if succeeded, otherwise NULL. Must call qrmi_config_resource_def_free() to free if no longer used.
///         Also NULL if the name or an environment variable of the resource contains a NUL byte.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_resource_def_get(
    config: *mut Config,
    resource_id: *const c_char,
) -> *mut ResourceDef {
    crate::common::initialize();
    null_pointer_check!(config, std::ptr::null_mut());
    null_pointer_check!(resource_id, std::ptr::null_mut());

    if let Some(id_str) = _utf8_arg(resource_id, "resource_id") {
        if let Some(resource) = (*config).resource_map.get(id_str) {
            // Convert every string before handing any to C, so a failure leaks nothing.
            let converted = CString::new(resource.name.as_str())
                .map_err(|_| "name")
                .and_then(|name| {
                    let envvars = resource
                        .environment
                        .iter()
                        .map(|(key, value)| {
                            Ok((
                                CString::new(key.as_str())
                                    .map_err(|_| "environment variable name")?,
                                CString::new(value.as_str())
                                    .map_err(|_| "environment variable value")?,
                            ))
                        })
                        .collect::<Result<Vec<_>, &str>>()?;
                    Ok((name, envvars))
                });
            let (name, envvars) = match converted {
                Ok(v) => v,
                Err(what) => {
                    _set_last_error(format!("resource {id_str}: {what} contains a NUL byte"));
                    return std::ptr::null_mut();
                }
            };
            let c_envvars: Box<[KeyValue]> = envvars
                .into_iter()
                .map(|(key, value)| KeyValue {
                    key: key.into_raw(),
                    value: value.into_raw(),
                })
                .collect();
            return Box::into_raw(Box::new(ResourceDef {
                name: name.into_raw(),
                r#type: resource.r#type.clone(),
                is_dynamic: resource.is_dynamic,
                environments: EnvironmentVariables {
                    length: c_envvars.len(),
                    variables: Box::into_raw(c_envvars) as *mut KeyValue,
                },
            }));
        }
    }
    std::ptr::null_mut()
}

/// @ingroup QrmiConfig
/// Converts ResourceType to string representation used in qrmi_config.json, e.g.
/// @ref QrmiResourceType::QRMI_RESOURCE_TYPE_IBM_QUANTUM_COMPUTE_SERVICE to `ibm-quantum-compute-service`.
///
/// # Safety
///
/// * `type` must be QrmiResourceType value.
///
/// # Example
///
/// @code
///   char *type_as_str = qrmi_config_resource_type_to_str(QRMI_RESOURCE_TYPE_IBM_QUANTUM_COMPUTE_SERVICE);
///   printf("%s\n", type_as_str);
///   qrmi_string_free(type_as_str);
/// @endcode
///
/// @param type (QrmiResourceType) ResourceType variant
/// @return string representation of ResourceType.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_resource_type_to_str(r#type: ResourceType) -> *const c_char {
    crate::common::initialize();
    if let Ok(type_as_str) = CString::new(r#type.as_str()) {
        return type_as_str.into_raw();
    }
    std::ptr::null()
}

/// @ingroup QrmiConfig
/// Parses the string representation used in qrmi_config.json and
/// `QRMI_JOB_QPU_TYPES`, e.g. `pasqal-local` to
/// @ref QrmiResourceType::QRMI_RESOURCE_TYPE_PASQAL_LOCAL.
///
/// # Safety
///
/// * The memory pointed to by `type_str` must contain a valid nul terminator.
///
/// * `outp` must be non-null.
///
/// # Example
///
/// @code
///   QrmiResourceType type;
///   if (qrmi_config_resource_type_from_str("pasqal-local", &type) == QRMI_RETURN_CODE_SUCCESS) {
///     QrmiQuantumResource *qrmi = qrmi_resource_new("your_resource_name", type);
///   }
/// @endcode
///
/// @param (type_str) [in] string representation of ResourceType
/// @param (outp) [out] ResourceType variant if succeeded
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded, @ref QrmiReturnCode::QRMI_RETURN_CODE_INVALID_INPUT_ERROR for an unknown string.
/// @version 0.27.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_resource_type_from_str(
    type_str: *const c_char,
    outp: *mut ResourceType,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(type_str, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);
    let value = CStr::from_ptr(type_str).to_string_lossy();
    match ResourceType::from_qpu_type_str(&value) {
        Some(v) => {
            *outp = v;
            ReturnCode::Success
        }
        None => _fail(QrmiError::InvalidInput(format!(
            "unknown resource type: {value}"
        ))),
    }
}

/// @ingroup QrmiConfig
/// Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_config_get_resource_def() or related functions. Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_config_resource_def_get().
///
/// # Example
///
/// @code
///   QrmiConfig *cnf = qrmi_config_load(argv[1]);
///   if (!cnf) {
///     QrmiResourceDef* res = qrmi_config_resource_def_get(cnf, "your_resource_id");
///     if (res != NULL) {
///       printf("%s %d\n", res->name, res->type);
///     }
///     qrmi_config_resource_def_free(res);
///   }
/// @endcode
///     
/// @param (ptr) a pointer to ResourceDef to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_resource_def_free(ptr: *mut ResourceDef) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }

    unsafe {
        let resource_def = Box::from_raw(ptr);
        let envvars = resource_def.environments;

        if !resource_def.name.is_null() {
            let _ = CString::from_raw(resource_def.name);
        }
        for i in 0..envvars.length {
            let item = envvars.variables.add(i);
            if !(*item).key.is_null() {
                let _ = CString::from_raw((*item).key);
            }
            if !(*item).value.is_null() {
                let _ = CString::from_raw((*item).value);
            }
        }
        let _ = Vec::from_raw_parts(envvars.variables, envvars.length, envvars.length);
    }
    ReturnCode::Success
}

/// @ingroup QrmiConfig
/// Returns a list of the resource names
///
/// # Safety
///
/// * `config` must have been returned by a previous call to qrmi_config_load().
///
/// * The memory pointed to by `outlen` must have enough room to store size_t value.
///
/// * `names` must be non-null.
///
/// # Example
///
/// @code
///   size_t num_names = 0;
///   char **names = NULL;
///   QrmiReturnCode rc = qrmi_config_resource_names_get(cnf, &num_names, &names);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     for (int i = 0; i < num_names; i++) {
///       printf("[%s]\n", names[i]);
///     }
///     qrmi_string_array_free(num_names, names);
///   }
/// @endcode
///
/// @param (config) [in] A Config handle
/// @param (num_names) [out] number of resource names in the list
/// @param (names) [out] A list of the resource names if succeeded. Must call qrmi_string_array_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
///         @ref QrmiReturnCode::QRMI_RETURN_CODE_ERROR if a name contains a NUL byte;
///         `*num_names` is then 0 and `*names` is NULL.
/// @version 0.6.0
#[no_mangle]
/// cbindgen:ptrs-as-arrays=[[names;]]
pub unsafe extern "C" fn qrmi_config_resource_names_get(
    config: *mut Config,
    num_names: *mut usize,
    names: *mut *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(config, ReturnCode::NullPointerError);
    null_pointer_check!(num_names, ReturnCode::NullPointerError);
    null_pointer_check!(names, ReturnCode::NullPointerError);

    _c_string_array(
        (*config).resource_map.keys(),
        "a resource name contains a NUL byte",
        num_names,
        names,
    )
}

/// @ingroup Qrmi
/// Returns the last error message that occurred during an FFI call from C to Rust.
/// This function is designed to be thread-safe by using thread-local storage, ensuring that
/// each thread retrieves its own error message independently.
///
/// # Behavior
/// * If an API call failed earlier in the current thread, this function returns a pointer to a null-terminated C string (const char*) containing the message of the most recent failure.
/// * If no error was set, it returns a null pointer (NULL).
/// * Reading the message clears it for the current thread. Subsequent calls return NULL until another call fails. qrmi_get_last_error_kind() is not cleared.
/// * A successful call does not clear an unread message, so only read the message after a call has returned a failure code.
/// * Freeing functions such as qrmi_string_free() do not record a message when given NULL, so cleanup does not replace the message being inspected.
///
/// # Thread Safety
/// This function uses Rust's thread_local! macro and RefCell to store error messages per thread.
/// This design mirrors the behavior of errno in POSIX and GetLastError() in Windows, ensuring compatibility with multithreaded C environments.
///
/// # Example
///
/// @code
///   char * last_error = qrmi_get_last_error();
///   if (last_error != NULL) {
///     printf("last error = %s\n", last_error);
///     qrmi_string_free(last_error);
///   }
/// @endcode
///
/// @return message text of the most recent error. The caller takes
/// ownership of the returned string and must release it with
/// `qrmi_string_free()` once done with it (or it will leak). Returns NULL
/// if no error has been recorded yet.
/// @version 0.8.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_get_last_error() -> *mut c_char {
    crate::common::initialize();
    LAST_ERROR.with(|cell| match cell.borrow_mut().take() {
        Some(cstr) => cstr.into_raw(),
        None => std::ptr::null_mut(),
    })
}

/// @ingroup QrmiCore
/// Returns a machine-readable classification of the most recent error
/// encountered during an API call, complementing `qrmi_get_last_error()`'s
/// human-readable message. Unlike `qrmi_get_last_error()`, this value is
/// NOT cleared after being read, since callers typically check it before
/// (or without) reading the message.
///
/// If no QRMI-specific error has been recorded (e.g. the failure came from
/// an unrelated source, or no error has occurred yet), this returns
/// `QRMI_RETURN_CODE_ERROR` -- the generic code -- as a safe default. A
/// failure that records only a message (e.g. a NULL argument) also resets
/// it to `QRMI_RETURN_CODE_ERROR` rather than keeping the classification
/// of an earlier failure.
///
/// The return code of the failing call is authoritative, e.g. for whether
/// qrmi_resource_task_start() created a task. This value is not always
/// equal to it: a NULL argument returns `QRMI_RETURN_CODE_NULL_POINTER_ERROR`
/// but is recorded as `QRMI_RETURN_CODE_ERROR`, and an argument that is not
/// valid UTF-8 can return `QRMI_RETURN_CODE_ERROR` but is recorded as
/// `QRMI_RETURN_CODE_INVALID_INPUT_ERROR`.
///
/// @return A `ReturnCode` describing the kind of the most recent error.
/// @version 0.16.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_get_last_error_kind() -> ReturnCode {
    crate::common::initialize();
    LAST_ERROR_KIND.with(|cell| ReturnCode::from(*cell.borrow()))
}

/// @ingroup QrmiQuantumResource
/// Returns a QrmiQuantumResource handle.
///
/// Created QrmiQuantumResource instance needs to be removed by qrmi_resource_free() call if
/// no longer needed.
///
/// # Safety
///
/// * The memory pointed to by `resource_id` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `resource_id`
///
/// # Example
///
/// @code
///   QrmiQuantumResource *qrmi = qrmi_resource_new("your_resource_name",
///                                                 QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM);
/// @endcode
///
/// @param (resource_id) [in] A resource identifier, i.e. backend name
/// @param (resource_type) [in] QrmiResourceType variant
/// @return a QrmiQuantumResource handle if succeeded, otherwise NULL. Must call qrmi_resource_free() to free if no longer used.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_new(
    resource_id: *const c_char,
    resource_type: ResourceType,
) -> *mut QuantumResource {
    crate::common::initialize();
    null_pointer_check!(resource_id, std::ptr::null_mut());

    if let Some(id_str) = _utf8_arg(resource_id, "resource_id") {
        let res = match crate::common::create_resource(&resource_type, id_str) {
            Ok(v) => v,
            Err(err) => {
                _record_error(err);
                return std::ptr::null_mut();
            }
        };

        let qrmi = Box::new(QuantumResource {
            inner: res,
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
        });
        return Box::into_raw(qrmi);
    }
    std::ptr::null_mut()
}

/// @ingroup QrmiQuantumResource
/// Constructs a QrmiQuantumResource from a config map.
///
/// Created QrmiQuantumResource instance needs to be removed by qrmi_resource_free() call if
/// no longer needed.
///
/// # Safety
///
/// * `config` must be a valid pointer to a QrmiConfigMap struct.
///
/// * The memory pointed to by `resource_id` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `resource_id`
///
/// # Example
///
/// @code
///   QrmiConfigMap config;
///
///   QrmiKeyValue variables[] = {
///       {(char *)"qrmi_warden_url", (char *)"http://localhost:8006"},
///       {(char *)"qrmi_job_id", (char *)"1"},
///       {(char *)"qrmi_job_uid", (char *)"1000"},
///   };
///   config.variables = variables;
///   config.length = 3;
///   QrmiQuantumResource *qrmi = qrmi_resource_new_from_config("your_resource_name",
///                                                 QRMI_RESOURCE_TYPE_PASQAL_LOCAL,
///                                                 &config);
/// @endcode
///
/// @param (resource_id) [in] A resource identifier, i.e. backend name
/// @param (resource_type) [in] QrmiResourceType variant
/// @param (config) [in] Pointer to QrmiConfigMap holding the config map
/// @return a QrmiQuantumResource handle if succeeded, otherwise NULL. Must call qrmi_resource_free() to free if no longer used.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_new_from_config(
    resource_id: *const c_char,
    resource_type: ResourceType,
    config: *const ConfigMap,
) -> *mut QuantumResource {
    crate::common::initialize();
    null_pointer_check!(resource_id, std::ptr::null_mut());
    if config.is_null() {
        _set_last_error("config is NULL".to_string());
        return std::ptr::null_mut();
    }

    let config_map = match envvars_to_hashmap(&*config) {
        Ok(m) => m,
        Err(e) => {
            _set_last_error(format!("{:?}", e));
            return std::ptr::null_mut();
        }
    };

    if let Some(id_str) = _utf8_arg(resource_id, "resource_id") {
        let res =
            match crate::common::create_resource_from_config(&resource_type, id_str, config_map) {
                Ok(v) => v,
                Err(err) => {
                    _record_error(err);
                    return std::ptr::null_mut();
                }
            };

        let qrmi = Box::new(QuantumResource {
            inner: res,
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
        });
        return Box::into_raw(qrmi);
    }
    std::ptr::null_mut()
}

/// @ingroup QrmiQuantumResource
/// Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_resource_new(). Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, returns < 0.
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_resource_new().
///
/// # Example
///
/// @code
///   QrmiQuantumResource *qrmi = qrmi_resource_new("your_resource_name",
///                                                 QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM);
///   if (qrmi != NULL) {
///     qrmi_resource_free(qrmi);
///   }
/// @endcode
///
/// @param (ptr) [in] A QrmiQuantumResource handle to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_free(ptr: *mut QuantumResource) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    ReturnCode::Success
}

/// @ingroup QrmiQuantumResource
/// Returns true if device is accessible, otherwise false.
///
/// @deprecated Use qrmi_resource_status() instead. This function will be
/// removed in a future release.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// # Example
///
/// @code
///   bool is_accessible = false;
///   int rc = qrmi_resource_is_accessible(qrmi, &is_accessible);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     if (is_accessible == false) {
///       printf("%s cannot be accessed.\n", argv[1]);
///     }
///   } else {
///     printf("qrmi_resource_is_accessible() failed.\n");
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (outp) [out] accessible or not
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_is_accessible(
    qrmi: *mut QuantumResource,
    outp: *mut bool,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::Error);

    // Deliberately calls the (deprecated) trait method rather than
    // deriving the answer from status(): each vendor keeps its own
    // definition of "accessible", so this binding's behavior is unchanged.
    #[allow(deprecated)]
    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.is_accessible().await });
    match result {
        Ok(v) => {
            *outp = v;
            ReturnCode::Success
        }
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Returns the status of the specified resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (outp) [out] Resource status if succeeded. Must call qrmi_resource_status_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status(
    qrmi: *mut QuantumResource,
    outp: *mut *mut ResourceStatus,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.status().await });

    match result {
        Ok(v) => {
            *outp = Box::into_raw(Box::new(ResourceStatus { inner: v }));
            ReturnCode::Success
        }
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiResourceStatus
/// Frees the memory space pointed to by `ptr`, which must have been
/// returned by a previous call to qrmi_resource_status(). Otherwise, or
/// if `ptr` has already been freed, segmentation fault occurs. If `ptr`
/// is NULL, returns < 0.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_resource_status().
///
/// # Example
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (ptr) [in] A QrmiResourceStatus handle to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_free(ptr: *mut ResourceStatus) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    ReturnCode::Success
}

/// @ingroup QrmiResourceStatus
/// Returns the resource's status code (Online/Offline/Paused).
///
/// # Safety
///
/// * `status` must have been returned by a previous call to qrmi_resource_status().
///
/// * `outp` must be non-null.
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     QrmiResourceStatusCode code;
///     rc = qrmi_resource_status_code(status, &code);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///       switch (code) {
///         case QRMI_RESOURCE_STATUS_CODE_ONLINE:
///           printf("online\n");
///           break;
///         case QRMI_RESOURCE_STATUS_CODE_OFFLINE:
///           printf("offline\n");
///           break;
///         case QRMI_RESOURCE_STATUS_CODE_PAUSED:
///           printf("paused\n");
///           break;
///         case QRMI_RESOURCE_STATUS_CODE_BUSY:
///           printf("busy\n");
///           break;
///       }
///     }
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (status) [in] A QrmiResourceStatus handle
/// @param (outp) [out] The status code
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_code(
    status: *mut ResourceStatus,
    outp: *mut crate::models::ResourceStatusCode,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(status, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);
    *outp = (*status).inner.status.clone();
    ReturnCode::Success
}

/// @ingroup QrmiResourceStatus
/// Converts a QrmiResourceStatusCode value to a human-readable, lowercase
/// string ("online", "offline", "paused"). Intended for logging
/// and diagnostic output.
///
/// # Example
///
/// @code
///   QrmiResourceStatusCode code;
///   QrmiReturnCode rc = qrmi_resource_status_code(status, &code);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("status=%s\n", qrmi_resource_status_code_to_string(code));
///   }
/// @endcode
///
/// @param (code) [in] A QrmiResourceStatusCode value
/// @return A statically-allocated, human-readable string. Must NOT be
///         freed and remains valid for the lifetime of the program.
/// @version 0.25.0
#[no_mangle]
pub extern "C" fn qrmi_resource_status_code_to_string(
    code: crate::models::ResourceStatusCode,
) -> *const c_char {
    match code {
        crate::models::ResourceStatusCode::Online => c"online".as_ptr(),
        crate::models::ResourceStatusCode::Offline => c"offline".as_ptr(),
        crate::models::ResourceStatusCode::Paused => c"paused".as_ptr(),
    }
}

/// @ingroup QrmiResourceStatus
/// Returns the vendor-specific status reason (e.g. "maintenance",
/// "calibration"), if reported.
///
/// # Safety
///
/// * `status` must have been returned by a previous call to qrmi_resource_status().
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     char *reason = qrmi_resource_status_reason(status);
///     if (reason != NULL) {
///       printf("reason=%s\n", reason);
///       qrmi_string_free(reason);
///     } else {
///       printf("no status reason reported\n");
///     }
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (status) [in] A QrmiResourceStatus handle
/// @return status reason string if the vendor reported one. Must call
///         qrmi_string_free() to free if no longer used. Returns NULL if
///         `status` is NULL or the vendor did not report a reason.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_reason(status: *mut ResourceStatus) -> *mut c_char {
    crate::common::initialize();
    if status.is_null() {
        return std::ptr::null_mut();
    }
    match &(*status).inner.status_reason {
        Some(reason) => match _c_string(reason.clone(), "status reason contains a NUL byte") {
            Some(cstr) => cstr.into_raw(),
            None => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

/// @ingroup QrmiResourceStatus
/// Returns whether the resource is currently healthy.
///
/// # Safety
///
/// * `status` must have been returned by a previous call to qrmi_resource_status().
///
/// * `outp` must be non-null.
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     bool healthy = false;
///     rc = qrmi_resource_status_healthy(status, &healthy);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///       printf("healthy=%d\n", healthy);
///     } else if (rc == QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR) {
///       printf("this vendor does not report health information\n");
///     } else {
///       printf("qrmi_resource_status_healthy() failed: %s\n", qrmi_get_last_error());
///     }
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (status) [in] A QrmiResourceStatus handle
/// @param (outp) [out] Whether the resource is healthy
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
///         @ref QrmiReturnCode::QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR
///         if the vendor does not report health information.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_healthy(
    status: *mut ResourceStatus,
    outp: *mut bool,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(status, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);
    match (*status).inner.healthy {
        Some(v) => {
            *outp = v;
            ReturnCode::Success
        }
        None => _fail(QrmiError::UnsupportedFunction(
            "this vendor does not report health information".to_string(),
        )),
    }
}

/// @ingroup QrmiResourceStatus
/// Returns whether the resource is currently busy.
///
/// # Safety
///
/// * `status` must have been returned by a previous call to qrmi_resource_status().
///
/// * `outp` must be non-null.
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     bool busy = false;
///     rc = qrmi_resource_status_busy(status, &busy);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///       printf("busy=%d\n", busy);
///     } else if (rc == QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR) {
///       printf("this vendor does not report busy information\n");
///     } else {
///       printf("qrmi_resource_status_busy() failed: %s\n", qrmi_get_last_error());
///     }
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (status) [in] A QrmiResourceStatus handle
/// @param (outp) [out] Whether the resource is busy
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
///         @ref QrmiReturnCode::QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR
///         if the vendor does not report health information.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_busy(
    status: *mut ResourceStatus,
    outp: *mut bool,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(status, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);
    match (*status).inner.busy {
        Some(v) => {
            *outp = v;
            ReturnCode::Success
        }
        None => _fail(QrmiError::UnsupportedFunction(
            "this vendor does not report busy information".to_string(),
        )),
    }
}

/// @ingroup QrmiResourceStatus
/// Returns the number of jobs pending in the queue.
///
/// # Safety
///
/// * `status` must have been returned by a previous call to qrmi_resource_status().
///
/// * `outp` must be non-null.
///
/// @code
///   QrmiResourceStatus *status = NULL;
///   QrmiReturnCode rc = qrmi_resource_status(qrmi, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     uint64_t pending_job_count = 0;
///     rc = qrmi_resource_status_pending_job_count(status, &pending_job_count);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///       printf("pending_job_count=%llu\n", (unsigned long long)pending_job_count);
///     } else if (rc == QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR) {
///       printf("this resource has no queue / does not report pending job count\n");
///     } else {
///       printf("qrmi_resource_status_pending_job_count() failed: %s\n", qrmi_get_last_error());
///     }
///     qrmi_resource_status_free(status);
///   }
/// @endcode
///
/// @param (status) [in] A QrmiResourceStatus handle
/// @param (outp) [out] Number of pending jobs
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
///         @ref QrmiReturnCode::QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR
///         if the resource has no queue / the vendor does not report this.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_pending_job_count(
    status: *mut ResourceStatus,
    outp: *mut u64,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(status, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);
    match (*status).inner.pending_job_count {
        Some(v) => {
            *outp = v;
            ReturnCode::Success
        }
        None => _fail(QrmiError::UnsupportedFunction(
            "this resource has no queue / does not report pending job count".to_string(),
        )),
    }
}

/// @ingroup QrmiResourceStatus
/// Returns the resource's capacity information, if reported.
///
/// # Safety
///
/// * `status` must have been returned by a previous call to qrmi_resource_status().
///
/// * `outp` must be non-null.
///
/// @code
///   QrmiResourceCapacity *capacity = NULL;
///   QrmiReturnCode rc = qrmi_resource_status_capacity(status, &capacity);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("available_slots=%u\n", capacity->available_slots);
///     qrmi_resource_capacity_free(capacity);
///   }
/// @endcode
///
/// @param (status) [in] A QrmiResourceStatus handle
/// @param (outp) [out] Capacity handle if succeeded. Must call
///        qrmi_resource_capacity_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
///         @ref QrmiReturnCode::QRMI_RETURN_CODE_UNSUPPORTED_FUNCTION_ERROR
///         if the vendor does not report capacity information.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_status_capacity(
    status: *mut ResourceStatus,
    outp: *mut *mut crate::models::ResourceCapacity,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(status, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);
    match (*status).inner.capacity.clone() {
        Some(cap) => {
            *outp = Box::into_raw(Box::new(cap));
            ReturnCode::Success
        }
        None => _fail(QrmiError::UnsupportedFunction(
            "this vendor does not report capacity information".to_string(),
        )),
    }
}

/// @ingroup QrmiResourceStatus
/// Frees a QrmiResourceCapacity handle returned by
/// qrmi_resource_status_capacity().
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to
///   qrmi_resource_status_capacity(). Otherwise, or if `ptr` has already
///   been freed, undefined behavior occurs.
///
/// @code
///   QrmiResourceCapacity *capacity = NULL;
///   QrmiReturnCode rc = qrmi_resource_status_capacity(status, &capacity);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("available_slots=%u\n", capacity->available_slots);
///     qrmi_resource_capacity_free(capacity);
///   }
/// @endcode
///
/// @param (ptr) [in] A QrmiResourceCapacity handle to be freed
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.25.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_capacity_free(
    ptr: *mut crate::models::ResourceCapacity,
) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    ReturnCode::Success
}

/// @ingroup QrmiQuantumResource
/// Returns resource identifier.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_id(
    qrmi: *mut QuantumResource,
    outp: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.resource_id().await });
    match result {
        Ok(v) => match _c_string(v, "resource ID contains a NUL byte") {
            Some(id_cstr) => {
                *outp = id_cstr.into_raw();
                ReturnCode::Success
            }
            None => ReturnCode::Error,
        },
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Returns resource type.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_type(
    qrmi: *mut QuantumResource,
    outp: *mut ResourceType,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::Error);

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.resource_type().await });
    match result {
        Ok(v) => {
            *outp = v;
            ReturnCode::Success
        }
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Acquires quantum resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
///
/// # Example
///
/// @code
///   char *acquisition_token;
///   QrmiReturnCode rc = qrmi_resource_acquire(qrmi, &acquisition_token);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("acquisition token = %s\n", acquisition_token);
///   }
///   else {
///     printf("qrmi_resource_acquire failed.");
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (acquisition_token) [out] An acquisition token if succeeded. Must call qrmi_string_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_acquire(
    qrmi: *mut QuantumResource,
    acquisition_token: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(acquisition_token, ReturnCode::NullPointerError);

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.acquire().await });
    match result {
        Ok(token) => match _c_string(
            token,
            "resource was acquired, but the acquisition token contains a NUL byte",
        ) {
            Some(token_cstr) => {
                *acquisition_token = token_cstr.into_raw();
                ReturnCode::Success
            }
            None => ReturnCode::Error,
        },
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Releases quantum resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `acquisition_token` must contain the nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `acquisition_token`
///
/// # Example
///
/// @code
///   char *acquisition_token = NULL;
///   QrmiReturnCode rc = qrmi_resource_acquire(qrmi, &acquisition_token);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     rc = qrmi_resource_release(qrmi, acquisition_token);
///     if (rc != QRMI_RETURN_CODE_SUCCESS) {
///       printf("Failed to release a quantum resource\n");
///     }
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (acquisition_token) [in] An acquisition token returned by qrmi_resource_acquire() call.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_release(
    qrmi: *mut QuantumResource,
    acquisition_token: *const c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(acquisition_token, ReturnCode::Error);

    let Some(token) = _utf8_arg(acquisition_token, "acquisition_token") else {
        return ReturnCode::Error;
    };
    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.release(token).await });
    match result {
        Ok(()) => ReturnCode::Success,
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Starts a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `task_id` must be non-null.
///
/// * The memory pointed to by `input` and `program_id` in QrmiPayload_QiskitPrimitive_Body contain a valid nul terminator.
///
/// * The memory pointed to by `sequence` in QrmiPayload_PasqalCloud_Body must contain a valid nul terminator.
///
/// * The memory pointed to by `job_spec` in QrmiPayload_Oqtopus_Body
///   must contain a valid nul terminator.
///
/// # Example
///
/// @code
///   QrmiPayload payload;
///   char *job_id = NULL;
///   QrmiReturnCode rc;
///   const char* input = "your Qiskit estimator primitive input";
///
///   payload.tag = QRMI_PAYLOAD_QISKIT_PRIMITIVE;
///   payload.QISKIT_PRIMITIVE.input = (char *)input;
///   payload.QISKIT_PRIMITIVE.program_id = "estimator";
///
///   rc = qrmi_resource_task_start(qrmi, &payload, &job_id);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("Job ID: %s\n", job_id);
///   }
///   else {
///     printf("failed to start a task.\n");
///   }
/// @endcode
///
/// Any failure code other than @ref QrmiReturnCode::QRMI_RETURN_CODE_ERROR means
/// that no task was created: QRMI or the vendor's API refused the request.
/// @ref QrmiReturnCode::QRMI_RETURN_CODE_ERROR leaves that unknown (e.g. a lost
/// response), so the task may exist. A required payload string that is NULL
/// or not valid UTF-8 is rejected with
/// @ref QrmiReturnCode::QRMI_RETURN_CODE_INVALID_INPUT_ERROR before anything
/// is sent. The optional IQM `tag` may be NULL.
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (payload) [in] payload
/// @param (task_id) [out] A task identifier if succeeded. Must call qrmi_string_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_task_start(
    qrmi: *mut QuantumResource,
    payload: *const Payload,
    task_id: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(payload, ReturnCode::NullPointerError);
    null_pointer_check!(task_id, ReturnCode::NullPointerError);

    let mut qrmi_payload: Option<crate::models::Payload> = None;
    if let Payload::QiskitPrimitive { input, program_id } = *payload {
        if let (Some(program_id_str), Some(input_str)) = (
            _utf8_arg(program_id, "program_id"),
            _utf8_arg(input, "input"),
        ) {
            qrmi_payload = Some(crate::models::Payload::QiskitPrimitive {
                input: input_str.to_string(),
                program_id: program_id_str.to_string(),
            });
        }
    } else if let Payload::PasqalCloud { sequence, job_runs } = *payload {
        if let Some(sequence_str) = _utf8_arg(sequence, "sequence") {
            qrmi_payload = Some(crate::models::Payload::PasqalCloud {
                sequence: sequence_str.to_string(),
                job_runs,
            });
        }
    } else if let Payload::AliceBobFelis {
        human_qir,
        input_params,
    } = *payload
    {
        if let (Some(human_qir_str), Some(input_params_str)) = (
            _utf8_arg(human_qir, "human_qir"),
            _utf8_arg(input_params, "input_params"),
        ) {
            qrmi_payload = Some(crate::models::Payload::AliceBobFelis {
                human_qir: human_qir_str.to_string(),
                input_params: input_params_str.to_string(),
            });
        }
    } else if let Payload::IQMServer {
        iqmjson,
        job_type,
        tag,
        use_timeslot,
    } = *payload
    {
        // A NULL tag means no tag.
        if let (Some(json_str), Some(type_str), Some(tag_opt)) = (
            _utf8_arg(iqmjson, "iqmjson"),
            _utf8_arg(job_type, "job_type"),
            if tag.is_null() {
                Some(None)
            } else {
                _utf8_arg(tag, "tag").map(|s| Some(s.to_string()))
            },
        ) {
            let use_timeslot_opt = match use_timeslot {
                1 => Some(true),
                _ => Some(false),
            };

            qrmi_payload = Some(crate::models::Payload::IQMServer {
                iqmjson: json_str.to_string(),
                job_type: type_str.to_string(),
                tag: tag_opt,
                use_timeslot: use_timeslot_opt,
            });
        }
    } else if let Payload::Oqtopus { job_spec } = *payload {
        if let Some(job_spec_str) = _utf8_arg(job_spec, "job_spec") {
            qrmi_payload = Some(crate::models::Payload::Oqtopus {
                job_spec: job_spec_str.to_string(),
            });
        }
    }

    // Nothing has been sent yet, so this is a refusal rather than an unknown
    // outcome. _utf8_arg recorded which argument was invalid.
    let Some(qrmi_payload) = qrmi_payload else {
        return ReturnCode::InvalidInputError;
    };
    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.task_start(qrmi_payload).await });
    match result {
        // The task exists, but its identifier cannot be returned.
        Ok(job_id) => match _c_string(job_id, "task was created, but its ID contains a NUL byte") {
            Some(job_id_cstr) => {
                *task_id = job_id_cstr.into_raw();
                ReturnCode::Success
            }
            None => ReturnCode::Error,
        },
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Stops a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// # Example
///
/// @code
///   QrmiReturnCode rc = qrmi_resource_task_stop(qrmi, job_id);
///   if (rc != QRMI_RETURN_CODE_SUCCESS) {
///     printf("Failed to stop a task\n");
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (task_id) [in] A task ID, returned by a previous call to qrmi_resource_task_start()
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_task_stop(
    qrmi: *mut QuantumResource,
    task_id: *const c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);

    null_pointer_check!(task_id, ReturnCode::Error);

    if let Some(task_id_str) = _utf8_arg(task_id, "task_id") {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_stop(task_id_str).await });
        match result {
            Ok(()) => {
                return ReturnCode::Success;
            }
            Err(err) => {
                return _fail(err);
            }
        }
    }
    ReturnCode::Error
}

/// @ingroup QrmiQuantumResource
/// Returns the status of the specified task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator.
///
/// * The memory pointed to by `status` must have enough room to store `QrmiTaskStatus` value.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// # Example
///
/// @code
///   QrmiTaskStatus status;
///   while (1) {
///     rc = qrmi_resource_task_status(qrmi, job_id, &status);
///     if (rc != QRMI_RETURN_CODE_SUCCESS || status != QRMI_TASK_STATUS_RUNNING) {
///       break;
///     }
///     sleep(1);
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (task_id) [in] A task identifier
/// @param (status) [out] A pointer to the memory to store `QrmiTaskStatus` value
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_task_status(
    qrmi: *mut QuantumResource,
    task_id: *const c_char,
    status: *mut TaskStatus,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);

    null_pointer_check!(task_id, ReturnCode::Error);
    null_pointer_check!(status, ReturnCode::Error);

    if let Some(task_id_str) = _utf8_arg(task_id, "task_id") {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_status(task_id_str).await });
        match result {
            Ok(v) => {
                *status = v;
                return ReturnCode::Success;
            }
            Err(err) => {
                return _fail(err);
            }
        }
    }
    ReturnCode::Error
}

/// @ingroup QrmiQuantumResource
/// Returns the result of a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// # Example
///
/// @code
///   QrmiReturnCode rc = qrmi_resource_task_status(qrmi, job_id, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS && status == QRMI_TASK_STATUS_COMPLETED) {
///     char *result = NULL;
///     qrmi_resource_task_result(qrmi, job_id, &result);
///     printf("%s\n", result);
///     qrmi_string_free((char *)result);
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (task_id) [in] A task identifier
/// @param (outp) [out] Task result if succeeded. Must call qrmi_string_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_task_result(
    qrmi: *mut QuantumResource,
    task_id: *const c_char,
    outp: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);

    null_pointer_check!(task_id, ReturnCode::Error);
    null_pointer_check!(outp, ReturnCode::Error);

    if let Some(task_id_str) = _utf8_arg(task_id, "task_id") {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_result(task_id_str).await });
        return match result {
            Ok(v) => match _c_string(v.value, "task result contains a NUL byte") {
                Some(result_cstr) => {
                    *outp = result_cstr.into_raw();
                    ReturnCode::Success
                }
                None => ReturnCode::Error,
            },
            Err(err) => _fail(err),
        };
    }
    ReturnCode::Error
}

/// @ingroup QrmiQuantumResource
/// Returns the log messages of a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// # Example
///
/// @code
///   QrmiReturnCode rc = qrmi_resource_task_status(qrmi, job_id, &status);
///   if (rc == QRMI_RETURN_CODE_SUCCESS && status == QRMI_TASK_STATUS_COMPLETED) {
///     char *logs = NULL;
///     qrmi_resource_task_logs(qrmi, job_id, &logs);
///     printf("%s\n", logs);
///     qrmi_string_free((char *)logs);
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (task_id) [in] A task identifier
/// @param (outp) [out] Task log messages if succeeded. Must call qrmi_string_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_task_logs(
    qrmi: *mut QuantumResource,
    task_id: *const c_char,
    outp: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);

    null_pointer_check!(task_id, ReturnCode::Error);
    null_pointer_check!(outp, ReturnCode::Error);

    if let Some(task_id_str) = _utf8_arg(task_id, "task_id") {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_logs(task_id_str).await });
        return match result {
            Ok(v) => match _c_string(v, "task logs contain a NUL byte") {
                Some(result_cstr) => {
                    *outp = result_cstr.into_raw();
                    ReturnCode::Success
                }
                None => ReturnCode::Error,
            },
            Err(err) => _fail(err),
        };
    }
    ReturnCode::Error
}

/// @ingroup QrmiQuantumResource
/// Returns a Target for the specified device. Vendor specific serialized data. This might contain the constraints(instructions, properties and timing information etc.) of a particular device to allow compilers to compile an input circuit to something that works and is optimized for a device. In IBM implementation, it contains JSON representations of [BackendConfiguration](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json) and [BackendProperties](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json) so that we are able to create a Target object by calling `qiskit_ibm_runtime.utils.backend_converter.convert_to_target` or uquivalent functions.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
///
/// # Example
///
/// @code
///   char *target = NULL;
///   QrmiReturnCode rc;
///   rc = qrmi_resource_target(qrmi, &target);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     printf("target = %s\n", target);
///     qrmi_string_free(target);
///   }
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (outp) [out] A serialized target data if succeeded. Must call qrmi_string_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_target(
    qrmi: *mut QuantumResource,
    outp: *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::Error);
    null_pointer_check!(outp, ReturnCode::NullPointerError);

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.target().await });
    match result {
        Ok(v) => match _c_string(v.value, "target contains a NUL byte") {
            Some(target_cstr) => {
                *outp = target_cstr.into_raw();
                ReturnCode::Success
            }
            None => ReturnCode::Error,
        },
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiQuantumResource
/// Returns a resource metadata
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non-null.
///
/// # Example
///
/// @code
///   QrmiResourceMetadata *metadata = NULL;
///   QrmiReturnCode rc = qrmi_resource_metadata(qrmi, &metadata);
/// @endcode
///
/// @param (qrmi) [in] A QrmiQuantumResource handle
/// @param (outp) [out] A QrmiResourceMetadata handle. Must call qrmi_resource_metadata_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_metadata(
    qrmi: *mut QuantumResource,
    outp: *mut *mut ResourceMetadata,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(qrmi, ReturnCode::NullPointerError);
    null_pointer_check!(outp, ReturnCode::NullPointerError);

    let metadata = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.metadata().await });

    let boxed_metadata = Box::new(ResourceMetadata { inner: metadata });
    unsafe {
        *outp = Box::into_raw(boxed_metadata);
    }
    ReturnCode::Success
}

/// @ingroup QrmiResourceMetadata
/// Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_resource_metadata(). Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, returns < 0.
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_resource_metadata().
///
/// # Example
///
/// @code
///   QrmiResourceMetadata *metadata = NULL;
///   QrmiReturnCode rc = qrmi_resource_metadata(qrmi, &metadata);
///   if (retval == QRMI_RETURN_CODE_SUCCESS) {
///     qrmi_resource_metadata_free(metadata);
///   }
/// @endcode
///
/// @param (ptr) [in] A QrmiResourceMetadata handle to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_metadata_free(ptr: *mut ResourceMetadata) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    ReturnCode::Success
}

/// @ingroup QrmiResourceMetadata
/// Returns metadata value of the specified key
///
/// # Safety
///
/// * `metadata` must have been returned by a previous call to qrmi_resource_metadata().
///
/// * The memory pointed to by `key` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `key`.
///
/// # Example
///
/// @code
///   char *value = qrmi_resource_metadata_value(metadata, "backend_name");
///   printf("metadata value=[%s]\n", value);
///   qrmi_string_free(value);
/// @endcode
///
/// @param (metadata) [in] A QrmiResourceMetadata handle
/// @param (key) [in] metadata key name
/// @return metadata value if succeeded. Must call qrmi_string_free() to free if no longer used.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_metadata_value(
    metadata: *mut ResourceMetadata,
    key: *const c_char,
) -> *mut c_char {
    crate::common::initialize();
    null_pointer_check!(metadata, std::ptr::null_mut());
    null_pointer_check!(key, std::ptr::null_mut());

    if let Some(key_str) = _utf8_arg(key, "key") {
        if let Some(val) = (*metadata).inner.get(key_str) {
            if let Some(value_cstr) = _c_string(val.clone(), "metadata value contains a NUL byte") {
                return value_cstr.into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

/// @ingroup QrmiResourceMetadata
/// Returns a list of the metadata keys
///
/// # Safety
///
/// * `metadata` must have been returned by a previous call to qrmi_resource_metadata().
///
/// * `num_keys` and `key_names` must be non-null.
///
/// # Example
///
/// @code
///   size_t num_keys = 0;
///   char **metadata_keys = NULL;
///   QrmiReturnCode rc = qrmi_resource_metadata_keys(metadata, &num_keys, &metadata_keys);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     for (int i = 0; i < num_keys; i++) {
///       printf("%s\n", metadata_keys[i]);
///     }
///     qrmi_string_array_free(num_keys, metadata_keys);
///   }
/// @endcode
///
/// @param (metadata) [in] A QrmiResourceMetadata handle
/// @param (num_keys) [out] number of keys available in the metadata
/// @param (key_names) [out] A list of metadata key names if succeeded. Must call qrmi_string_array_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
///         @ref QrmiReturnCode::QRMI_RETURN_CODE_ERROR if a key contains a NUL byte;
///         `*num_keys` is then 0 and `*key_names` is NULL.
/// cbindgen:ptrs-as-arrays=[[key_names;]]
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_metadata_keys(
    metadata: *mut ResourceMetadata,
    num_keys: *mut usize,
    key_names: *mut *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(metadata, ReturnCode::NullPointerError);
    null_pointer_check!(num_keys, ReturnCode::NullPointerError);
    null_pointer_check!(key_names, ReturnCode::NullPointerError);

    _c_string_array(
        (*metadata).inner.keys(),
        "a metadata key contains a NUL byte",
        num_keys,
        key_names,
    )
}

// ---------------------------------------------------------------------------
// ResourceProvider C bindings
// ---------------------------------------------------------------------------

/// Resource provider handle.
pub struct ResourceProvider {
    inner: Box<dyn crate::ResourceProvider>,
    runtime: Arc<tokio::runtime::Runtime>,
}

/// @ingroup QrmiResourceProvider
/// Returns a QrmiResourceProvider handle for the specified resource type and environment.
///
/// Created handle must be released with qrmi_provider_free() when no longer needed.
///
/// Currently supported resource types:
/// - @ref QrmiResourceType::QRMI_RESOURCE_TYPE_QISKIT_RUNTIME_SERVICE(deprecated)
/// - @ref QrmiResourceType::QRMI_RESOURCE_TYPE_IBM_QUANTUM_COMPUTE_SERVICE
/// - @ref QrmiResourceType::QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM
///
/// # Safety
///
/// * `environments` must be a valid pointer to a QrmiEnvironmentVariables struct.
///
/// # Example
///
/// @code
///   QrmiConfig *config = qrmi_config_load("/path/to/qrmi_config.json");
///   QrmiResourceDef *def = qrmi_config_resource_def_get(config, "ibm_inst1");
///   QrmiResourceProvider *provider = qrmi_provider_new(def->type, &def->environments);
///   if (provider == NULL) {
///     char *err = qrmi_get_last_error();
///     printf("error: %s\n", err);
///     qrmi_string_free(err);
///   }
/// @endcode
///
/// @param (resource_type) [in] QrmiResourceType variant
/// @param (environments)  [in] Pointer to QrmiEnvironmentVariables
/// @return A QrmiResourceProvider handle if succeeded, otherwise NULL.
///         Must call qrmi_provider_free() to free if no longer used.
/// @version 0.15.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_provider_new(
    resource_type: ResourceType,
    environments: *const EnvironmentVariables,
) -> *mut ResourceProvider {
    crate::common::initialize();
    if environments.is_null() {
        _set_last_error("environments is NULL".to_string());
        return std::ptr::null_mut();
    }

    // Convert EnvironmentVariables to HashMap<String, String>
    let env_map = match envvars_to_hashmap(&*environments) {
        Ok(m) => m,
        Err(e) => {
            _set_last_error(format!("{:?}", e));
            return std::ptr::null_mut();
        }
    };

    let provider: Box<dyn crate::ResourceProvider> = match resource_type {
        ResourceType::QiskitRuntimeService => {
            match IBMQiskitRuntimeServiceProvider::new(&env_map) {
                Ok(inner) => Box::new(inner),
                Err(err) => {
                    _record_error(err);
                    return std::ptr::null_mut();
                }
            }
        }
        ResourceType::IBMQuantumComputeService => {
            match IBMQuantumComputeServiceProvider::new(&env_map) {
                Ok(inner) => Box::new(inner),
                Err(err) => {
                    _record_error(err);
                    return std::ptr::null_mut();
                }
            }
        }
        ResourceType::IBMQuantumSystem => match IBMQuantumSystemProvider::new(&env_map) {
            Ok(inner) => Box::new(inner),
            Err(err) => {
                _record_error(err);
                return std::ptr::null_mut();
            }
        },
        _ => {
            _record_error(QrmiError::UnsupportedResourceType(format!(
                "{resource_type:?}"
            )));
            return std::ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(ResourceProvider {
        inner: provider,
        runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
    }))
}

/// @ingroup QrmiResourceProvider
/// Frees the memory space pointed to by `ptr`, which must have been returned by
/// a previous call to qrmi_provider_new(). If `ptr` is NULL, returns NullPointerError.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_provider_new().
///
/// # Example
///
/// @code
///   QrmiConfig *config = qrmi_config_load("/path/to/qrmi_config.json");
///   QrmiResourceDef *def = qrmi_config_resource_def_get(config, "ibm_inst1");
///   QrmiResourceProvider *provider = qrmi_provider_new(def->type, &def->environments);
///   if (provider != NULL) {
///     qrmi_provider_free(provider);
///   }
/// @endcode
///
/// @param (ptr) [in] A QrmiResourceProvider handle to be freed
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.15.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_provider_free(ptr: *mut ResourceProvider) -> ReturnCode {
    crate::common::initialize();
    if ptr.is_null() {
        return ReturnCode::NullPointerError;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    }
    ReturnCode::Success
}

/// A list of [`QuantumResource`] handles returned by [`qrmi_provider_resources`].
///
/// Must be freed with [`qrmi_provider_resources_free`] when no longer needed.
#[repr(C)]
pub struct QuantumResources {
    /// Pointer to the first element in the contiguous array of `QrmiQuantumResource` handles.
    pub resources: *mut *mut QuantumResource,
    /// Number of handles in the array.
    pub length: usize,
}

impl Default for QuantumResources {
    fn default() -> Self {
        Self {
            resources: std::ptr::null_mut(),
            length: 0,
        }
    }
}

/// @ingroup QrmiResourceProvider
/// Returns a list of available quantum resources, optionally filtered.
///
/// Results are expected to be sorted in least-busy order.
///
/// The caller is responsible for freeing the returned struct with
/// qrmi_provider_resources_free(). Individual handles inside the struct
/// must NOT be freed separately.
///
/// # Safety
///
/// * `provider` must have been returned by a previous call to qrmi_provider_new().
/// * `filters` may be NULL (no filtering) or a valid nul-terminated C string.
/// * `resources_out` must be non-null and point to a zero-initialized QrmiQuantumResources.
///
/// # Filter string format
///
/// `key=value` pairs joined by `&`. Supported filters(constraints) are defined by each resource provider's implementation.
///
/// # Example
///
/// @code
///   QrmiQuantumResources resources = {0};
///   QrmiReturnCode rc = qrmi_provider_resources(provider, "num_qubits=127", &resources);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     for (size_t i = 0; i < resources.length; i++) {
///       char *id = NULL;
///       qrmi_resource_id(resources.resources[i], &id);
///       printf("resource: %s\n", id);
///       qrmi_string_free(id);
///     }
///     qrmi_provider_resources_free(&resources);
///   }
/// @endcode
///
/// @param (provider)      [in]  A QrmiResourceProvider handle
/// @param (filters)       [in]  Filter string, or NULL for no filtering
/// @param (resources_out) [out] Pointer to a QrmiQuantumResources struct to populate
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.15.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_provider_resources(
    provider: *mut ResourceProvider,
    filters: *const c_char,
    resources_out: *mut QuantumResources,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(provider, ReturnCode::NullPointerError);
    null_pointer_check!(resources_out, ReturnCode::NullPointerError);

    let filters_opt: Option<String> = if filters.is_null() {
        None
    } else {
        match CStr::from_ptr(filters).to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                _set_last_error("filters: invalid UTF-8 string".to_string());
                return ReturnCode::Error;
            }
        }
    };

    let result = (*provider)
        .runtime
        .block_on(async { (*provider).inner.resources(filters_opt).await });

    match result {
        Ok(resource_list) => {
            let count = resource_list.len();
            let mut raw_ptrs: Vec<*mut QuantumResource> = resource_list
                .into_iter()
                .map(|r| {
                    Box::into_raw(Box::new(QuantumResource {
                        inner: r,
                        runtime: (*provider).runtime.clone(),
                    }))
                })
                .collect();

            let ptr = raw_ptrs.as_mut_ptr();
            std::mem::forget(raw_ptrs);

            (*resources_out).resources = ptr;
            (*resources_out).length = count;
            ReturnCode::Success
        }
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiResourceProvider
/// Frees a QrmiQuantumResources struct populated by qrmi_provider_resources().
///
/// This frees both the individual QrmiQuantumResource handles and the internal
/// array. After calling this, the struct's fields are zeroed.
/// Do NOT call qrmi_resource_free() on individual elements after calling this.
///
/// # Safety
///
/// * `resources` must have been populated by a previous call to qrmi_provider_resources().
///
/// # Example
///
/// @code
///   qrmi_provider_resources_free(&resources);
/// @endcode
///
/// @param (resources) [in] Pointer to a QrmiQuantumResources struct to free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.15.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_provider_resources_free(
    resources: *mut QuantumResources,
) -> ReturnCode {
    crate::common::initialize();
    if resources.is_null() {
        return ReturnCode::NullPointerError;
    }
    let length = (*resources).length;
    let ptr = (*resources).resources;
    if !ptr.is_null() {
        for i in 0..length {
            let p = *ptr.add(i);
            if !p.is_null() {
                let _ = Box::from_raw(p);
            }
        }
        let _ = Vec::from_raw_parts(ptr, length, length);
    }
    (*resources).resources = std::ptr::null_mut();
    (*resources).length = 0;
    ReturnCode::Success
}

/// @ingroup QrmiResourceProvider
/// Returns the least busy available quantum resource, optionally filtered.
///
/// Equivalent to calling qrmi_provider_resources() and taking the first element.
/// If no resources match the filter, `*resource_out` is set to NULL and the
/// function returns QRMI_RETURN_CODE_SUCCESS.
///
/// The returned QrmiQuantumResource handle must be freed with qrmi_resource_free()
/// when no longer needed.
///
/// # Safety
///
/// * `provider` must have been returned by a previous call to qrmi_provider_new().
/// * `filters` may be NULL (no filtering) or a valid nul-terminated C string.
/// * `resource_out` must be non-null.
///
/// # Example
///
/// @code
///   QrmiQuantumResource *resource = NULL;
///   QrmiReturnCode rc = qrmi_provider_least_busy(provider, NULL, &resource);
///   if (rc == QRMI_RETURN_CODE_SUCCESS && resource != NULL) {
///     char *id = NULL;
///     qrmi_resource_id(resource, &id);
///     printf("least busy: %s\n", id);
///     qrmi_string_free(id);
///     qrmi_resource_free(resource);
///   }
/// @endcode
///
/// @param (provider)     [in]  A QrmiResourceProvider handle
/// @param (filters)      [in]  Filter string, or NULL for no filtering
/// @param (resource_out) [out] Least busy QrmiQuantumResource handle, or NULL if none found
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.15.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_provider_least_busy(
    provider: *mut ResourceProvider,
    filters: *const c_char,
    resource_out: *mut *mut QuantumResource,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(provider, ReturnCode::NullPointerError);
    null_pointer_check!(resource_out, ReturnCode::NullPointerError);

    let filters_opt: Option<String> = if filters.is_null() {
        None
    } else {
        match CStr::from_ptr(filters).to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                _set_last_error("filters: invalid UTF-8 string".to_string());
                return ReturnCode::Error;
            }
        }
    };

    let result = (*provider)
        .runtime
        .block_on(async { (*provider).inner.least_busy(filters_opt).await });

    match result {
        Ok(Some(r)) => {
            *resource_out = Box::into_raw(Box::new(QuantumResource {
                inner: r,
                runtime: (*provider).runtime.clone(),
            }));
            ReturnCode::Success
        }
        Ok(None) => {
            *resource_out = std::ptr::null_mut();
            ReturnCode::Success
        }
        Err(err) => _fail(err),
    }
}

// ---------------------------------------------------------------------------
// QRMIService C bindings
// ---------------------------------------------------------------------------

/// @ingroup QrmiService
/// Discovers the QPU resources assigned to the current job -- read from the
/// `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES` environment variables, or
/// their legacy `SLURM_JOB_QPU_RESOURCES` / `SLURM_JOB_QPU_TYPES`
/// equivalents -- and returns the ones that are currently accessible.
///
/// This is the C counterpart of `qrmi.QRMIService` (Python) and
/// `qrmi::QRMIService` (Rust); all three share the same underlying
/// discovery/filtering logic.
///
/// Unlike qrmi_provider_resources(), which is called against a persistent
/// QrmiResourceProvider handle (and can be called repeatedly, e.g. with
/// different filters), this function takes no arguments and is a one-shot
/// operation: there is no separate "service" handle to create or free.
/// Discovery happens inline, and each QrmiQuantumResource handle placed in
/// `resources_out` is independently owned by the caller from that point on
/// -- usable with the same qrmi_resource_*() functions as a handle returned
/// by qrmi_resource_new(), just not individually freed (see
/// qrmi_service_resources_free() below).
///
/// The caller is responsible for freeing the returned struct with
/// qrmi_service_resources_free(). Individual handles inside the struct must
/// NOT be freed separately.
///
/// # Safety
///
/// * `resources_out` must be non-null and point to a zero-initialized
///   QrmiQuantumResources.
///
/// # Example
///
/// @code
///   QrmiQuantumResources resources = {0};
///   QrmiReturnCode rc = qrmi_service_resources(&resources);
///   if (rc == QRMI_RETURN_CODE_SUCCESS) {
///     for (size_t i = 0; i < resources.length; i++) {
///       char *id = NULL;
///       qrmi_resource_id(resources.resources[i], &id);
///       printf("resource: %s\n", id);
///       qrmi_string_free(id);
///     }
///     qrmi_service_resources_free(&resources);
///   } else {
///     const char *err = qrmi_get_last_error();
///     printf("error: %s\n", err);
///   }
/// @endcode
///
/// @param (resources_out) [out] Pointer to a QrmiQuantumResources struct to populate
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.23.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_service_resources(
    resources_out: *mut QuantumResources,
) -> ReturnCode {
    crate::common::initialize();
    null_pointer_check!(resources_out, ReturnCode::NullPointerError);

    // One `Runtime`, shared (via `Arc`) across every `QuantumResource`
    // handle this call produces -- same approach `qrmi_provider_resources`
    // takes for the handles it produces, rather than each handle getting
    // its own `Runtime` (contrast `PyQuantumResource` in `pyext.rs`, where
    // each Python-visible object needs to be independently droppable and
    // so does own its own).
    let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let result = runtime.block_on(async { crate::QRMIService::new().await });

    match result {
        Ok(service) => {
            let resource_map = service.into_resource_map();
            let count = resource_map.len();
            let mut raw_ptrs: Vec<*mut QuantumResource> = resource_map
                .into_values()
                .map(|r| {
                    Box::into_raw(Box::new(QuantumResource {
                        inner: r,
                        runtime: runtime.clone(),
                    }))
                })
                .collect();

            let ptr = raw_ptrs.as_mut_ptr();
            std::mem::forget(raw_ptrs);

            (*resources_out).resources = ptr;
            (*resources_out).length = count;
            ReturnCode::Success
        }
        Err(err) => _fail(err),
    }
}

/// @ingroup QrmiService
/// Frees a QrmiQuantumResources struct populated by qrmi_service_resources().
///
/// This frees both the individual QrmiQuantumResource handles and the
/// internal array. After calling this, the struct's fields are zeroed.
/// Do NOT call qrmi_resource_free() on individual elements after calling
/// this.
///
/// # Safety
///
/// * `resources` must have been populated by a previous call to
///   qrmi_service_resources().
///
/// # Example
///
/// @code
///   qrmi_service_resources_free(&resources);
/// @endcode
///
/// @param (resources) [in] Pointer to a QrmiQuantumResources struct to free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.23.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_service_resources_free(
    resources: *mut QuantumResources,
) -> ReturnCode {
    // Identical shape and freeing logic to `qrmi_provider_resources_free`
    // (same `QuantumResources` struct, same ownership rules) -- delegate to
    // it rather than duplicating the unsafe pointer-walking code. A
    // separate function (rather than just telling callers to reuse
    // `qrmi_provider_resources_free`) exists so the name at the call site
    // matches the `qrmi_service_*` family it was populated by, and so it
    // shows up under this file's `QrmiService` Doxygen group.
    unsafe { qrmi_provider_resources_free(resources) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResourceType;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Returns `output` from every string-producing call and counts task
    /// submissions.
    struct StubResource {
        output: String,
        starts: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl crate::QuantumResource for StubResource {
        async fn resource_id(&mut self) -> crate::Result<String> {
            Ok(self.output.clone())
        }
        async fn resource_type(&mut self) -> crate::Result<ResourceType> {
            Ok(ResourceType::PasqalCloud)
        }
        async fn is_accessible(&mut self) -> crate::Result<bool> {
            Ok(true)
        }
        async fn acquire(&mut self) -> crate::Result<String> {
            Ok(self.output.clone())
        }
        async fn task_start(&mut self, _payload: crate::models::Payload) -> crate::Result<String> {
            self.starts.fetch_add(1, Ordering::SeqCst);
            Ok(self.output.clone())
        }
        async fn task_result(
            &mut self,
            _task_id: &str,
        ) -> crate::Result<crate::models::TaskResult> {
            Ok(crate::models::TaskResult {
                value: self.output.clone(),
            })
        }
        async fn task_logs(&mut self, _task_id: &str) -> crate::Result<String> {
            Ok(self.output.clone())
        }
        async fn target(&mut self) -> crate::Result<crate::models::Target> {
            Ok(crate::models::Target {
                value: self.output.clone(),
            })
        }
    }

    /// Returns a stub and the number of `task_start` calls it receives.
    fn stub(output: &str) -> (QuantumResource, Arc<AtomicUsize>) {
        let starts = Arc::new(AtomicUsize::new(0));
        let qrmi = QuantumResource {
            inner: Box::new(StubResource {
                output: output.to_string(),
                starts: starts.clone(),
            }),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
        };
        (qrmi, starts)
    }

    /// Records an unrelated error, so tests can tell a fresh one from it.
    fn seed_error() {
        _fail(QrmiError::TaskNotFound("seed".to_string()));
    }

    unsafe fn take_last_error() -> Option<String> {
        let ptr = qrmi_get_last_error();
        if ptr.is_null() {
            return None;
        }
        let msg = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        qrmi_string_free(ptr);
        Some(msg)
    }

    #[test]
    fn last_error_is_cleared_after_read() {
        unsafe {
            let rc = qrmi_resource_task_start(
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
            );
            assert_eq!(rc, ReturnCode::NullPointerError);
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
            assert_eq!(take_last_error().as_deref(), Some("qrmi is NULL"));
            assert_eq!(take_last_error(), None);
        }
    }

    #[test]
    fn last_error_kind_is_not_cleared_after_read() {
        unsafe {
            seed_error();
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);
            take_last_error();
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);
        }
    }

    #[test]
    fn successful_call_keeps_unread_error() {
        unsafe {
            let (mut qrmi, _) = stub("stub");
            seed_error();
            let mut id = std::ptr::null_mut();
            assert_eq!(qrmi_resource_id(&mut qrmi, &mut id), ReturnCode::Success);
            qrmi_string_free(id);
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);
            assert_eq!(take_last_error().as_deref(), Some("task not found: seed"));
        }
    }

    #[test]
    fn last_error_is_per_thread() {
        seed_error();
        std::thread::spawn(|| unsafe {
            assert_eq!(take_last_error(), None);
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
            qrmi_resource_task_start(std::ptr::null_mut(), std::ptr::null(), std::ptr::null_mut());
            assert_eq!(take_last_error().as_deref(), Some("qrmi is NULL"));
        })
        .join()
        .unwrap();
        unsafe {
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);
            assert_eq!(take_last_error().as_deref(), Some("task not found: seed"));
        }
    }

    #[test]
    fn message_only_error_resets_kind() {
        unsafe {
            seed_error();
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);

            let (mut qrmi, _) = stub("stub");
            let rc = qrmi_resource_task_start(&mut qrmi, std::ptr::null(), std::ptr::null_mut());
            assert_eq!(rc, ReturnCode::NullPointerError);
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
            assert_eq!(take_last_error().as_deref(), Some("payload is NULL"));
        }
    }

    #[test]
    fn backend_error_keeps_its_kind_and_message() {
        unsafe {
            let (mut qrmi, _) = stub("stub");
            seed_error();
            let task = CString::new("t").unwrap();
            let rc = qrmi_resource_task_stop(&mut qrmi, task.as_ptr());
            assert_eq!(rc, ReturnCode::UnsupportedFunctionError);
            assert_eq!(
                qrmi_get_last_error_kind(),
                ReturnCode::UnsupportedFunctionError
            );
            assert_eq!(
                take_last_error().as_deref(),
                Some("unsupported function: qrmi::QuantumResource::task_stop")
            );
        }
    }

    #[test]
    fn resource_type_from_str_names_null_argument() {
        unsafe {
            let outp: *mut ResourceType = &mut ResourceType::PasqalCloud;
            let type_str = CString::new("pasqal-local").unwrap();
            let calls: [(&str, &dyn Fn() -> ReturnCode); 2] = [
                ("type_str", &|| {
                    qrmi_config_resource_type_from_str(std::ptr::null(), outp)
                }),
                ("outp", &|| {
                    qrmi_config_resource_type_from_str(type_str.as_ptr(), std::ptr::null_mut())
                }),
            ];
            for (name, call) in calls {
                seed_error();
                assert_eq!(call(), ReturnCode::NullPointerError);
                assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
                assert_eq!(take_last_error(), Some(format!("{name} is NULL")));
            }
        }
    }

    #[test]
    fn null_output_pointers_are_rejected() {
        unsafe {
            let (mut qrmi, _) = stub("stub");
            let config: *mut Config = &mut Config {
                resource_map: Default::default(),
            };
            let metadata: *mut ResourceMetadata = &mut ResourceMetadata {
                inner: Default::default(),
            };
            let q: *mut QuantumResource = &mut qrmi;
            let mut count = 0;
            let mut strings = std::ptr::null_mut();
            let (count_p, strings_p): (*mut usize, *mut *mut *mut c_char) =
                (&mut count, &mut strings);
            let calls: [(&str, &dyn Fn() -> ReturnCode); 4] = [
                ("outp", &|| qrmi_resource_target(q, std::ptr::null_mut())),
                ("num_names", &|| {
                    qrmi_config_resource_names_get(config, std::ptr::null_mut(), strings_p)
                }),
                ("num_keys", &|| {
                    qrmi_resource_metadata_keys(metadata, std::ptr::null_mut(), strings_p)
                }),
                ("key_names", &|| {
                    qrmi_resource_metadata_keys(metadata, count_p, std::ptr::null_mut())
                }),
            ];
            for (name, call) in calls {
                seed_error();
                assert_eq!(call(), ReturnCode::NullPointerError);
                assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
                assert_eq!(take_last_error(), Some(format!("{name} is NULL")));
            }
            assert!(strings.is_null());
        }
    }

    #[test]
    fn release_rejects_invalid_utf8_token() {
        unsafe {
            let (mut qrmi, _) = stub("stub");
            let token = CString::from_vec_unchecked(vec![0xff, 0xfe]);
            let rc = qrmi_resource_release(&mut qrmi, token.as_ptr());
            assert_eq!(rc, ReturnCode::Error);
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::InvalidInputError);
            assert_eq!(
                take_last_error().as_deref(),
                Some("acquisition_token is not a valid UTF-8 string")
            );

            let token = CString::new("token").unwrap();
            assert_eq!(
                qrmi_resource_release(&mut qrmi, token.as_ptr()),
                ReturnCode::Success
            );
        }
    }

    #[test]
    fn task_start_rejects_bad_payload_strings_before_submission() {
        unsafe {
            let valid = CString::new("x").unwrap();
            let invalid = CString::from_vec_unchecked(vec![0xff]);
            let ok = valid.as_ptr() as *mut c_char;
            for (bad, problem) in [
                (
                    invalid.as_ptr() as *mut c_char,
                    "is not a valid UTF-8 string",
                ),
                (std::ptr::null_mut(), "is NULL"),
            ] {
                let iqm = |iqmjson, job_type| Payload::IQMServer {
                    iqmjson,
                    job_type,
                    use_timeslot: 0,
                    tag: ok,
                };
                for (field, payload) in [
                    (
                        "input",
                        Payload::QiskitPrimitive {
                            input: bad,
                            program_id: ok,
                        },
                    ),
                    (
                        "program_id",
                        Payload::QiskitPrimitive {
                            input: ok,
                            program_id: bad,
                        },
                    ),
                    (
                        "sequence",
                        Payload::PasqalCloud {
                            sequence: bad,
                            job_runs: 1,
                        },
                    ),
                    (
                        "human_qir",
                        Payload::AliceBobFelis {
                            human_qir: bad,
                            input_params: ok,
                        },
                    ),
                    (
                        "input_params",
                        Payload::AliceBobFelis {
                            human_qir: ok,
                            input_params: bad,
                        },
                    ),
                    ("iqmjson", iqm(bad, ok)),
                    ("job_type", iqm(ok, bad)),
                    ("job_spec", Payload::Oqtopus { job_spec: bad }),
                ] {
                    let (mut qrmi, starts) = stub("task");
                    seed_error();
                    let mut task_id = std::ptr::null_mut();
                    let rc = qrmi_resource_task_start(&mut qrmi, &payload, &mut task_id);
                    assert_eq!(rc, ReturnCode::InvalidInputError, "{field}");
                    assert_eq!(qrmi_get_last_error_kind(), ReturnCode::InvalidInputError);
                    assert_eq!(take_last_error(), Some(format!("{field} {problem}")));
                    assert!(task_id.is_null());
                    assert_eq!(starts.load(Ordering::SeqCst), 0, "{field}");
                }
            }
        }
    }

    #[test]
    fn task_start_accepts_null_iqm_tag_and_rejects_invalid_one() {
        unsafe {
            let valid = CString::new("x").unwrap();
            let invalid = CString::from_vec_unchecked(vec![0xff]);
            let ok = valid.as_ptr() as *mut c_char;
            for (tag, expected) in [
                (std::ptr::null_mut(), ReturnCode::Success),
                (ok, ReturnCode::Success),
                (
                    invalid.as_ptr() as *mut c_char,
                    ReturnCode::InvalidInputError,
                ),
            ] {
                let (mut qrmi, starts) = stub("task");
                seed_error();
                let payload = Payload::IQMServer {
                    iqmjson: ok,
                    job_type: ok,
                    use_timeslot: 0,
                    tag,
                };
                let mut task_id = std::ptr::null_mut();
                let rc = qrmi_resource_task_start(&mut qrmi, &payload, &mut task_id);
                assert_eq!(rc, expected);
                if expected == ReturnCode::Success {
                    assert_eq!(starts.load(Ordering::SeqCst), 1);
                    qrmi_string_free(task_id);
                } else {
                    assert_eq!(starts.load(Ordering::SeqCst), 0);
                    assert!(task_id.is_null());
                    assert_eq!(
                        take_last_error().as_deref(),
                        Some("tag is not a valid UTF-8 string")
                    );
                }
            }
        }
    }

    #[test]
    fn output_with_nul_byte_records_fresh_generic_error() {
        unsafe {
            let (mut qrmi, starts) = stub("s3cr3t\0v4lue");
            let task = CString::new("t").unwrap();
            let sequence = CString::new("x").unwrap();
            let payload = Payload::PasqalCloud {
                sequence: sequence.as_ptr() as *mut c_char,
                job_runs: 1,
            };
            let q: *mut QuantumResource = &mut qrmi;
            let calls: [(&str, &dyn Fn(*mut *mut c_char) -> ReturnCode); 6] = [
                ("resource ID contains a NUL byte", &|out| {
                    qrmi_resource_id(q, out)
                }),
                (
                    "resource was acquired, but the acquisition token contains a NUL byte",
                    &|out| qrmi_resource_acquire(q, out),
                ),
                ("task was created, but its ID contains a NUL byte", &|out| {
                    qrmi_resource_task_start(q, &payload, out)
                }),
                ("task result contains a NUL byte", &|out| {
                    qrmi_resource_task_result(q, task.as_ptr(), out)
                }),
                ("task logs contain a NUL byte", &|out| {
                    qrmi_resource_task_logs(q, task.as_ptr(), out)
                }),
                ("target contains a NUL byte", &|out| {
                    qrmi_resource_target(q, out)
                }),
            ];
            for (message, call) in calls {
                seed_error();
                let mut out = std::ptr::null_mut();
                assert_eq!(call(&mut out), ReturnCode::Error, "{message}");
                assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
                let error = take_last_error().unwrap();
                assert_eq!(error, message);
                assert!(!error.contains("s3cr3t") && !error.contains("v4lue"));
                assert!(out.is_null());
            }
            assert_eq!(starts.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn null_free_keeps_last_error() {
        unsafe {
            seed_error();
            assert_eq!(
                qrmi_string_free(std::ptr::null_mut()),
                ReturnCode::NullPointerError
            );
            qrmi_string_array_free(0, std::ptr::null_mut());
            qrmi_config_free(std::ptr::null_mut());
            qrmi_config_resource_def_free(std::ptr::null_mut());
            qrmi_resource_free(std::ptr::null_mut());
            qrmi_resource_status_free(std::ptr::null_mut());
            qrmi_resource_capacity_free(std::ptr::null_mut());
            qrmi_resource_metadata_free(std::ptr::null_mut());
            qrmi_provider_free(std::ptr::null_mut());
            qrmi_provider_resources_free(std::ptr::null_mut());
            qrmi_service_resources_free(std::ptr::null_mut());
            assert_eq!(qrmi_get_last_error_kind(), ReturnCode::TaskNotFoundError);
            assert_eq!(take_last_error().as_deref(), Some("task not found: seed"));
        }
    }

    fn config_with(name: &str, key: &str, value: &str) -> Config {
        let def = crate::models::ResourceDef {
            name: name.to_string(),
            r#type: ResourceType::PasqalCloud,
            is_dynamic: false,
            environment: [(key.to_string(), value.to_string())].into(),
        };
        Config {
            resource_map: [("r".to_string(), def)].into(),
        }
    }

    #[test]
    fn resource_def_get_returns_definition() {
        unsafe {
            let mut config = config_with("r", "QRMI_KEY", "value");
            let id = CString::new("r").unwrap();
            let def = qrmi_config_resource_def_get(&mut config, id.as_ptr());
            let def_ref = def.as_ref().unwrap();
            assert_eq!(CStr::from_ptr(def_ref.name).to_str(), Ok("r"));
            assert_eq!(def_ref.environments.length, 1);
            let kv = def_ref.environments.variables.as_ref().unwrap();
            assert_eq!(CStr::from_ptr(kv.key).to_str(), Ok("QRMI_KEY"));
            assert_eq!(CStr::from_ptr(kv.value).to_str(), Ok("value"));
            assert_eq!(qrmi_config_resource_def_free(def), ReturnCode::Success);
        }
    }

    #[test]
    fn resource_def_get_rejects_nul_byte() {
        unsafe {
            let id = CString::new("r").unwrap();
            for (mut config, what) in [
                (config_with("secret\0name", "QRMI_KEY", "v"), "name"),
                (
                    config_with("r", "secret\0key", "v"),
                    "environment variable name",
                ),
                (
                    config_with("r", "QRMI_KEY", "secret\0value"),
                    "environment variable value",
                ),
            ] {
                seed_error();
                let def = qrmi_config_resource_def_get(&mut config, id.as_ptr());
                assert!(def.is_null());
                assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
                let error = take_last_error().unwrap();
                assert_eq!(error, format!("resource r: {what} contains a NUL byte"));
                assert!(!error.contains("secret"));
            }
        }
    }

    #[test]
    fn string_arrays_round_trip() {
        unsafe {
            let mut config = config_with("r", "k", "v");
            let mut metadata = ResourceMetadata {
                inner: [("backend".to_string(), "v".to_string())].into(),
            };
            let mut count = 0;
            let mut strings = std::ptr::null_mut();
            let rc = qrmi_config_resource_names_get(&mut config, &mut count, &mut strings);
            assert_eq!((rc, count), (ReturnCode::Success, 1));
            assert_eq!(CStr::from_ptr(*strings.as_ref().unwrap()).to_str(), Ok("r"));
            assert_eq!(qrmi_string_array_free(count, strings), ReturnCode::Success);

            let rc = qrmi_resource_metadata_keys(&mut metadata, &mut count, &mut strings);
            assert_eq!((rc, count), (ReturnCode::Success, 1));
            assert_eq!(
                CStr::from_ptr(*strings.as_ref().unwrap()).to_str(),
                Ok("backend")
            );
            assert_eq!(qrmi_string_array_free(count, strings), ReturnCode::Success);
        }
    }

    #[test]
    fn string_arrays_reject_nul_byte() {
        unsafe {
            let mut config = config_with("r", "k", "v");
            let def = config.resource_map["r"].clone();
            config.resource_map.insert("secret\0name".to_string(), def);
            let mut metadata = ResourceMetadata {
                inner: [
                    ("ok".to_string(), "v".to_string()),
                    ("secret\0key".to_string(), "v".to_string()),
                ]
                .into(),
            };
            let config: *mut Config = &mut config;
            let metadata: *mut ResourceMetadata = &mut metadata;
            type Call<'a> = &'a dyn Fn(*mut usize, *mut *mut *mut c_char) -> ReturnCode;
            let calls: [(&str, Call); 2] = [
                ("a resource name contains a NUL byte", &|count, strings| {
                    qrmi_config_resource_names_get(config, count, strings)
                }),
                ("a metadata key contains a NUL byte", &|count, strings| {
                    qrmi_resource_metadata_keys(metadata, count, strings)
                }),
            ];
            for (message, call) in calls {
                seed_error();
                // Non-zero, non-NULL outputs show that failure resets them.
                let mut count = 7;
                let mut strings = std::ptr::NonNull::dangling().as_ptr();
                assert_eq!(call(&mut count, &mut strings), ReturnCode::Error);
                assert_eq!((count, strings), (0, std::ptr::null_mut()));
                assert_eq!(qrmi_get_last_error_kind(), ReturnCode::Error);
                let error = take_last_error().unwrap();
                assert_eq!(error, message);
                assert!(!error.contains("secret"));
            }
        }
    }
}
