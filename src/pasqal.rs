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

/// Env var (unprefixed) that, when set to a truthy value, disables HTTP retries
/// for a Pasqal client. QRMI namespaces backend configuration under a
/// `<backend_name>_` prefix, so this is resolved as
/// `<backend_name>_QRMI_PASQAL_RETRIES_DISABLED` (e.g.
/// `PASQAL_LOCAL_QRMI_PASQAL_RETRIES_DISABLED`), falling back to the unprefixed
/// name as a global override (e.g. for e2e tests).
const RETRIES_DISABLED_ENV: &str = "QRMI_PASQAL_RETRIES_DISABLED";

/// Returns `true` if HTTP retries should be disabled for `backend_name`.
///
/// Checks `<backend_name>_QRMI_PASQAL_RETRIES_DISABLED` first — matching how the
/// rest of QRMI's per-backend configuration is namespaced — then falls back to
/// the unprefixed `QRMI_PASQAL_RETRIES_DISABLED` as a global override. A value
/// of `1`, `true`, `yes`, or `on` (case-insensitive) disables retries; anything
/// else (or unset) leaves the default retry policy in place.
fn retries_disabled(backend_name: &str) -> bool {
    let is_truthy = |v: String| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    };
    let scoped = format!("{backend_name}_{RETRIES_DISABLED_ENV}");
    env::var(&scoped)
        .or_else(|_| env::var(RETRIES_DISABLED_ENV))
        .map(is_truthy)
        .unwrap_or(false)
}
