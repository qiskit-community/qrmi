// This code is part of Qiskit.
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
use crate::error::QrmiError;
use std::collections::HashMap;

/// Looks up `key` from `config` if given, otherwise from the OS environment.
/// See [`required_env`]/[`resolve_opt_required`] for the mandatory case.
pub(crate) fn resolve_opt(key: &str, config: Option<&HashMap<String, String>>) -> Option<String> {
    match config {
        Some(map) => map
            .get(key)
            .or_else(|| map.get(&key.to_lowercase()))
            .cloned(),
        None => std::env::var(key).ok(),
    }
}

fn not_found_error(name: String, config: Option<&HashMap<String, String>>) -> QrmiError {
    match config {
        Some(_) => QrmiError::MissingConfigKey(name),
        None => QrmiError::EnvVarNotSet(name),
    }
}

/// Looks up `key` from `config` if given, otherwise from the OS environment.
/// If `key` is not found, raises [`QrmiError::EnvVarNotSet`] or
/// [`QrmiError::MissingConfigKey`] with the variable's name accordingly
pub(crate) fn resolve_opt_required(
    key: &str,
    config: Option<&HashMap<String, String>>,
) -> Result<String, QrmiError> {
    resolve_opt(key, config).ok_or_else(|| not_found_error(key.into(), config))
}

/// Like [`resolve_opt_required`], but tries each of `keys` in order and only
/// fails if none of them are found. Use this when a setting has multiple,
/// independently-valid names.
///
/// Adjacent duplicate names (e.g. a prefixed and a global name that happen
/// to be identical because there was no prefix to begin with) are only
/// tried, and named in the error, once.
pub(crate) fn resolve_opt_required_any(
    keys: &[&str],
    config: Option<&HashMap<String, String>>,
) -> Result<String, QrmiError> {
    let mut keys = keys.to_vec();
    keys.dedup();
    keys.iter()
        .find_map(|key| resolve_opt(key, config))
        .ok_or_else(|| not_found_error(keys.join(" or "), config))
}

/// Reads a required environment variable, returning a [`QrmiError::EnvVarNotSet`]
/// with the variable's name if it isn't set.
pub(crate) fn required_env(name: impl Into<String>) -> Result<String, QrmiError> {
    let name = name.into();
    resolve_opt_required(&name, None)
}
