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

//! QRMI implementations for Pasqal Cloud Services and Pasqal Local

mod cloud;
mod cloud_config;
mod local;

pub use self::cloud::PasqalCloud;
pub use self::local::PasqalLocal;

use std::env;

/// Env var that, when set to a truthy value, disables HTTP retries for all
/// Pasqal clients. This is a global toggle; it is not scoped per backend.
const RETRIES_DISABLED_ENV: &str = "QRMI_PASQAL_RETRIES_DISABLED";

/// Returns `true` if HTTP retries should be disabled for the Pasqal clients.
///
/// Reads the `QRMI_PASQAL_RETRIES_DISABLED` env var: a value of `1`, `true`,
/// `yes`, or `on` (case-insensitive) disables retries; anything else (or unset)
/// leaves the default retry policy in place.
fn retries_disabled() -> bool {
    let is_truthy = |v: String| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    };
    env::var(RETRIES_DISABLED_ENV)
        .map(is_truthy)
        .unwrap_or(false)
}
