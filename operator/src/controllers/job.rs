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

//! Controller for annotated Kubernetes Jobs.
//!
//! When a Job carries the label `quantum.qrmi.io/resource: <resource-name>`,
//! the operator:
//!
//! 1. Suspends the Job (`spec.suspend: true`).
//! 2. Creates a `QuantumResourceClaim` owned by the Job.
//! 3. Waits until the claim is `Bound` (its Secret exists).
//! 4. Patches the Job's pod template to inject `envFrom: [{secretRef: ...}]`.
//! 5. Unsuspends the Job.
//!
//! When the Job completes or is deleted, the ownerReference cascade-deletes
//! the claim, triggering the claim controller's cleanup / release path.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use k8s_openapi::api::batch::v1::Job;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::{Api, Client, ResourceExt};
use tracing::{debug, info, instrument, warn};

use crate::controllers::resource_claim::secret_name_for;
use crate::crd::{ClaimPhase, QuantumResourceClaim, QuantumResourceClaimSpec};
use crate::error::{ControllerError, Result};

/// Annotation that opts a Job into QRMI resource management.
/// Value is the name of the QuantumResource to claim.
pub const ANNOTATION_RESOURCE: &str = "quantum.qrmi.io/resource";
/// Optional annotation: TTL in seconds for the resulting claim.
pub const ANNOTATION_TTL: &str = "quantum.qrmi.io/ttl";
/// Optional annotation: override the name of the Secret the operator creates.
/// Defaults to `qrc-<job-name>`.
pub const ANNOTATION_SECRET_NAME: &str = "quantum.qrmi.io/secret-name";

const REQUEUE_WAITING: Duration = Duration::from_secs(5);

pub struct Context {
    pub client: Client,
}

#[instrument(skip(ctx, job), fields(name = %job.name_any(), ns = job.namespace().as_deref().unwrap_or("-")))]
pub async fn reconcile(
    job: Arc<Job>,
    ctx: Arc<Context>,
) -> std::result::Result<Action, ControllerError> {
    let annotations = job.annotations();
    let resource_name = match annotations.get(ANNOTATION_RESOURCE) {
        Some(v) => v.clone(),
        None => {
            debug!("Job {} has no quantum.qrmi.io/resource annotation; skipping", job.name_any());
            return Ok(Action::await_change());
        }
    };

    let ns = job.namespace().unwrap_or_default();
    debug!(
        job = %job.name_any(), ns = %ns, resource = %resource_name,
        "reconciling annotated Job"
    );

    let jobs_api: Api<Job> = Api::namespaced(ctx.client.clone(), &ns);
    let claims_api: Api<QuantumResourceClaim> = Api::namespaced(ctx.client.clone(), &ns);

    let claim_name = claim_name_for(&job.name_any());
    let custom_secret = annotations.get(ANNOTATION_SECRET_NAME).cloned();
    let secret_name = custom_secret.clone().unwrap_or_else(|| secret_name_for(&claim_name));

    let existing_claim = claims_api.get(&claim_name).await.ok();

    match existing_claim {
        None => {
            // Step 1: suspend the job before creating the claim.
            info!(job = %job.name_any(), "suspending Job and creating QuantumResourceClaim {claim_name}");
            suspend_job(&jobs_api, &job.name_any()).await?;

            // Step 2: create the claim owned by this job.
            let ttl: Option<u32> = annotations.get(ANNOTATION_TTL).and_then(|s| s.parse().ok());
            create_claim(&claims_api, &claim_name, &resource_name, ttl, custom_secret, &job).await?;
            info!(claim = %claim_name, job = %job.name_any(), "QuantumResourceClaim created");
            Ok(Action::requeue(REQUEUE_WAITING))
        }
        Some(claim) => {
            let phase = claim
                .status
                .as_ref()
                .map(|s| s.phase.clone())
                .unwrap_or_default();

            debug!(claim = %claim_name, job = %job.name_any(), phase = ?phase, "existing claim found");

            match phase {
                ClaimPhase::Pending => {
                    debug!(claim = %claim_name, "claim still Pending; requeuing");
                    Ok(Action::requeue(REQUEUE_WAITING))
                }
                ClaimPhase::Bound => {
                    // Release the claim as soon as the job has finished (complete or failed),
                    // without waiting for the Job object itself to be garbage-collected.
                    debug!(claim = %claim_name, job = %job.name_any(), "claim Bound; checking job state");
                    if job_has_finished(&job) {
                        claims_api
                            .delete(&claim_name, &Default::default())
                            .await
                            .ok(); // ignore not-found; finalizer handles release
                        info!(claim = %claim_name, job = %job.name_any(), "Job finished; deleted claim to release resource");
                        return Ok(Action::await_change());
                    }

                    let is_suspended = job
                        .spec
                        .as_ref()
                        .and_then(|s| s.suspend)
                        .unwrap_or(false);

                    if is_suspended {
                        unsuspend_job(&jobs_api, &job.name_any()).await?;
                        info!(
                            secret = %secret_name, job = %job.name_any(),
                            "Secret bound, unsuspended Job"
                        );
                    }
                    Ok(Action::await_change())
                }
                ClaimPhase::Failed => {
                    warn!(claim = %claim_name, job = %job.name_any(), "claim Failed; Job is stalled");
                    Ok(Action::await_change())
                }
                ClaimPhase::Released => {
                    info!(claim = %claim_name, job = %job.name_any(), "claim Released");
                    Ok(Action::await_change())
                }
            }
        }
    }
}

pub fn error_policy(_job: Arc<Job>, err: &ControllerError, _ctx: Arc<Context>) -> Action {
    warn!("job reconcile error: {err}");
    Action::requeue(REQUEUE_WAITING)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn claim_name_for(job_name: &str) -> String {
    format!("job-{job_name}")
}

async fn suspend_job(api: &Api<Job>, name: &str) -> Result<()> {
    let patch = serde_json::json!({"spec": {"suspend": true}});
    api.patch(name, &PatchParams::apply("qrmi-operator"), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

async fn create_claim(
    api: &Api<QuantumResourceClaim>,
    claim_name: &str,
    resource_ref: &str,
    ttl: Option<u32>,
    secret_name: Option<String>,
    owner_job: &Job,
) -> Result<()> {
    let owner_ref = OwnerReference {
        api_version: "batch/v1".to_string(),
        kind: "Job".to_string(),
        name: owner_job.name_any(),
        uid: owner_job.uid().unwrap_or_default(),
        block_owner_deletion: Some(true),
        controller: Some(true),
    };

    let mut claim = QuantumResourceClaim::new(
        claim_name,
        QuantumResourceClaimSpec {
            quantum_resource: resource_ref.to_string(),
            secret_name,
            ttl,
        },
    );
    claim.metadata.owner_references = Some(vec![owner_ref]);
    claim.metadata.labels = Some(BTreeMap::from([(
        "quantum.qrmi.io/job".to_string(),
        owner_job.name_any(),
    )]));

    api.patch(
        claim_name,
        &PatchParams::apply("qrmi-operator"),
        &Patch::Apply(&claim),
    )
    .await?;

    Ok(())
}

async fn unsuspend_job(api: &Api<Job>, job_name: &str) -> Result<()> {
    let patch = serde_json::json!({"spec": {"suspend": false}});
    api.patch(job_name, &PatchParams::apply("qrmi-operator"), &Patch::Merge(&patch))
        .await?;
    Ok(())
}

/// Returns true if the Job has run to completion (success or terminal failure).
fn job_has_finished(job: &Job) -> bool {
    let status = match job.status.as_ref() {
        Some(s) => s,
        None => return false,
    };
    // Completed successfully
    if status.completion_time.is_some() {
        return true;
    }
    // Failed beyond backoff limit
    let failed = status.failed.unwrap_or(0);
    let backoff_limit = job.spec.as_ref().and_then(|s| s.backoff_limit).unwrap_or(6);
    failed > backoff_limit
}
