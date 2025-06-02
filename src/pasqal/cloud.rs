// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{bail, Result};
use pasqal_cloud_api::{BatchStatus, Client, ClientBuilder, DeviceType};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

// c binding
use crate::consts::{QRMI_ERROR, QRMI_SUCCESS};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use async_trait::async_trait;

/// QRMI implementation for Pasqal Cloud
pub struct PasqalCloud {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

impl PasqalCloud {
    /// Constructs a QRMI to access Pasqal Cloud Service
    ///
    /// # Environment variables
    ///
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID`: Pasqal Cloud Project ID to access the QPU
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN`: Pasqal Cloud Auth Token
    ///
    /// Let's hardcode the rest for now
    pub fn new(backend_name: &str) -> Self {
        // Check to see if the environment variables required to run this program are set.
        let project_id =
            env::var(format!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID")).unwrap_or_else(|_| panic!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID"));
        let auth_token =
            env::var(format!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_TOKEN")).unwrap_or_else(|_| panic!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_TOKEN"));
        Self {
            api_client: ClientBuilder::new(auth_token, project_id).build().unwrap(),
            backend_name: backend_name.to_string(),
        }
    }
}

impl Default for PasqalCloud {
    fn default() -> Self {
        Self::new("")
    }
}
#[async_trait]
impl QuantumResource for PasqalCloud {
    async fn is_accessible(&mut self) -> bool {
        let fresnel = DeviceType::Fresnel.to_string();
        if self.backend_name != fresnel {
            let err = format!(
                "Device {} is invalid. Only {} device can receive jobs.",
                self.backend_name, fresnel,
            );
            panic!("{}", err);
        };
        match self.api_client.get_device(DeviceType::Fresnel).await {
            Ok(device) => device.data.status == "UP",
            Err(_err) => false,
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        // TBD on cloud side for POC
        // Pasqal Cloud does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // TBD on cloud side for POC
        // Pasqal Cloud does not support session concept, so simply ignores
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        if let Payload::PasqalCloud { sequence, job_runs } = payload {
            // TODO: Make configurable (get emulator from qrmi)
            match self
                .api_client
                .create_batch(sequence, job_runs, DeviceType::EmuFree)
                .await
            {
                Ok(batch) => Ok(batch.data.id),
                Err(err) => Err(err),
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload))
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        match self.api_client.cancel_batch(task_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        // TODO: Change for Fresnel after testing
        match self.api_client.get_batch(task_id).await {
            Ok(batch) => {
                let status = match batch.data.status {
                    BatchStatus::Pending => TaskStatus::Queued,
                    BatchStatus::Running => TaskStatus::Running,
                    BatchStatus::Done => TaskStatus::Completed,
                    BatchStatus::Canceled => TaskStatus::Cancelled,
                    BatchStatus::TimedOut => TaskStatus::Failed,
                    BatchStatus::Error => TaskStatus::Failed,
                    BatchStatus::Paused => TaskStatus::Queued,
                };
                return Ok(status);
            }
            Err(err) => Err(err),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        match self.api_client.get_batch_results(task_id).await {
            Ok(resp) => Ok(TaskResult { value: resp }),
            Err(_err) => Err(_err),
        }
    }

    async fn target(&mut self) -> Result<Target> {
        let fresnel = DeviceType::Fresnel.to_string();
        if self.backend_name != fresnel {
            let err = format!(
                "Device {} is invalid. Only {} device can receive jobs.",
                self.backend_name, fresnel
            );
            panic!("{}", err);
        };
        match self.api_client.get_device_specs(DeviceType::Fresnel).await {
            Ok(resp) => Ok(Target {
                value: resp.data.specs,
            }),
            Err(_err) => Err(_err),
        }
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let metadata: HashMap<String, String> = HashMap::new();
        metadata
    }
}

// The following code is for C API binding.

/// @brief Returns a PasqalCloud QRMI handle.
///
/// Created PasqalCloud instance needs to be removed by qrmi_pasqc_free() call if
/// no longer needed.
///
/// # Safety
///
/// @param (resource_id) [in] A resource identifier, i.e. backend name
/// @return a PasqalCloud QRMI handle if succeeded, otherwise NULL. Must call qrmi_pasqc_free() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_new(resource_id: *const c_char) -> *mut PasqalCloud {
    ffi_helpers::null_pointer_check!(resource_id, std::ptr::null_mut());

    if let Ok(id_str) = CStr::from_ptr(resource_id).to_str() {
        let qrmi = Box::new(PasqalCloud::new(id_str));
        return Box::into_raw(qrmi);
    }
    std::ptr::null_mut()
}

/// @brief Returns true if device is accessible, otherwise false.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @param (outp) [out] accessible or not
/// @return QRMI_SUCCESS(0) if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_is_accessible(
    qrmi: *mut PasqalCloud,
    outp: *mut bool,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(outp, QRMI_ERROR);

    let rt = tokio::runtime::Runtime::new().unwrap();
    *outp = rt.block_on(async {
        (*qrmi).is_accessible().await
    });
    QRMI_SUCCESS
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_pasqc_new(). Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, returns < 0.
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_pasqc_new().
///
/// @param (ptr) [in] A PasqalCloud QRMI handle
/// @return QRMI_SUCCESS(0) if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_free(ptr: *mut PasqalCloud) -> c_int {
    if ptr.is_null() {
        return QRMI_ERROR;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    QRMI_SUCCESS
}

/// @brief Acquires quantum resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @return Acquisition token if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_acquire(qrmi: *mut PasqalCloud) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        (*qrmi).acquire().await
    });
    match result {
        Ok(token) => {
            if let Ok(token_cstr) = CString::new(token) {
                return token_cstr.into_raw();
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
    std::ptr::null()
}

// The following code is for C API binding.

/// @brief Returns a Pasqal Cloud QRMI handle.
///
/// Created PasqalCloud instance needs to be removed by qrmi_pasqc_free() call if
/// no longer needed.
///
/// # Safety
///
/// @param (resource_id) [in] A resource identifier, i.e. backend name
/// @return a PasqalCloud QRMI handle if succeeded, otherwise NULL. Must call qrmi_pasqc_free() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_release(
    qrmi: *mut PasqalCloud,
    acquisition_token: *const c_char,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(acquisition_token, QRMI_ERROR);

    if let Ok(token) = CStr::from_ptr(acquisition_token).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).release(token).await
        });
        match result {
            Ok(()) => {
                return QRMI_SUCCESS;
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    QRMI_SUCCESS
}

/// @brief Starts a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// * The memory pointed to by `program_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * The memory pointed to by `input` must contain a valid nul terminator at the
///   end of the string.
///
/// * `program_id` and `input` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `program_id`
///
/// * The nul terminator must be within `isize::MAX` from `input`
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @param (program_id) [in] Program ID (`sampler` or `estimator`)
/// @param (input) [in] primitive input
/// @return A task identifier if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_task_start(
    qrmi: *mut PasqalCloud,
    input: *const c_char,
    job_runs: i32,
) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    ffi_helpers::null_pointer_check!(input, std::ptr::null());

    if let Ok(input_str) = CStr::from_ptr(input).to_str() {
        let payload = Payload::PasqalCloud {
            sequence: input_str.to_string(),
            job_runs,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_start(payload).await
        });
        match result {
            Ok(job_id) => {
                if let Ok(job_id_cstr) = CString::new(job_id) {
                    return job_id_cstr.into_raw();
                }
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    std::ptr::null()
}

/// @brief Stops a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `task_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `task_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @param (task_id) [in] A task ID, returned by a previous call to qrmi_pasqc_task_start()
/// @param (input) [in] Primitive input
/// @return QRMI_SUCCESS if succeeded, otherwise QRMI_ERROR.i
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_task_stop(
    qrmi: *mut PasqalCloud,
    task_id: *const c_char,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }

    ffi_helpers::null_pointer_check!(task_id, QRMI_ERROR);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_stop(task_id_str).await
        });
        match result {
            Ok(()) => {
                return QRMI_SUCCESS;
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    QRMI_ERROR
}

/// @brief Returns the status of the specified task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * The memory pointed to by `outp` must have enough room to store `TaskStatus` value.
///
/// * `task_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `task_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @param (task_id) [in] A task identifier
/// @return QRMI_SUCCESS if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_task_status(
    qrmi: *mut PasqalCloud,
    task_id: *const c_char,
    outp: *mut TaskStatus,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }

    ffi_helpers::null_pointer_check!(task_id, QRMI_ERROR);
    ffi_helpers::null_pointer_check!(outp, QRMI_ERROR);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_status(task_id_str).await
        });
        match result {
            Ok(v) => {
                *outp = v;
                return QRMI_SUCCESS;
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    QRMI_ERROR
}

/// @brief Returns the result of a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `task_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `task_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @param (task_id) [in] A task identifier
/// @return Task result if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_task_result(
    qrmi: *mut PasqalCloud,
    task_id: *const c_char,
) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    ffi_helpers::null_pointer_check!(task_id, std::ptr::null());

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_result(task_id_str).await
        });
        match result {
            Ok(v) => {
                if let Ok(result_cstr) = CString::new(v.value) {
                    return result_cstr.into_raw();
                }
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    std::ptr::null()
}

/// @brief Returns a Target for the specified device. Vendor specific serialized data. This might contain the constraints(instructions, properties and timing information etc.) of a particular device to allow compilers to compile an input circuit to something that works and is optimized for a device. In IBM implementation, it contains JSON representations of [BackendConfiguration](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json) and [BackendProperties](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json) so that we are able to create a Target object by calling `qiskit_ibm_runtime.utils.backend_converter.convert_to_target` or uquivalent functions.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_pasqc_new().
///
/// @param (qrmi) [in] A PasqalCloud QRMI handle
/// @return A serialized target data if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_pasqc_target(qrmi: *mut PasqalCloud) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        (*qrmi).target().await
    });
    match result {
        Ok(v) => {
            if let Ok(target_cstr) = CString::new(v.value) {
                return target_cstr.into_raw();
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
    std::ptr::null()
}
