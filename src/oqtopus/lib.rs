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

//! Unix-only (Linux/macOS): loads libpython via `dlopen`
//! (`libloading::os::unix`), which has no Windows equivalent. Windows
//! is intentionally unsupported here -- QRMI targets HPC environments,
//! which are Linux-only in practice, so this module should be gated
//! behind `#[cfg(unix)]` at its `mod oqtopus;` declaration in `src/lib.rs`.

use crate::error::QrmiError;
use crate::models::{
    Payload, ResourceStatus, ResourceStatusCode, ResourceType, Target, TaskResult, TaskStatus,
};
use crate::oqtopus::models::{OqtopusDeviceInfo, OqtopusDeviceStatus};
use crate::{QuantumResource, Result};
use async_trait::async_trait;
use libloading::Library;
use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

use log::{info, warn};

#[cfg(not(target_os = "macos"))]
fn pythonhome_lib_candidate(home: &str) -> Option<String> {
    let home = std::path::Path::new(home);
    for subdir in ["lib64", "lib"] {
        if let Some(found) = find_libpython_in(&home.join(subdir)) {
            return Some(found);
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
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

/// macOS equivalent of the above. Two install layouts are common:
/// - Non-framework builds (Homebrew, `pyenv install --enable-shared`,
///   etc.): `$PYTHONHOME/lib/libpython3.x.dylib`.
/// - Framework builds (python.org installers, `pyenv install
///   --enable-framework`): `PYTHONHOME` points at a `.../Versions/3.x`
///   directory, and the shared library sits right there named plainly
///   `Python` (no `lib` prefix, no `.dylib` extension).
#[cfg(target_os = "macos")]
fn pythonhome_lib_candidate(home: &str) -> Option<String> {
    let home = std::path::Path::new(home);

    if let Some(found) = find_libpython_in(&home.join("lib")) {
        return Some(found);
    }

    let framework_binary = home.join("Python");
    if framework_binary.is_file() {
        return Some(framework_binary.to_string_lossy().into_owned());
    }

    None
}

#[cfg(target_os = "macos")]
fn find_libpython_in(lib_dir: &std::path::Path) -> Option<String> {
    let entries = std::fs::read_dir(lib_dir).ok()?;
    let mut best: Option<std::path::PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_str()?.to_string();
        if name.starts_with("libpython") && name.ends_with(".dylib") {
            // Prefer a version-specific name (libpython3.12.dylib) over
            // a bare libpython3.dylib, if both exist.
            let is_versioned = name
                .strip_prefix("libpython")
                .and_then(|rest| rest.chars().next())
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false);
            if best.is_none() || is_versioned {
                best = Some(path);
            }
        }
    }
    best.map(|p| p.to_string_lossy().into_owned())
}

/// Sonames tried, in order, when neither `PYTHON_LIBRARY` nor
/// `PYTHONHOME` is set -- via the dynamic linker's normal search path
/// (`LD_LIBRARY_PATH`, `ld.so.cache`/Mach-O default paths, etc.).
#[cfg(target_os = "macos")]
fn fallback_soname_candidates() -> &'static [&'static str] {
    &[
        "libpython3.13.dylib",
        "libpython3.12.dylib",
        "libpython3.11.dylib",
        "libpython3.10.dylib",
        "libpython3.dylib",
    ]
}

#[cfg(not(target_os = "macos"))]
fn fallback_soname_candidates() -> &'static [&'static str] {
    &[
        "libpython3.13.so.1.0",
        "libpython3.12.so.1.0",
        "libpython3.11.so.1.0",
        "libpython3.10.so.1.0",
        "libpython3.so",
    ]
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

#[cfg(all(unix, not(target_os = "macos")))]
fn default_bridge_path() -> String {
    "liboqtopus_py_bridge.so".to_string()
}

/// C ABI signature shared by every `dependencies/oqtopus` py_bridge
/// entry point used here: one input string (plus `config_json`), one
/// output string, one error-message out-param.
type BridgeFn =
    unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_char, *mut *mut c_char) -> c_int;

fn parse_value<T: std::str::FromStr>(raw: Option<String>, key_name: &str) -> Result<Option<T>>
where
    T::Err: std::fmt::Display,
{
    match raw {
        None => Ok(None),
        Some(raw) => raw
            .trim()
            .parse::<T>()
            .map(Some)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid value for {key_name}: {e}"))),
    }
}

fn parse_value_list(raw: Option<String>) -> Option<Vec<String>> {
    raw.map(|raw| {
        raw.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    })
}

fn build_config_json(
    device_id: &str,
    config_params: Option<&HashMap<String, String>>,
) -> Result<CString> {
    let key_name = |suffix: &str| -> String {
        match config_params {
            Some(_) => format!("QRMI_OQTOPUS_{suffix}"),
            None => format!("{device_id}_QRMI_OQTOPUS_{suffix}"),
        }
    };

    let lookup = |suffix: &str| -> Option<String> {
        match config_params {
            Some(params) => params.get(&key_name(suffix)).cloned(),
            None => std::env::var(key_name(suffix)).ok(),
        }
    };

    let mut config = serde_json::Map::new();

    if let Some(v) = lookup("BASE_URL") {
        config.insert("url".into(), serde_json::Value::String(v));
    }
    if let Some(v) = lookup("API_TOKEN") {
        config.insert("api_token".into(), serde_json::Value::String(v));
    }
    if let Some(v) = lookup("PROXY") {
        config.insert("proxy".into(), serde_json::Value::String(v));
    }

    if let Some(v) = parse_value::<f64>(lookup("TIMEOUT"), &key_name("TIMEOUT"))? {
        config.insert("timeout".into(), serde_json::json!(v));
    }
    if let Some(v) = parse_value::<u32>(
        lookup("RETRY_MAX_ATTEMPTS"),
        &key_name("RETRY_MAX_ATTEMPTS"),
    )? {
        config.insert("retry_max_attempts".into(), serde_json::json!(v));
    }
    if let Some(v) = parse_value::<f64>(
        lookup("RETRY_BACKOFF_SECONDS"),
        &key_name("RETRY_BACKOFF_SECONDS"),
    )? {
        config.insert("retry_backoff_seconds".into(), serde_json::json!(v));
    }

    if let Some(items) = parse_value_list(lookup("RETRY_STATUS_CODES")) {
        let codes: Vec<i64> = items
            .iter()
            .map(|s| {
                s.parse::<i64>().map_err(|e| {
                    QrmiError::Other(anyhow::anyhow!(
                        "invalid value in {}: {e}",
                        key_name("RETRY_STATUS_CODES")
                    ))
                })
            })
            .collect::<Result<_>>()?;
        config.insert("retry_status_codes".into(), serde_json::json!(codes));
    }
    if let Some(items) = parse_value_list(lookup("RETRY_METHODS")) {
        let methods: Vec<String> = items.into_iter().map(|s| s.to_uppercase()).collect();
        config.insert("retry_methods".into(), serde_json::json!(methods));
    }

    let config_json = serde_json::Value::Object(config).to_string();
    CString::new(config_json)
        .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid config json: {e}")))
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
    /// * {device_id}_QRMI_OQTOPUS_URL: OQTOPUS API URL.
    /// * {device_id}_QRMI_OQTOPUS_API_TOKEN: API token string.
    /// * {device_id}_QRMI_OQTOPUS_PROXY: Proxy URL.
    /// * {device_id}_QRMI_OQTOPUS_TIMEOUT: HTTP request timeout seconds. (e.g. 30.0)
    /// * {device_id}_QRMI_OQTOPUS_RETRY_MAX_ATTEMPTS: Max retry attempts for retryable requests. (e.g. 3)
    /// * {device_id}_QRMI_OQTOPUS_RETRY_BACKOFF_SECONDS: Exponential backoff base seconds.(e.g. 0.2)
    /// * {device_id}_QRMI_OQTOPUS_RETRY_STATUS_CODES: HTTP status codes treated as retryable. (e.g. 429,500,502,503)
    /// * {device_id}_QRMI_OQTOPUS_RETRY_METHODS: HTTP methods treated as retryable.(e.g. GET,POST)
    pub fn new(device_id: &str) -> Result<Self> {
        Self::from_opt(device_id, None)
    }

    pub fn from_config(device_id: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_opt(device_id, Some(&config))
    }

    fn from_opt(device_id: &str, config: Option<&HashMap<String, String>>) -> Result<Self> {
        let config_json_c = build_config_json(device_id, config)?;

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
                        "PYTHONHOME={home} is set, but no libpython was found under it"
                    )));
                }
            }
        }

        if !loaded {
            for name in fallback_soname_candidates() {
                if let Ok(lib) = unsafe { UnixLibrary::open(Some(*name), RTLD_GLOBAL | RTLD_NOW) } {
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

    /// Calls a py_bridge C entry point that takes one argument string
    /// plus `self.config_json`, and returns one output string on
    /// success. Centralizes symbol lookup, the call itself, error
    /// handling, and freeing the output/error strings -- the shape
    /// shared by every py_bridge function used here (`get_device_status`,
    /// `cancel_job`, `get_job_status`, `get_device_json`,
    /// `get_job_result_json`, `submit_job`).
    ///
    /// `op_name` is used only to prefix error messages (e.g.
    /// `"get_device_status failed: ..."`).
    fn call_bridge(&self, symbol: &[u8], arg: &CStr, op_name: &str) -> Result<String> {
        let func: libloading::Symbol<BridgeFn> = unsafe {
            self.py_bridge
                .get(symbol)
                .map_err(|e| QrmiError::Other(anyhow::anyhow!("symbol not found: {e}")))?
        };

        let mut out_ptr: *mut c_char = std::ptr::null_mut();
        let mut err_ptr: *mut c_char = std::ptr::null_mut();

        let ret = unsafe {
            func(
                arg.as_ptr(),
                self.config_json.as_ptr(),
                &mut out_ptr,
                &mut err_ptr,
            )
        };

        if ret != 0 {
            let msg = self.take_error_string(err_ptr);
            return Err(QrmiError::Other(anyhow::anyhow!("{op_name} failed: {msg}")));
        }

        let value = unsafe { CStr::from_ptr(out_ptr) }
            .to_string_lossy()
            .into_owned();
        self.free_string(out_ptr);
        Ok(value)
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

        let status = self.call_bridge(b"get_device_status", &device_id_c, "get_device_status")?;

        info!("device status: {status}");

        Ok(status == "available")
    }

    async fn status(&mut self) -> Result<ResourceStatus> {
        let device_id_c = CString::new(self.device_id.clone())
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid device_id: {e}")))?;

        let value = self.call_bridge(b"get_device_json", &device_id_c, "get_device_json")?;
        let device: OqtopusDeviceInfo = serde_json::from_str(&value)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid device json: {e}")))?;

        let status_reason = match (&device.available_at, &device.calibrated_at) {
            (Some(available_at), Some(calibrated_at)) => Some(format!(
                "available at {available_at}, calibrated at {calibrated_at}"
            )),
            (Some(available_at), None) => Some(format!("available at {available_at}")),
            (None, Some(calibrated_at)) => Some(format!("calibrated at {calibrated_at}")),
            (None, None) => None,
        };

        Ok(ResourceStatus {
            status: match device.status {
                OqtopusDeviceStatus::Available => ResourceStatusCode::Online,
                OqtopusDeviceStatus::Unavailable => ResourceStatusCode::Offline,
            },
            status_reason,
            healthy: None,
            busy: None,
            capacity: None,
            pending_job_count: Some(device.n_pending_jobs),
        })
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let job_id_c = CString::new(task_id)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid job_id: {e}")))?;

        let message = self.call_bridge(b"cancel_job", &job_id_c, "cancel_job")?;

        info!("cancel_job succeeded: {message}");

        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let job_id_c = CString::new(task_id)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid job_id: {e}")))?;

        let status = self.call_bridge(b"get_job_status", &job_id_c, "get_job_status")?;

        info!("job status: {status}");

        map_job_status(&status)
    }

    async fn target(&mut self) -> Result<Target> {
        let device_id_c = CString::new(self.device_id.clone())
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid device_id: {e}")))?;

        let value = self.call_bridge(b"get_device_json", &device_id_c, "get_device_json")?;

        Ok(Target { value })
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let job_id_c = CString::new(task_id)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid job_id: {e}")))?;

        let value = self.call_bridge(b"get_job_result_json", &job_id_c, "get_job_result_json")?;

        Ok(TaskResult { value })
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let Payload::Oqtopus {
            job_type,
            program,
            shots,
            name,
            description,
            transpiler_info,
            simulator_info,
            mitigation_info,
        } = payload
        else {
            return Err(QrmiError::Other(anyhow::anyhow!(
                "unsupported payload for Oqtopus backend"
            )));
        };

        let program_value = if program.trim_start().starts_with('[') {
            serde_json::from_str::<serde_json::Value>(&program)
                .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid program JSON array: {e}")))?
        } else {
            serde_json::Value::String(program)
        };

        let parse_info = |s: &Option<String>| -> Result<serde_json::Value> {
            match s {
                None => Ok(serde_json::Value::Null),
                Some(s) => serde_json::from_str(s)
                    .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid JSON: {e}"))),
            }
        };

        let job_spec_json = serde_json::json!({
            "job_type": job_type,
            "device_id": self.device_id,
            "program": program_value,
            "shots": shots,
            "name": name,
            "description": description,
            "transpiler_info": parse_info(&transpiler_info)?,
            "simulator_info": parse_info(&simulator_info)?,
            "mitigation_info": parse_info(&mitigation_info)?,
        })
        .to_string();
        let job_spec_json_c = CString::new(job_spec_json)
            .map_err(|e| QrmiError::Other(anyhow::anyhow!("invalid job_spec json: {e}")))?;

        let job_id = self.call_bridge(b"submit_job", &job_spec_json_c, "submit_job")?;

        info!("submit_job succeeded: job_id={job_id}");

        Ok(job_id)
    }
}
