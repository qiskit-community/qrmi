// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal, UKRI-STFC (Hartree Centre) 2025 - 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;
use std::sync::{Once, RwLock};

static INIT: Once = Once::new();
static LOG_CALLBACK: RwLock<QrmiLogCallback> = RwLock::new(None);

pub type QrmiLogCallback = Option<
    unsafe extern "C" fn(level: *const c_char, target: *const c_char, message: *const c_char),
>;

pub(crate) fn set_log_callback(callback: QrmiLogCallback) -> Result<(), ()> {
    LOG_CALLBACK
        .write()
        .map(|mut current| *current = callback)
        .map_err(|_| ())
}

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

fn dispatch_to_callback(record: &log::Record<'_>) -> bool {
    let callback = LOG_CALLBACK.read().ok().and_then(|current| *current);
    let Some(callback) = callback else {
        return false;
    };
    let level = sanitized_cstring(record.level().as_str());
    let target = sanitized_cstring(record.target());
    let message = sanitized_cstring(&record.args().to_string());
    unsafe {
        callback(level.as_ptr(), target.as_ptr(), message.as_ptr());
    }
    true
}

/// Called once before using the API library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.
pub(crate) fn initialize() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format(|buf, record| {
                if dispatch_to_callback(record) {
                    Ok(())
                } else {
                    writeln!(
                        buf,
                        "[{} {} {}] {}",
                        buf.timestamp(),
                        record.level(),
                        record.target(),
                        record.args()
                    )
                }
            })
            .init();
    });
}
