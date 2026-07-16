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

//! QuantumResourceClaim CRD — namespace-scoped workload request for a quantum backend.
//!
//! A workload (Job or Pod) creates a QuantumResourceClaim referencing a
//! [`QuantumResource`].  The operator acquires the QRMI lock, stores credentials
//! in a managed Secret, and sets the claim's status to `Bound`.  When the claim
//! is deleted (or its TTL expires) the operator releases the lock.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Lifecycle phase of a QuantumResourceClaim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub enum ClaimPhase {
    /// Waiting for the QRMI lock to be acquired.
    #[default]
    Pending,
    /// Lock held; the bound Secret is ready.
    Bound,
    /// Lock released; the claim is being cleaned up.
    Released,
    /// A permanent error occurred; manual intervention required.
    Failed,
}

/// Status sub-resource for [`QuantumResourceClaim`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct QuantumResourceClaimStatus {
    /// Current lifecycle phase.
    pub phase: ClaimPhase,
    /// Name of the managed Secret containing the injected environment variables.
    /// Set when `phase == Bound`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_name: Option<String>,
    /// When the claim transitioned to `Bound`.
    #[serde(skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp when the claim was bound.
    pub bound_at: Option<String>,
    /// When the claim will be automatically released (derived from `spec.ttl`).
    #[serde(skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp when the claim expires (based on TTL).
    pub expires_at: Option<String>,
    /// Human-readable message, populated on `Failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// QuantumResourceClaim is a namespace-scoped resource through which a workload
/// requests exclusive access to a [`QuantumResource`].
///
/// The operator reconciles the claim lifecycle:
/// - Creates a managed Secret with backend credentials once the QRMI lock is acquired.
/// - Deletes the Secret and releases the lock when the claim is deleted or expires.
#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "quantum.qrmi.io",
    version = "v1alpha1",
    kind = "QuantumResourceClaim",
    namespaced = true,
    shortname = "qrc",
    status = "QuantumResourceClaimStatus",
    printcolumn = r#"{"name":"Resource","type":"string","jsonPath":".spec.quantumResource"}"#,
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Secret","type":"string","jsonPath":".status.secretName"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct QuantumResourceClaimSpec {
    /// Name of the `QuantumResource` (cluster-scoped) to claim.
    pub quantum_resource: String,
    /// Optional name for the managed Secret.  Defaults to `qrc-<claim-name>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_name: Option<String>,
    /// Optional time-to-live in seconds.  When elapsed the operator releases
    /// the claim automatically, even if the owning workload is still running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
}
