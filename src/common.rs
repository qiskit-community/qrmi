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
use std::sync::Once;

static INIT: Once = Once::new();

/// Called once before using the API library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.
///
/// Uses `try_init()`, not `init()`: `init()` panics if another library in
/// the same process already registered a `log` logger first (only one can
/// ever be registered process-wide), and -- because that failure happens
/// inside this `Once::call_once` closure -- a panic here poisons `INIT`,
/// so every later call to `initialize()` (this runs at the top of nearly
/// every QRMI entry point) panics too, for the rest of the process's
/// lifetime. `try_init()` fails quietly instead: if we lose that race,
/// QRMI's `log` output is simply not registered (records fall through to
/// whichever logger did win), rather than taking down every subsequent
/// call into QRMI.
pub(crate) fn initialize() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    });
}
