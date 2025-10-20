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
#![allow(dead_code)]
use crate::ibm::{IBMDirectAccess, IBMQiskitRuntimeService};
use crate::models::{Config, ResourceType, TaskStatus};
use crate::pasqal::PasqalCloud;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

/// Integer return codes returned to C.
#[repr(C)]
pub enum ReturnCode {
    /// Success.
    Success = 0,
    /// Error.
    Error = 100,
    /// Unexpected null pointer.
    NullPointerError = 101,
}

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
    /// Payload for Pasqal Cloud
    PasqalCloud {
        /// Pulser sequence
        sequence: *mut c_char,
        /// Number of job runs
        job_runs: i32,
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
    /// environment variables for this resource
    environments: EnvironmentVariables,
}

/// Quantum resource metadata
#[derive(Debug)]
pub struct ResourceMetadata {
    inner: std::collections::HashMap<String, String>,
}

/// Quantum resource handle
pub struct QuantumResource {
    inner: Box<dyn crate::QuantumResource>,
    runtime: Arc<tokio::runtime::Runtime>,
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
///     char *target = NULL;
///     QrmiReturnCode rc;
///     rc = qrmi_resource_target(qrmi, &target);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///         printf("target = %s\n", target);
///         qrmi_string_free(target);
///     }
///
/// @param (ptr) [in] pointer to the memory to be free
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_string_free(ptr: *mut c_char) -> ReturnCode {
    crate::common::initialize();
    ffi_helpers::null_pointer_check!(ptr, ReturnCode::NullPointerError);
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
///       size_t num_names = 0;
///       char **names = NULL;
///       QrmiReturnCode rc = qrmi_config_resource_names_get(cnf, &num_names, &names);
///       if (rc == QRMI_RETURN_CODE_SUCCESS) {
///           for (int i = 0; i < num_names; i++) {
///               printf("[%s]\n", names[i]);
///           }
///           qrmi_string_array_free(num_names, names);
///       }
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
        let _ = Box::from_raw(array);
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
///     QrmiConfig *cnf = qrmi_config_load("/etc/slurm/qrmi_config.json");
///
/// @param (filename) [in] qrmi_config.json file path
/// @return A QrmiConfig if succeeded, otherwise NULL. Must call qrmi_config_free() to free if no longer used.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_load(filename: *const c_char) -> *mut Config {
    crate::common::initialize();
    ffi_helpers::null_pointer_check!(filename, std::ptr::null_mut());

    if let Ok(file) = CStr::from_ptr(filename).to_str() {
        let result = Box::new(Config::load(file));
        match *result {
            Ok(v) => {
                return Box::into_raw(Box::new(v));
            }
            Err(err) => {
                log::error!("{:?}", err);
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
///     QrmiConfig *cnf = qrmi_config_load("/etc/slurm/qrmi_config.json");
///     qrmi_config_free(cnf);
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
///     QrmiConfig *cnf = qrmi_config_load(argv[1]);
///     if (!cnf) {
///         QrmiResourceDef* res = qrmi_config_resource_def_get(cnf, "your_resource_id");
///         if (res != NULL) {
///             printf("%s %d\n", res->name, res->type);
///             QrmiEnvironmentVariables envvars = res->environments;
///             for (int j = 0; j < envvars.length; j++) {
///                 QrmiKeyValue envvar = envvars.variables[j];
///                 printf("%s = %s\n", envvar.key, envvar.value);
///             }
///         }
///     }
///
/// @param (config) [in] a Config handle
/// @param (resource_id) [in] resource identifier
/// @return A QrmiResourceDef if succeeded, otherwise NULL. Must call qrmi_config_resource_def_free() to free if no longer used.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_config_resource_def_get(
    config: *mut Config,
    resource_id: *const c_char,
) -> *mut ResourceDef {
    crate::common::initialize();
    if config.is_null() {
        return std::ptr::null_mut();
    }
    ffi_helpers::null_pointer_check!(resource_id, std::ptr::null_mut());

    if let Ok(id_str) = CStr::from_ptr(resource_id).to_str() {
        if let Some(resource) = (*config).resource_map.get(id_str) {
            let mut c_envvars = Vec::new();
            for (key, value) in resource.environment.clone().into_iter() {
                c_envvars.push(KeyValue {
                    key: CString::new(key.clone()).unwrap().into_raw(),
                    value: CString::new(value.clone()).unwrap().into_raw(),
                });
            }
            let boxed_res = Box::new(ResourceDef {
                name: CString::new(resource.name.clone()).unwrap().into_raw(),
                r#type: resource.r#type.clone(),
                environments: EnvironmentVariables {
                    variables: c_envvars.as_mut_ptr(),
                    length: c_envvars.len(),
                },
            });

            std::mem::forget(c_envvars);
            return Box::into_raw(boxed_res);
        }
    }
    std::ptr::null_mut()
}

/// @ingroup QrmiConfig
/// Converts ResourceType to string representation used in qrmi_config.json, e.g.
/// @ref QrmiResourceType::QRMI_RESOURCE_TYPE_QISKIT_RUNTIME_SERVICE to `qiskit-runtime-service`.
///
/// # Safety
///
/// * `type` must be QrmiResourceType value.
///
/// # Example
///
///      char *type_as_str = qrmi_config_resource_type_to_str(QRMI_RESOURCE_TYPE_QISKIT_RUNTIME_SERVICE):
///      printf("%s\n", type_as_str);
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
/// Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_config_get_resource_def() or related functions. Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_config_resource_def_get().
///
/// # Example
///
///     QrmiConfig *cnf = qrmi_config_load(argv[1]);
///     if (!cnf) {
///         QrmiResourceDef* res = qrmi_config_resource_def_get(cnf, "your_resource_id");
///         if (res != NULL) {
///             printf("%s %d\n", res->name, res->type);
///         }
///         qrmi_config_resource_def_free(res);
///     }
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
/// * `names` must be non nul.
///
/// # Example
///
///      size_t num_names = 0;
///      char **names = NULL;
///      QrmiReturnCode rc = qrmi_config_resource_names_get(cnf, &num_names, &names);
///      if (rc == QRMI_RETURN_CODE_SUCCESS) {
///          for (int i = 0; i < num_names; i++) {
///              printf("[%s]\n", names[i]);
///          }
///          qrmi_string_array_free(num_names, names);
///      }
///
/// @param (config) [in] A Config handle
/// @param (num_names) [out] number of resource names in the list
/// @param (names) [out] A list of the resource names if succeeded. Must call qrmi_string_array_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// @version 0.6.0
#[no_mangle]
/// cbindgen:ptrs-as-arrays=[[names;]]
pub unsafe extern "C" fn qrmi_config_resource_names_get(
    config: *mut Config,
    num_names: *mut usize,
    names: *mut *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    if config.is_null() || names.is_null() {
        return ReturnCode::NullPointerError;
    }

    let keys = (*config).resource_map.keys();
    let count = keys.len();
    let mut raw_ptrs: Vec<*mut c_char> = Vec::with_capacity(count);
    for key in keys {
        let str_c = CString::new(key.as_str()).unwrap();
        raw_ptrs.push(str_c.into_raw());
    }

    let boxed_array = raw_ptrs.into_boxed_slice();
    let raw = boxed_array.as_ptr() as *mut *mut c_char;
    std::mem::forget(boxed_array);

    unsafe {
        *num_names = count;
        *names = raw;
    }
    ReturnCode::Success
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
///     QrmiQuantumResource *qrmi = qrmi_resource_new("your_resource_name",
///                                                   QRMI_RESOURCE_TYPE_IBM_DIRECT_ACCESS);
///
/// @param (resource_id) [in] A resource identifier, i.e. backend name
/// @param (resource_type) [in] QrmiResourceType variant
/// @return a QrmiQuantumResource handle if succeeded, otherwise NULL. Must call qrmi_resource_free() to free if no longer used.
/// @version 0.6.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_new(
    resource_id: *const c_char,
    resource_type: ResourceType,
    error: *mut *mut c_char,
) -> *mut QuantumResource {
    crate::common::initialize();
    ffi_helpers::null_pointer_check!(resource_id, std::ptr::null_mut());

    if let Ok(id_str) = CStr::from_ptr(resource_id).to_str() {
        let res: Box<dyn crate::QuantumResource> = match resource_type {
            ResourceType::IBMDirectAccess => match IBMDirectAccess::new(id_str) {
                Ok(v) => Box::new(v),
                Err(err) => {
                    log::error!("{:?}", err);
                    if !error.is_null() {
                        if let Ok(error_cstr) = CString::new(format!("{}", err)) {
                            unsafe {
                                *error = error_cstr.into_raw();
                            }
                        }
                    }
                    return std::ptr::null_mut();
                }
            },
            ResourceType::QiskitRuntimeService => match IBMQiskitRuntimeService::new(id_str) {
                Ok(v) => Box::new(v),
                Err(err) => {
                    log::error!("{:?}", err);
                    if !error.is_null() {
                        if let Ok(error_cstr) = CString::new(format!("{}", err)) {
                            unsafe {
                                *error = error_cstr.into_raw();
                            }
                        }
                    }
                    return std::ptr::null_mut();
                }
            },
            ResourceType::PasqalCloud => match PasqalCloud::new(id_str) {
                Ok(v) => Box::new(v),
                Err(err) => {
                    log::error!("{:?}", err);
                    if !error.is_null() {
                        if let Ok(error_cstr) = CString::new(format!("{}", err)) {
                            unsafe {
                                *error = error_cstr.into_raw();
                            }
                        }
                    }
                    return std::ptr::null_mut();
                }
            },
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
///     QrmiQuantumResource *qrmi = qrmi_resource_new("your_resource_name",
///                                                   QRMI_RESOURCE_TYPE_IBM_DIRECT_ACCESS);
///     if (qrmi != NULL) {
///         qrmi_resource_free(qrmi);
///     }
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
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// # Example
///
///     bool is_accessible = false;
///     int rc = qrmi_resource_is_accessible(qrmi, &is_accessible);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///        if (is_accessible == false) {
///            printf("%s cannot be accessed.\n", argv[1]);
///        }
///     } else {
///        printf("qrmi_resource_is_accessible() failed.\n");
///     }
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
    if qrmi.is_null() {
        return ReturnCode::NullPointerError;
    }
    ffi_helpers::null_pointer_check!(outp, ReturnCode::Error);

    *outp = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.is_accessible().await });
    ReturnCode::Success
}

/// @ingroup QrmiQuantumResource
/// Acquires quantum resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non nul.
///
/// # Example
///
///     char *acquisition_token;
///     QrmiReturnCode rc = qrmi_resource_acquire(qrmi, &acquisition_token);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///         printf("acquisition token = %s\n", acquisition_token);
///     }
///     else {
///         printf("qrmi_resource_acquire failed.");
///     }
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
    if qrmi.is_null() || acquisition_token.is_null() {
        return ReturnCode::NullPointerError;
    }

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.acquire().await });
    match result {
        Ok(token) => {
            if let Ok(token_cstr) = CString::new(token) {
                unsafe {
                    *acquisition_token = token_cstr.into_raw();
                }
                return ReturnCode::Success;
            }
        }
        Err(err) => {
            log::error!("{:?}", err);
        }
    }
    ReturnCode::Error
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
///     char *acquisition_token = NULL;
///     QrmiReturnCode rc = qrmi_resource_acquire(qrmi, &acquisition_token);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///         rc = qrmi_resource_release(qrmi, acquisition_token);
///         if (rc != QRMI_RETURN_CODE_SUCCESS) {
///             printf("Failed to release a quantum resource\n");
///         }
///     }
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
    if qrmi.is_null() {
        return ReturnCode::NullPointerError;
    }
    ffi_helpers::null_pointer_check!(acquisition_token, ReturnCode::Error);

    if let Ok(token) = CStr::from_ptr(acquisition_token).to_str() {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.release(token).await });
        match result {
            Ok(()) => {
                return ReturnCode::Success;
            }
            Err(err) => {
                log::error!("{:?}", err);
            }
        }
    }
    ReturnCode::Success
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
/// # Example
///
///     QrmiPayload payload;
///     char *job_id = NULL;
///     QrmiReturnCode rc;
///
///     payload.tag = QRMI_PAYLOAD_QISKIT_PRIMITIVE;
///     payload.QISKIT_PRIMITIVE.input = (char *)input;
///     payload.QISKIT_PRIMITIVE.program_id = "estimator";
///
///     rc = qrmi_resource_task_start(qrmi, &payload, &job_id);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///         printf("Job ID: %s\n", job_id);
///     }
///     else {
///         printf("failed to start a task.\n");
///     }
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
    if qrmi.is_null() || task_id.is_null() {
        return ReturnCode::NullPointerError;
    }

    let mut qrmi_payload: Option<crate::models::Payload> = None;
    if let Payload::QiskitPrimitive { input, program_id } = *payload {
        if let (Ok(program_id_str), Ok(input_str)) = (
            CStr::from_ptr(program_id).to_str(),
            CStr::from_ptr(input).to_str(),
        ) {
            qrmi_payload = Some(crate::models::Payload::QiskitPrimitive {
                input: input_str.to_string(),
                program_id: program_id_str.to_string(),
            });
        }
    } else if let Payload::PasqalCloud { sequence, job_runs } = *payload {
        if let Ok(sequence_str) = CStr::from_ptr(sequence).to_str() {
            qrmi_payload = Some(crate::models::Payload::PasqalCloud {
                sequence: sequence_str.to_string(),
                job_runs,
            });
        }
    }

    if qrmi_payload.is_some() {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_start(qrmi_payload.unwrap()).await });
        match result {
            Ok(job_id) => {
                if let Ok(job_id_cstr) = CString::new(job_id) {
                    unsafe {
                        *task_id = job_id_cstr.into_raw();
                    }
                    return ReturnCode::Success;
                }
            }
            Err(err) => {
                log::error!("{:?}", err);
            }
        }
    }
    ReturnCode::Error
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
///     QrmiReturnCode rc = qrmi_resource_task_stop(qrmi, job_id);
///     if (rc != QRMI_RETURN_CODE_SUCCESS) {
///         printf("Failed to stop a task\n");
///     }
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
    if qrmi.is_null() {
        return ReturnCode::NullPointerError;
    }

    ffi_helpers::null_pointer_check!(task_id, ReturnCode::Error);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_stop(task_id_str).await });
        match result {
            Ok(()) => {
                return ReturnCode::Success;
            }
            Err(err) => {
                log::error!("{:?}", err);
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
///     QrmiTaskStatus status;
///     while (1) {
///         rc = qrmi_resource_task_status(qrmi, job_id, &status);
///         if (rc != QRMI_RETURN_CODE_SUCCESS || status != QRMI_TASK_STATUS_RUNNING) {
///             break;
///         }
///         sleep(1);
///     }
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
    if qrmi.is_null() {
        return ReturnCode::NullPointerError;
    }

    ffi_helpers::null_pointer_check!(task_id, ReturnCode::Error);
    ffi_helpers::null_pointer_check!(status, ReturnCode::Error);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_status(task_id_str).await });
        match result {
            Ok(v) => {
                *status = v;
                return ReturnCode::Success;
            }
            Err(err) => {
                log::error!("{:?}", err);
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
/// * `outp` must be non nul.
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// # Example
///
///     QrmiReturnCode rc = qrmi_resource_task_status(qrmi, job_id, &status);
///     if (rc == QRMI_RETURN_CODE_SUCCESS && status == QRMI_TASK_STATUS_COMPLETED) {
///         char *result = NULL;
///         qrmi_resource_task_result(qrmi, job_id, &result);
///         printf("%s\n", result);
///         qrmi_string_free((char *)result);
///     }
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
    if qrmi.is_null() {
        return ReturnCode::NullPointerError;
    }

    ffi_helpers::null_pointer_check!(task_id, ReturnCode::Error);
    ffi_helpers::null_pointer_check!(outp, ReturnCode::Error);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let result = (*qrmi)
            .runtime
            .block_on(async { (*qrmi).inner.task_result(task_id_str).await });
        match result {
            Ok(v) => {
                if let Ok(result_cstr) = CString::new(v.value) {
                    unsafe {
                        *outp = result_cstr.into_raw();
                    }
                    return ReturnCode::Success;
                }
            }
            Err(err) => {
                log::error!("{:?}", err);
            }
        }
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
/// * `outp` must be non nul.
///
/// # Example
///
///     char *target = NULL;
///     QrmiReturnCode rc;
///     rc = qrmi_resource_target(qrmi, &target);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///         printf("target = %s\n", target);
///         qrmi_string_free(target);
///     }
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
    if qrmi.is_null() {
        return ReturnCode::Error;
    }

    let result = (*qrmi)
        .runtime
        .block_on(async { (*qrmi).inner.target().await });
    match result {
        Ok(v) => {
            if let Ok(target_cstr) = CString::new(v.value) {
                unsafe {
                    *outp = target_cstr.into_raw();
                }
                return ReturnCode::Success;
            }
        }
        Err(err) => {
            log::error!("{:?}", err);
        }
    }
    ReturnCode::Error
}

/// @ingroup QrmiQuantumResource
/// Returns a resource metadata
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_resource_new().
///
/// * `outp` must be non nul.
///
/// # Example
///
///     QrmiResourceMetadata *metadata = NULL;
///     QrmiReturnCode rc = qrmi_resource_metadata(qrmi, &metadata);
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
    if qrmi.is_null() || outp.is_null() {
        return ReturnCode::NullPointerError;
    }

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
///     QrmiResourceMetadata *metadata = NULL;
///     QrmiReturnCode rc = qrmi_resource_metadata(qrmi, &metadata);
///     if (retval == QRMI_RETURN_CODE_SUCCESS) {
///         qrmi_resource_metadata_free(metadata);
///     }
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
///     char *value = qrmi_resource_metadata_value(metadata, "backend_name");
///     printf("metadata value=[%s]\n", value);
///     qrmi_string_free(value);
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
    if metadata.is_null() {
        return std::ptr::null_mut();
    }
    ffi_helpers::null_pointer_check!(key, std::ptr::null_mut());

    if let Ok(key_str) = CStr::from_ptr(key).to_str() {
        if let Some(val) = (*metadata).inner.get(key_str) {
            if let Ok(value_cstr) = CString::new(val.as_str()) {
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
/// * `num_keys` and `key_names` must be non nul.
///
/// # Example
///
///     size_t num_keys = 0;
///     char **metadata_keys = NULL;
///     QrmiReturnCode rc = qrmi_resource_metadata_keys(metadata, &num_keys, &metadata_keys);
///     if (rc == QRMI_RETURN_CODE_SUCCESS) {
///         for (int i = 0; i < num_keys; i++) {
///             printf("%s\n", metadata_keys[i]);
///         }
///         qrmi_string_array_free(num_keys, metadata_keys);
///     }
///
/// @param (metadata) [in] A QrmiResourceMetadata handle
/// @param (num_keys) [out] number of keys available in the metadata
/// @param (key_names) [out] A list of metadata key names if succeeded. Must call qrmi_string_array_free() to free if no longer used.
/// @return @ref QrmiReturnCode::QRMI_RETURN_CODE_SUCCESS if succeeded.
/// cbindgen:ptrs-as-arrays=[[key_names;]]
#[no_mangle]
pub unsafe extern "C" fn qrmi_resource_metadata_keys(
    metadata: *mut ResourceMetadata,
    num_keys: *mut usize,
    key_names: *mut *mut *mut c_char,
) -> ReturnCode {
    crate::common::initialize();
    if metadata.is_null() {
        return ReturnCode::NullPointerError;
    }

    let keys = (*metadata).inner.keys();
    let count = keys.len();
    let mut raw_ptrs: Vec<*mut c_char> = Vec::with_capacity(count);
    for key in keys {
        let str_c = CString::new(key.as_str()).unwrap();
        raw_ptrs.push(str_c.into_raw());
    }

    let boxed_array = raw_ptrs.into_boxed_slice();
    let raw = boxed_array.as_ptr() as *mut *mut c_char;
    std::mem::forget(boxed_array);

    unsafe {
        *num_keys = count;
        *key_names = raw;
    }
    ReturnCode::Success
}
