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

use crate::error::{required_env, QrmiError};
use crate::models::{ResourceType, TaskStatus, Target};
use crate::{QuantumResource, Result};
use async_trait::async_trait;
use libloading::Library;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use log::{info, warn};

fn pythonhome_lib_candidate(home: &str) -> Option<String> {
    let home = std::path::Path::new(home);
    for subdir in ["lib64", "lib"] {
        if let Some(found) = find_libpython_in(&home.join(subdir)) {
            return Some(found);
        }
    }
    None
}

fn find_libpython_in(lib_dir: &std::path::Path) -> Option<String> {
    let entries = std::fs::read_dir(lib_dir).ok()?;
    let mut best: Option<std::path::PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_str()?.to_string();
        if name.starts_with("libpython") && name.contains(".so") {
            let is_versioned = name.matches(".so.").count() > 0;
            if best.is_none() || is_versioned {
                best = Some(path);
            }
        }
    }
    best.map(|p| p.to_string_lossy().into_owned())
}

fn map_job_status(status: &str) -> Result<TaskStatus> {
    match status {
        "registered" | "submitted" | "ready" => Ok(TaskStatus::Queued),
        "running" => Ok(TaskStatus::Running),
        "succeeded" => Ok(TaskStatus::Completed),
        "failed" => Ok(TaskStatus::Failed),
        "cancelled" => Ok(TaskStatus::Cancelled),
        other => Err(QrmiError::Other(anyhow::anyhow!(
            "unknown job status from oqtopus_client: {other}"
        ))),
    }
}

#[cfg(target_os = "macos")]
fn default_bridge_path() -> String {
    "liboqtopus_py_bridge.dylib".to_string()
}

#[cfg(target_os = "windows")]
fn default_bridge_path() -> String {
    "oqtopus_py_bridge.dll".to_string()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn default_bridge_path() -> String {
    "liboqtopus_py_bridge.so".to_string()
}

/// QRMI implementation for OQTOPUS Cloud
pub struct Oqtopus {
    pub(crate) device_id: String,
    py_bridge: Library,
    config_json: CString,
}

impl Oqtopus {
    /// Constructs a OQTOPUS cloud instance.
    ///
    /// Environment variables used:
    /// * QRMI_OQTOPUS_BASE_URL - Oqtopus cloud base URL
    /// * QRMI_OQTOPUS_API_TOKEN - Oqtopus cloud API token
    pub fn new(device_id: &str) -> Result<Self> {
        let endpoint = required_env(format!("{device_id}_QRMI_OQTOPUS_BASE_URL"))?;
        let api_token = required_env(format!("{device_id}_QRMI_OQTOPUS_API_TOKEN"))?;

        use libloading::os::unix::{Library as UnixLibrary, RTLD_GLOBAL, RTLD_NOW};

        let mut loaded = false;

        if let Ok(explicit_path) = env::var("PYTHON_LIBRARY") {
            match unsafe { UnixLibrary::open(Some(&explicit_path), RTLD_GLOBAL | RTLD_NOW) } {
                Ok(lib) => {
                    info!("loaded {explicit_path} (via PYTHON_LIBRARY)");
                    std::mem::forget(lib);
                    loaded = true;
                }
                Err(e) => {
                    warn!("failed to load PYTHON_LIBRARY={explicit_path}, {e}");
                    return Err(QrmiError::Other(
                        anyhow::Error::new(e)
                            .context(format!("failed to load PYTHON_LIBRARY={explicit_path}")),
                    ));
                }
            }
        } else if let Ok(home) = env::var("PYTHONHOME") {
            match pythonhome_lib_candidate(&home) {
                Some(home_path) => {
                    match unsafe { UnixLibrary::open(Some(&home_path), RTLD_GLOBAL | RTLD_NOW) } {
                        Ok(lib) => {
                            info!("loaded {home_path} (via PYTHONHOME)");
                            std::mem::forget(lib);
                            loaded = true;
                        }
                        Err(e) => {
                            return Err(QrmiError::Other(
                                anyhow::Error::new(e)
                                    .context(format!("failed to load {home_path}")),
                            ));
                        }
                    }
                }
                None => {
                    return Err(QrmiError::Other(anyhow::anyhow!(
                        "PYTHONHOME={home} is set, but no libpython*.so* found under its lib/ or lib64/ directory"
                    )));
                }
            }
        }

        if !loaded {
            let candidates = [
                "libpython3.13.so.1.0",
                "libpython3.12.so.1.0",
                "libpython3.11.so.1.0",
                "libpython3.10.so.1.0",
                "libpython3.so",
            ];

            for name in candidates {
                if let Ok(lib) = unsafe { UnixLibrary::open(Some(name), RTLD_GLOBAL | RTLD_NOW) } {
                    info!("loaded {name} (via default search path)");
                    std::mem::forget(lib);
                    loaded = true;
                    break;
                }
            }
        }

        if !loaded {
            return Err(QrmiError::Other(anyhow::anyhow!(
                "libpython not found (set PYTHON_LIBRARY or PYTHONHOME to an explicit path)"
            )));
        }

        let bridge_path = env::var("PY_BRIDGE_PATH").unwrap_or_else(|_| default_bridge_path());

        let lib = unsafe {
            Library::new(&bridge_path).map_err(|e| {
                QrmiError::Other(anyhow::anyhow!("failed to load {bridge_path}: {e}"))
            })?
        };

        let config_json = serde_json::json!({
            "url": endpoint,
            "api_token": api_token,
        })
        .to_string();

        let config_json_c = CString::new(config_json)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid config json: {e}")))?;

        Ok(Self {
            device_id: device_id.to_string(),
            py_bridge: lib,
            config_json: config_json_c,
        })
    }

    fn free_string(&self, ptr: *mut c_char) {
        if ptr.is_null() {
            return;
        }
        if let Ok(free_func) = unsafe {
            self.py_bridge
                .get::<unsafe extern "C" fn(*mut c_char)>(b"py_bridge_free_string")
        } {
            unsafe { free_func(ptr) };
        }
    }

    fn take_error_string(&self, err_ptr: *mut c_char) -> String {
        if err_ptr.is_null() {
            return "unknown error".to_string();
        }
        let msg = unsafe { CStr::from_ptr(err_ptr) }
            .to_string_lossy()
            .into_owned();
        self.free_string(err_ptr);
        msg
    }
}

// Implement the QuantumResource trait using the asynchronous wrappers.
#[async_trait]
impl QuantumResource for Oqtopus {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::OQTOPUS)
    }

    /// Asynchronously checks if a backend is accessible.
    async fn is_accessible(&mut self) -> Result<bool> {
        let device_id_c = CString::new(self.device_id.clone())
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid device_id: {e}")))?;

        let func: libloading::Symbol<
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut *mut c_char,
                *mut *mut c_char,
            ) -> c_int,
        > = unsafe {
            self.py_bridge
                .get(b"get_device_status")
                .map_err(|e| QrmiError::Other(anyhow::anyhow!("symbol not found: {e}")))?
        };

        let mut status_ptr: *mut c_char = std::ptr::null_mut();
        let mut err_ptr: *mut c_char = std::ptr::null_mut();

        let ret = unsafe {
            func(
                device_id_c.as_ptr(),
                self.config_json.as_ptr(),
                &mut status_ptr,
                &mut err_ptr,
            )
        };

        if ret != 0 {
            let msg = self.take_error_string(err_ptr);
            return Err(QrmiError::Other(anyhow::anyhow!(
                "get_device_status failed: {msg}"
            )));
        }

        let status = unsafe { CStr::from_ptr(status_ptr) }
            .to_string_lossy()
            .into_owned();
        self.free_string(status_ptr);

        info!("device status: {status}");

        Ok(status == "available")
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let job_id_c = CString::new(task_id)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid job_id: {e}")))?;

        let func: libloading::Symbol<
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut *mut c_char,
                *mut *mut c_char,
            ) -> c_int,
        > = unsafe {
            self.py_bridge
                .get(b"cancel_job")
                .map_err(|e| QrmiError::Other(anyhow::anyhow!("symbol not found: {e}")))?
        };

        let mut message_ptr: *mut c_char = std::ptr::null_mut();
        let mut err_ptr: *mut c_char = std::ptr::null_mut();

        let ret = unsafe {
            func(
                job_id_c.as_ptr(),
                self.config_json.as_ptr(),
                &mut message_ptr,
                &mut err_ptr,
            )
        };

        if ret != 0 {
            let msg = self.take_error_string(err_ptr);
            return Err(QrmiError::Other(anyhow::anyhow!(
                "cancel_job failed: {msg}"
            )));
        }

        let message = unsafe { CStr::from_ptr(message_ptr) }
            .to_string_lossy()
            .into_owned();
        self.free_string(message_ptr);

        info!("cancel_job succeeded: {message}");

        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let job_id_c = CString::new(task_id)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid job_id: {e}")))?;

        let func: libloading::Symbol<
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut *mut c_char,
                *mut *mut c_char,
            ) -> c_int,
        > = unsafe {
            self.py_bridge
                .get(b"get_job_status")
                .map_err(|e| QrmiError::Other(anyhow::anyhow!("symbol not found: {e}")))?
        };

        let mut status_ptr: *mut c_char = std::ptr::null_mut();
        let mut err_ptr: *mut c_char = std::ptr::null_mut();

        let ret = unsafe {
            func(
                job_id_c.as_ptr(),
                self.config_json.as_ptr(),
                &mut status_ptr,
                &mut err_ptr,
            )
        };

        if ret != 0 {
            let msg = self.take_error_string(err_ptr);
            return Err(QrmiError::Other(anyhow::anyhow!(
                "get_job_status failed: {msg}"
            )));
        }

        let status = unsafe { CStr::from_ptr(status_ptr) }
            .to_string_lossy()
            .into_owned();
        self.free_string(status_ptr);

        info!("job status: {status}");

        map_job_status(&status)
    }

    async fn target(&mut self) -> Result<Target> {
        let device_id_c = CString::new(self.device_id.clone())
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid device_id: {e}")))?;

        let func: libloading::Symbol<
            unsafe extern "C" fn(
                *const c_char,
                *const c_char,
                *mut *mut c_char,
                *mut *mut c_char,
            ) -> c_int,
        > = unsafe {
            self.py_bridge
                .get(b"get_device_json")
                .map_err(|e| QrmiError::Other(anyhow::anyhow!("symbol not found: {e}")))?
        };

        let mut json_ptr: *mut c_char = std::ptr::null_mut();
        let mut err_ptr: *mut c_char = std::ptr::null_mut();

        let ret = unsafe {
            func(
                device_id_c.as_ptr(),
                self.config_json.as_ptr(),
                &mut json_ptr,
                &mut err_ptr,
            )
        };

        if ret != 0 {
            let msg = self.take_error_string(err_ptr);
            return Err(QrmiError::Other(anyhow::anyhow!(
                "get_device_json failed: {msg}"
            )));
        }

        let value = unsafe { CStr::from_ptr(json_ptr) }
            .to_string_lossy()
            .into_owned();
        self.free_string(json_ptr);

        Ok(Target { value })
    }
}
