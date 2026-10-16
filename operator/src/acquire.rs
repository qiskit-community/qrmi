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

//! QRMI bridge: acquire and release quantum resources on behalf of the operator.
//!
//! All QRMI backends are configured via environment variables.  Because the
//! operator may reconcile multiple claims concurrently inside the same process,
//! and `std::env::set_var` is process-wide, we serialise every acquire/release
//! through a single `tokio::sync::Mutex`.  This mutex is held only for the
//! duration of backend construction + the acquire/release call, not for the
//! lifetime of the claim.

use crate::crd::{QrmiResourceType, QuantumResourceSpec};
use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::Secret;
use kube::Api;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::instrument;

use qrmi::alice_bob::AliceBobFelis;
use qrmi::ibm::{IBMQiskitRuntimeService, IBMQuantumSystem};
use qrmi::iqm::IQMServer;
use qrmi::pasqal::{PasqalCloud, PasqalLocal};
use qrmi::QuantumResource;

/// How long to wait for an `acquire()` call before returning a retryable error.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);

/// Process-wide serialisation lock for environment variable mutation.
static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn env_mutex() -> &'static Mutex<()> {
    ENV_MUTEX.get_or_init(|| Mutex::new(()))
}

/// Resolve all environment variables for a QuantumResource spec, reading
/// Secret-backed values from the cluster.
///
/// Returns a map of `env_var_name -> value` ready to be set in the process
/// environment and/or written into a Kubernetes Secret.
#[instrument(skip_all, fields(resource_id = %spec.resource_id))]
pub async fn resolve_env_vars(
    spec: &QuantumResourceSpec,
    secrets_api: &Api<Secret>,
) -> Result<HashMap<String, String>> {
    let mut vars: HashMap<String, String> = spec.env_vars.clone();

    for secret_ref in &spec.secret_refs {
        let value = read_secret_key(secrets_api, &secret_ref.secret_name, &secret_ref.secret_key)
            .await
            .with_context(|| {
                format!(
                    "reading secret {}/{} for env var {}",
                    secret_ref.secret_name, secret_ref.secret_key, secret_ref.env_var_name
                )
            })?;
        vars.insert(secret_ref.env_var_name.clone(), value);
    }

    Ok(vars)
}

/// Read a single key from a Kubernetes Secret, base64-decoded.
async fn read_secret_key(api: &Api<Secret>, secret_name: &str, key: &str) -> Result<String> {
    let secret = api.get(secret_name).await.with_context(|| {
        format!("getting Secret {secret_name}")
    })?;
    let data = secret
        .data
        .ok_or_else(|| anyhow::anyhow!("Secret {secret_name} has no data"))?;
    let bytes = data
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("key {key} not found in Secret {secret_name}"))?;
    let value = String::from_utf8(bytes.0.clone())
        .with_context(|| format!("Secret {secret_name}/{key} is not valid UTF-8"))?;
    Ok(value)
}

/// Acquire the QRMI lock for the given resource.
///
/// Sets the resolved environment variables in the process environment for the
/// duration of backend construction and the `acquire()` call, then restores the
/// previous values.  The operation is serialised via [`ENV_MUTEX`].
///
/// Returns the acquisition token on success.
#[instrument(skip(env_vars), fields(resource_id = %spec.resource_id))]
pub async fn acquire(
    spec: &QuantumResourceSpec,
    env_vars: &HashMap<String, String>,
) -> Result<String> {
    let _guard = env_mutex().lock().await;
    let env_vars = backend_env_vars(spec, env_vars);

    // Save current values of every variable we are about to mutate.
    let saved: HashMap<String, Option<String>> = env_vars
        .keys()
        .map(|k| (k.clone(), std::env::var(k).ok()))
        .collect();

    // Apply
    for (k, v) in &env_vars {
        // SAFETY: we hold the env_mutex, no concurrent mutation is possible
        // within this process.
        unsafe { std::env::set_var(k, v) };
    }

    let result = do_acquire(spec).await;

    // Restore
    for (k, maybe_v) in &saved {
        unsafe {
            match maybe_v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }

    result
}

/// Release the QRMI lock identified by `token`.
#[instrument(skip(env_vars), fields(resource_id = %spec.resource_id, token = %token))]
pub async fn release(
    spec: &QuantumResourceSpec,
    env_vars: &HashMap<String, String>,
    token: &str,
) -> Result<()> {
    let _guard = env_mutex().lock().await;
    let env_vars = backend_env_vars(spec, env_vars);

    let saved: HashMap<String, Option<String>> = env_vars
        .keys()
        .map(|k| (k.clone(), std::env::var(k).ok()))
        .collect();

    for (k, v) in &env_vars {
        unsafe { std::env::set_var(k, v) };
    }

    let result = do_release(spec, token).await;

    for (k, maybe_v) in &saved {
        unsafe {
            match maybe_v {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Construct the correct QRMI backend and call `acquire()`, with a timeout.
async fn do_acquire(spec: &QuantumResourceSpec) -> Result<String> {
    let mut backend = build_backend(spec)?;
    timeout(ACQUIRE_TIMEOUT, backend.acquire())
        .await
        .with_context(|| {
            format!(
                "acquire() timed out after {}s for resource {}",
                ACQUIRE_TIMEOUT.as_secs(),
                spec.resource_id
            )
        })?
        .with_context(|| format!("acquire() failed for resource {}", spec.resource_id))
}

/// Construct the correct QRMI backend and call `release()`.
async fn do_release(spec: &QuantumResourceSpec, token: &str) -> Result<()> {
    let mut backend = build_backend(spec)?;
    backend
        .release(token)
        .await
        .with_context(|| format!("release() failed for resource {}", spec.resource_id))
}

/// Instantiate the correct QRMI backend from `spec.resource_type`.
/// Environment variables must already be set by the caller.
fn build_backend(spec: &QuantumResourceSpec) -> Result<Box<dyn QuantumResource>> {
    let id = &spec.resource_id;
    let backend: Box<dyn QuantumResource> = match spec.resource_type {
        QrmiResourceType::DirectAccess => {
            anyhow::bail!(
                "direct-access is unavailable in this QRMI build; configure a supported resource type"
            )
        }
        QrmiResourceType::IbmQuantumSystem => {
            Box::new(IBMQuantumSystem::new(id).with_context(|| {
                format!("constructing IBMQuantumSystem for {id}")
            })?)
        }
        QrmiResourceType::QiskitRuntimeService => {
            Box::new(IBMQiskitRuntimeService::new(id).with_context(|| {
                format!("constructing IBMQiskitRuntimeService for {id}")
            })?)
        }
        QrmiResourceType::PasqalCloud => {
            Box::new(PasqalCloud::new(id).with_context(|| {
                format!("constructing PasqalCloud for {id}")
            })?)
        }
        QrmiResourceType::PasqalLocal => {
            Box::new(PasqalLocal::new(id).with_context(|| {
                format!("constructing PasqalLocal for {id}")
            })?)
        }
        QrmiResourceType::AliceBobFelis => {
            Box::new(AliceBobFelis::new(id).with_context(|| {
                format!("constructing AliceBobFelis for {id}")
            })?)
        }
        QrmiResourceType::IqmServer => {
            Box::new(IQMServer::new(id).with_context(|| {
                format!("constructing IQMServer for {id}")
            })?)
        }
    };
    Ok(backend)
}

// Modified to scope IBM Runtime credentials for operator acquisition and release.
fn backend_env_vars(
    spec: &QuantumResourceSpec,
    env_vars: &HashMap<String, String>,
) -> HashMap<String, String> {
    env_vars
        .iter()
        .map(|(key, value)| {
            let name = if spec.resource_type == QrmiResourceType::QiskitRuntimeService
                && key.starts_with("QRMI_IBM_QRS_")
            {
                format!("{}_{}", spec.resource_id, key)
            } else {
                key.clone()
            };
            (name, value.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ibm_credentials_are_scoped_without_changing_other_variables() {
        let spec = QuantumResourceSpec {
            resource_id: "ibm_fez".into(),
            resource_type: QrmiResourceType::QiskitRuntimeService,
            env_vars: HashMap::new(),
            secret_refs: vec![],
        };
        let vars = HashMap::from([
            ("QRMI_IBM_QRS_IAM_APIKEY".into(), "test-value".into()),
            ("QRMI_JOB_TIMEOUT_SECONDS".into(), "60".into()),
        ]);
        let scoped = backend_env_vars(&spec, &vars);
        assert_eq!(scoped["ibm_fez_QRMI_IBM_QRS_IAM_APIKEY"], "test-value");
        assert!(!scoped.contains_key("QRMI_IBM_QRS_IAM_APIKEY"));
        assert_eq!(scoped["QRMI_JOB_TIMEOUT_SECONDS"], "60");

        let other = QuantumResourceSpec {
            resource_type: QrmiResourceType::AliceBobFelis,
            ..spec
        };
        assert_eq!(backend_env_vars(&other, &vars), vars);
    }
}
