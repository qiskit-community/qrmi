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
use std::io::Write;
use std::sync::{Arc, Once, RwLock};

static INIT: Once = Once::new();
static LOG_SINK: RwLock<Option<LogSink>> = RwLock::new(None);

/// A destination for `log` records, in plain Rust terms: no C ABI, no
/// pointers. `cext` and `pyext` each register their own adapter here --
/// `cext`'s wraps a C function pointer and does the `CString` conversion
/// at dispatch time (see `cext::qrmi_log_callback_set`); `pyext`'s calls
/// straight into Python's `logging` module, or into a user-supplied
/// Python callable (see `pyext::set_log_callback`). Neither shape leaks
/// into this module.
///
/// `Arc` rather than `Box`: `dispatch_to_sink` below clones this handle
/// and releases `LOG_SINK`'s lock *before* calling it, since a sink may
/// call arbitrary code (a user's own Python callback in particular) that
/// could be slow or could itself log again on the same thread --
/// `std::sync::RwLock` does not guarantee safe recursive read-locking, so
/// still holding the lock across that call would risk a deadlock or worse.
pub(crate) type LogSink = Arc<dyn Fn(log::Level, &str, &str) + Send + Sync>;

/// Registers `sink` as the destination for future `log` records, replacing
/// any previously registered sink. `None` clears it, reverting to plain
/// stderr output.
pub(crate) fn set_log_sink(sink: Option<LogSink>) -> Result<(), ()> {
    LOG_SINK
        .write()
        .map(|mut current| *current = sink)
        .map_err(|_| ())
}

fn dispatch_to_sink(record: &log::Record<'_>) -> bool {
    let sink = {
        let Ok(guard) = LOG_SINK.read() else {
            return false;
        };
        guard.clone()
    };
    let Some(sink) = sink else {
        return false;
    };
    sink(record.level(), record.target(), &record.args().to_string());
    true
}

/// Called once before using the API library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.
pub(crate) fn initialize() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
            .format(|buf, record| {
                if dispatch_to_sink(record) {
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
