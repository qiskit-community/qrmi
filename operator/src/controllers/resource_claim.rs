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

//! Controller for `QuantumResourceClaim` objects.
//! The controller also manages a Kubernetes Secret named `qrc-<claim-name>`
//! in the same namespace, containing all resolved environment variables.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta as K8sObjectMeta;
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::{Api, Client, ResourceExt};
use tracing::{debug, error, info, instrument, warn};

use crate::acquire;
use crate::crd::{
    ClaimPhase, QuantumResource, QuantumResourceClaim, QuantumResourceClaimStatus,
};
use crate::error::ControllerError;

const FINALIZER: &str = "quantum.qrmi.io/finalizer";
const REQUEUE_ON_PENDING: Duration = Duration::from_secs(15);
const REQUEUE_BOUND_POLL: Duration = Duration::from_secs(60);

/// Shared operator state passed to every reconciler invocation.
pub struct Context {
    pub client: Client,
}

#[instrument(skip(ctx, claim), fields(name = %claim.name_any(), ns = claim.namespace().as_deref().unwrap_or("-")))]
pub async fn reconcile(
    claim: Arc<QuantumResourceClaim>,
    ctx: Arc<Context>,
) -> std::result::Result<Action, ControllerError> {
    let ns = claim.namespace().unwrap_or_default();
    let claims_api: Api<QuantumResourceClaim> = Api::namespaced(ctx.client.clone(), &ns);

    // Handle deletion via finalizer
    if claim.metadata.deletion_timestamp.is_some() {
        if has_finalizer(&claim) {
            if let Err(e) = cleanup(&claim, &ctx).await {
                error!("cleanup error: {e}");
            }
            remove_finalizer(&claims_api, &claim.name_any()).await?;
        }
        return Ok(Action::await_change());
    }

    // Ensure finalizer is present
    if !has_finalizer(&claim) {
        add_finalizer(&claims_api, &claim.name_any()).await?;
        return Ok(Action::requeue(Duration::from_secs(1)));
    }

    // Main reconcile
    apply(claim, &ns, &ctx).await.map_err(ControllerError::from)
}

pub fn error_policy(
    _claim: Arc<QuantumResourceClaim>,
    err: &ControllerError,
    _ctx: Arc<Context>,
) -> Action {
    warn!("claim reconcile error: {err}");
    Action::requeue(REQUEUE_ON_PENDING)
}

// ---------------------------------------------------------------------------
// Apply (create / update)
// ---------------------------------------------------------------------------

async fn apply(claim: Arc<QuantumResourceClaim>, ns: &str, ctx: &Arc<Context>) -> Result<Action> {
    let phase = claim
        .status
        .as_ref()
        .map(|s| s.phase.clone())
        .unwrap_or_default();

    match phase {
        ClaimPhase::Pending => {
            info!(claim = %claim.name_any(), ns = %ns, "processing Pending QuantumResourceClaim");
            handle_pending(&claim, ns, ctx).await
        }
        ClaimPhase::Bound => {
            debug!(claim = %claim.name_any(), ns = %ns, "processing Bound QuantumResourceClaim");
            handle_bound(&claim, ns, ctx).await
        }
        ClaimPhase::Released | ClaimPhase::Failed => Ok(Action::await_change()),
    }
}

async fn handle_pending(
    claim: &QuantumResourceClaim,
    ns: &str,
    ctx: &Arc<Context>,
) -> Result<Action> {
    let resource_name = &claim.spec.quantum_resource;
    let qr_api: Api<QuantumResource> = Api::all(ctx.client.clone());

    let qr = match qr_api.get(resource_name).await {
        Ok(r) => {
            info!(claim = %claim.name_any(), resource = %resource_name, "resolved QuantumResource");
            r
        }
        Err(e) => {
            warn!(claim = %claim.name_any(), resource = %resource_name, "QuantumResource not found: {e}");
            patch_status(
                ctx,
                ns,
                &claim.name_any(),
                ClaimPhase::Pending,
                None,
                None,
                None,
                Some(format!("QuantumResource {resource_name} not found: {e}")),
            )
            .await?;
            return Ok(Action::requeue(REQUEUE_ON_PENDING));
        }
    };

    let secrets_api: Api<Secret> = Api::namespaced(ctx.client.clone(), ns);
    let env_vars = match acquire::resolve_env_vars(&qr.spec, &secrets_api).await {
        Ok(v) => v,
        Err(e) => {
            warn!(claim = %claim.name_any(), resource = %resource_name, "failed to resolve env vars: {e}");
            patch_status(
                ctx,
                ns,
                &claim.name_any(),
                ClaimPhase::Pending,
                None,
                None,
                None,
                Some(format!("env var resolution failed: {e}")),
            )
            .await?;
            return Ok(Action::requeue(REQUEUE_ON_PENDING));
        }
    };

    let token = match acquire::acquire(&qr.spec, &env_vars).await {
        Ok(t) => t,
        Err(e) => {
            warn!(claim = %claim.name_any(), resource = %resource_name, "acquire failed (will retry): {e}");
            patch_status(
                ctx,
                ns,
                &claim.name_any(),
                ClaimPhase::Pending,
                None,
                None,
                None,
                Some(format!("acquire failed (will retry): {e}")),
            )
            .await?;
            return Ok(Action::requeue(REQUEUE_ON_PENDING));
        }
    };

    let secret_name = claim.spec.secret_name.clone().unwrap_or_else(|| secret_name_for(&claim.name_any()));
    let resource_prefix = qr.spec.resource_id.to_lowercase().replace('-', "_");
    let mut secret_data: BTreeMap<String, String> = env_vars
        .clone()
        .into_iter()
        .map(|(k, v)| (format!("{resource_prefix}_{k}"), v))
        .collect();
    secret_data.insert(
        format!("{resource_prefix}_QRMI_JOB_ACQUISITION_TOKEN"),
        token.clone(),
    );

    create_or_update_secret(ctx, ns, &secret_name, secret_data, claim).await?;

    let now = Utc::now();
    let expires_at = claim
        .spec
        .ttl
        .map(|ttl| (now + chrono::Duration::seconds(ttl as i64)).to_rfc3339());

    patch_status(
        ctx,
        ns,
        &claim.name_any(),
        ClaimPhase::Bound,
        Some(secret_name.clone()),
        Some(now.to_rfc3339()),
        expires_at,
        None,
    )
    .await?;

    info!(
        claim = %claim.name_any(), resource = %resource_name,
        secret = %secret_name, "claim Bound; token acquired"
    );
    Ok(Action::requeue(REQUEUE_BOUND_POLL))
}

async fn handle_bound(
    claim: &QuantumResourceClaim,
    ns: &str,
    ctx: &Arc<Context>,
) -> Result<Action> {
    let status = match &claim.status {
        Some(s) => s,
        None => return Ok(Action::requeue(REQUEUE_BOUND_POLL)),
    };

    if let Some(expires_at_str) = &status.expires_at {
        if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_at_str).map(|dt| dt.with_timezone(&Utc)) {
            if Utc::now() >= expires_at {
                info!(claim = %claim.name_any(), "TTL expired; deleting claim");
                let claims_api: Api<QuantumResourceClaim> =
                    Api::namespaced(ctx.client.clone(), ns);
                claims_api
                    .delete(&claim.name_any(), &Default::default())
                    .await?;
                return Ok(Action::await_change());
            }
            let remaining = (expires_at - Utc::now())
                .to_std()
                .unwrap_or(REQUEUE_BOUND_POLL);
            return Ok(Action::requeue(remaining.min(REQUEUE_BOUND_POLL)));
        }
    }

    Ok(Action::requeue(REQUEUE_BOUND_POLL))
}

// ---------------------------------------------------------------------------
// Cleanup (on deletion)
// ---------------------------------------------------------------------------

async fn cleanup(claim: &QuantumResourceClaim, ctx: &Arc<Context>) -> Result<()> {
    let ns = claim.namespace().unwrap_or_default();
    let secret_name = claim.spec.secret_name.clone().unwrap_or_else(|| secret_name_for(&claim.name_any()));
    let secrets_api: Api<Secret> = Api::namespaced(ctx.client.clone(), &ns);

    if let Some(status) = &claim.status {
        if status.phase == ClaimPhase::Bound {
            let qr_api: Api<QuantumResource> = Api::all(ctx.client.clone());
            if let Ok(qr) = qr_api.get(&claim.spec.quantum_resource).await {
                let token_key = format!(
                    "{}_QRMI_JOB_ACQUISITION_TOKEN",
                    qr.spec.resource_id.to_lowercase().replace('-', "_")
                );
                if let Ok(secret) = secrets_api.get(&secret_name).await {
                    if let Some(data) = &secret.data {
                        if let Some(token_bytes) = data.get(&token_key) {
                            if let Ok(token) = String::from_utf8(token_bytes.0.clone()) {
                                let env_vars =
                                    acquire::resolve_env_vars(&qr.spec, &secrets_api)
                                        .await
                                        .unwrap_or_default();
                                if let Err(e) =
                                    acquire::release(&qr.spec, &env_vars, &token).await
                                {
                                    warn!(claim = %claim.name_any(), "release failed (continuing cleanup): {e}");
                                } else {
                                    info!(claim = %claim.name_any(), resource = %claim.spec.quantum_resource, "released quantum resource lock");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Err(e) = secrets_api
        .delete(&secret_name, &Default::default())
        .await
    {
        if !is_not_found(&e) {
            warn!("failed to delete Secret {secret_name}: {e}");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Finalizer helpers
// ---------------------------------------------------------------------------

fn has_finalizer(claim: &QuantumResourceClaim) -> bool {
    claim
        .metadata
        .finalizers
        .as_ref()
        .map(|f| f.iter().any(|s| s == FINALIZER))
        .unwrap_or(false)
}

async fn add_finalizer(
    api: &Api<QuantumResourceClaim>,
    name: &str,
) -> std::result::Result<(), ControllerError> {
    let patch = serde_json::json!({
        "metadata": { "finalizers": [FINALIZER] }
    });
    api.patch(name, &PatchParams::apply("qrmi-operator"), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

async fn remove_finalizer(
    api: &Api<QuantumResourceClaim>,
    name: &str,
) -> std::result::Result<(), ControllerError> {
    // Patch the finalizers list to remove our entry.
    let claim = api.get(name).await?;
    let finalizers: Vec<String> = claim
        .metadata
        .finalizers
        .unwrap_or_default()
        .into_iter()
        .filter(|f| f != FINALIZER)
        .collect();
    let patch = serde_json::json!({ "metadata": { "finalizers": finalizers } });
    api.patch(name, &PatchParams::apply("qrmi-operator"), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Secret management
// ---------------------------------------------------------------------------

pub fn secret_name_for(claim_name: &str) -> String {
    format!("qrc-{claim_name}")
}

async fn create_or_update_secret(
    ctx: &Arc<Context>,
    ns: &str,
    secret_name: &str,
    data: BTreeMap<String, String>,
    owner: &QuantumResourceClaim,
) -> Result<()> {
    let secrets_api: Api<Secret> = Api::namespaced(ctx.client.clone(), ns);

    let owner_ref = OwnerReference {
        api_version: "quantum.qrmi.io/v1alpha1".to_string(),
        kind: "QuantumResourceClaim".to_string(),
        name: owner.name_any(),
        uid: owner.uid().unwrap_or_default(),
        block_owner_deletion: Some(true),
        controller: Some(true),
    };

    let secret = Secret {
        metadata: K8sObjectMeta {
            name: Some(secret_name.to_string()),
            namespace: Some(ns.to_string()),
            owner_references: Some(vec![owner_ref]),
            labels: Some(BTreeMap::from([
                (
                    "app.kubernetes.io/managed-by".to_string(),
                    "qrmi-operator".to_string(),
                ),
                (
                    "quantum.qrmi.io/claim".to_string(),
                    owner.name_any(),
                ),
            ])),
            ..Default::default()
        },
        string_data: Some(data),
        ..Default::default()
    };

    secrets_api
        .patch(
            secret_name,
            &PatchParams::apply("qrmi-operator"),
            &Patch::Apply(&secret),
        )
        .await?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Status helper
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
async fn patch_status(
    ctx: &Arc<Context>,
    ns: &str,
    name: &str,
    phase: ClaimPhase,
    secret_name: Option<String>,
    bound_at: Option<String>,
    expires_at: Option<String>,
    message: Option<String>,
) -> Result<()> {
    let claims_api: Api<QuantumResourceClaim> = Api::namespaced(ctx.client.clone(), ns);
    let status = QuantumResourceClaimStatus {
        phase,
        secret_name,
        bound_at,
        expires_at,
        message,
    };
    let patch = serde_json::json!({ "status": status });
    claims_api
        .patch_status(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

fn is_not_found(e: &kube::Error) -> bool {
    matches!(e, kube::Error::Api(r) if r.code == 404)
}


