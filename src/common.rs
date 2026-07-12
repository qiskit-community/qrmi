// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal, UKRI-STFC (Hartree Centre) 2025
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
use std::sync::{Mutex, Once};

static INIT: Once = Once::new();
static LOG_CALLBACK: Mutex<QrmiLogCallback> = Mutex::new(None);

pub type QrmiLogCallback = Option<
    unsafe extern "C" fn(level: *const c_char, target: *const c_char, message: *const c_char),
>;

pub(crate) fn set_log_callback(callback: QrmiLogCallback) {
    if let Ok(mut current) = LOG_CALLBACK.lock() {
        *current = callback;
    }
}

fn callback_log(record: &log::Record<'_>) -> bool {
    let callback = LOG_CALLBACK.lock().ok().and_then(|current| *current);
    let Some(callback) = callback else {
        return false;
    };
    let level = CString::new(record.level().as_str()).ok();
    let target = CString::new(record.target()).ok();
    let message = CString::new(record.args().to_string()).ok();
    if let (Some(level), Some(target), Some(message)) = (level, target, message) {
        unsafe {
            callback(level.as_ptr(), target.as_ptr(), message.as_ptr());
        }
        true
    } else {
        false
    }
}

/// Called once before using the API library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.
pub(crate) fn initialize() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format(|buf, record| {
                if callback_log(record) {
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
