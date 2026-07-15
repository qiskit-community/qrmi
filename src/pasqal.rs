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

use log::warn;
use pasqal_common::DEFAULT_MAX_RETRIES;
use std::env;

/// Env var (unprefixed) that, when set to a truthy value, disables HTTP retries
/// for a Pasqal client.
const RETRIES_DISABLED_ENV: &str = "QRMI_PASQAL_RETRIES_DISABLED";

/// Env var (unprefixed) that sets how many times a failed HTTP request is
/// retried. Defaults to [`DEFAULT_MAX_RETRIES`] when unset or unparseable.
const MAX_RETRY_COUNT_ENV: &str = "QRMI_PASQAL_MAX_RETRY_COUNT";

/// Whether a Pasqal client retries transient HTTP failures.
///
/// Retries are **off** unless a caller explicitly asks for them, because not
/// every QRMI consumer can afford to block. The Slurm SPANK plugin calls QRMI
/// through the C bindings while holding up job launch, and must fail fast; a
/// job's own requests, made through the Python bindings, are the ones that want
/// to ride out a transient outage. Keeping the default at [`Self::Disabled`]
/// means a new call site has to opt in deliberately rather than inherit
/// blocking behavior by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Retries {
    /// Never retry, regardless of environment. Used by every entry point except
    /// the Python bindings.
    Disabled,
    /// Retry using the default policy, unless the environment opts out. Used by
    /// the Python bindings, so a job can still tune or turn off retries.
    EnabledUnlessEnvOptsOut,
}

impl Retries {
    /// How many times the client should retry a failed request, or `None` when
    /// it should not retry at all.
    pub(crate) fn max_retries_for(self, backend_name: &str) -> Option<u32> {
        match self {
            Self::Disabled => None,
            Self::EnabledUnlessEnvOptsOut => {
                (!disabled_by_env(backend_name)).then(|| max_retry_count(backend_name))
            }
        }
    }
}

/// Reads `var` for `backend_name`.
///
/// QRMI namespaces backend configuration under a `<backend_name>_` prefix, so
/// `<backend_name>_<var>` (e.g. `FRESNEL_QRMI_PASQAL_MAX_RETRY_COUNT`) wins,
/// falling back to the unprefixed name as a global override (e.g. for e2e
/// tests).
fn scoped_var(backend_name: &str, var: &str) -> Option<String> {
    env::var(format!("{backend_name}_{var}"))
        .or_else(|_| env::var(var))
        .ok()
}

/// Returns `true` if the environment opts `backend_name` out of retries.
///
/// A value of `1`, `true`, `yes`, or `on` (case-insensitive) disables retries;
/// anything else (or unset) leaves the retry policy in place.
fn disabled_by_env(backend_name: &str) -> bool {
    scoped_var(backend_name, RETRIES_DISABLED_ENV)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

/// Returns how many times `backend_name` should retry a failed request.
///
/// A value that is not a non-negative integer is ignored — a typo'd retry count
/// should not silently change how long a job hangs on a failing backend — and
/// the default is used instead.
fn max_retry_count(backend_name: &str) -> u32 {
    let Some(value) = scoped_var(backend_name, MAX_RETRY_COUNT_ENV) else {
        return DEFAULT_MAX_RETRIES;
    };
    match value.trim().parse::<u32>() {
        Ok(count) => count,
        Err(_) => {
            warn!(
                "Ignoring {MAX_RETRY_COUNT_ENV}='{value}': expected a non-negative integer, using {DEFAULT_MAX_RETRIES}"
            );
            DEFAULT_MAX_RETRIES
        }
    }
}

/// Guards the retry environment variables, which are process-wide.
///
/// Shared by the tests in this module and in `pasqal/tests/local.rs`: both read
/// or write the same variables, so they must not run concurrently.
#[cfg(test)]
pub(crate) fn env_lock() -> &'static std::sync::Mutex<()> {
    static ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    ENV_LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BACKEND: &str = "FRESNEL";

    /// Runs `body` with the retry env vars cleared, restoring them afterwards.
    fn with_clean_env(body: impl FnOnce()) {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let vars = [
            format!("{BACKEND}_{RETRIES_DISABLED_ENV}"),
            RETRIES_DISABLED_ENV.to_string(),
            format!("{BACKEND}_{MAX_RETRY_COUNT_ENV}"),
            MAX_RETRY_COUNT_ENV.to_string(),
        ];
        let saved: Vec<_> = vars.iter().map(|v| (v, env::var(v).ok())).collect();
        for var in &vars {
            env::remove_var(var);
        }

        body();

        for (var, value) in saved {
            match value {
                Some(v) => env::set_var(var, v),
                None => env::remove_var(var),
            }
        }
    }

    #[test]
    fn disabled_never_retries_even_when_env_is_silent() {
        with_clean_env(|| {
            assert_eq!(Retries::Disabled.max_retries_for(BACKEND), None);
        });
    }

    #[test]
    fn disabled_ignores_env_opt_in_attempts() {
        // The C bindings must never retry, so no environment value can turn
        // retries back on for a `Retries::Disabled` caller.
        with_clean_env(|| {
            env::set_var(format!("{BACKEND}_{RETRIES_DISABLED_ENV}"), "false");
            env::set_var(RETRIES_DISABLED_ENV, "false");
            env::set_var(MAX_RETRY_COUNT_ENV, "10");
            assert_eq!(Retries::Disabled.max_retries_for(BACKEND), None);
        });
    }

    #[test]
    fn enabled_retries_the_default_number_of_times() {
        with_clean_env(|| {
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                Some(DEFAULT_MAX_RETRIES)
            );
        });
    }

    #[test]
    fn enabled_honors_scoped_env_opt_out() {
        with_clean_env(|| {
            env::set_var(format!("{BACKEND}_{RETRIES_DISABLED_ENV}"), "1");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                None
            );
        });
    }

    #[test]
    fn enabled_honors_global_env_opt_out() {
        with_clean_env(|| {
            env::set_var(RETRIES_DISABLED_ENV, "on");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                None
            );
        });
    }

    #[test]
    fn scoped_env_takes_precedence_over_global() {
        with_clean_env(|| {
            env::set_var(RETRIES_DISABLED_ENV, "true");
            env::set_var(format!("{BACKEND}_{RETRIES_DISABLED_ENV}"), "false");
            env::set_var(MAX_RETRY_COUNT_ENV, "9");
            env::set_var(format!("{BACKEND}_{MAX_RETRY_COUNT_ENV}"), "2");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                Some(2)
            );
        });
    }

    #[test]
    fn non_truthy_env_values_leave_retries_on() {
        with_clean_env(|| {
            env::set_var(RETRIES_DISABLED_ENV, "maybe");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                Some(DEFAULT_MAX_RETRIES)
            );
        });
    }

    #[test]
    fn max_retry_count_is_configurable() {
        with_clean_env(|| {
            env::set_var(MAX_RETRY_COUNT_ENV, "3");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                Some(3)
            );
        });
    }

    #[test]
    fn zero_max_retry_count_means_a_single_attempt() {
        with_clean_env(|| {
            env::set_var(MAX_RETRY_COUNT_ENV, "0");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                Some(0)
            );
        });
    }

    #[test]
    fn unparseable_max_retry_count_falls_back_to_the_default() {
        with_clean_env(|| {
            env::set_var(MAX_RETRY_COUNT_ENV, "lots");
            assert_eq!(
                Retries::EnabledUnlessEnvOptsOut.max_retries_for(BACKEND),
                Some(DEFAULT_MAX_RETRIES)
            );
        });
    }
}
