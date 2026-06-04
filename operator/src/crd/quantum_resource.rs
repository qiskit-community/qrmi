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

//! QuantumResource CRD — cluster-scoped, administrator-managed backend configuration.
//!
//! A QuantumResource names a specific backend (e.g. `ibm_torino`) and its
//! resource type, and collects the environment variables needed to authenticate
//! with it.  Secret values are referenced by name so they are never stored
//! in plaintext inside the CRD itself.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// QRMI resource type, matching the string identifiers used by the existing
/// `qrmi::models::ResourceType`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QrmiResourceType {
    DirectAccess,
    IbmQuantumSystem,
    QiskitRuntimeService,
    PasqalCloud,
    PasqalLocal,
    AliceBobFelis,
    IqmServer,
}

/// Reference to a key inside a Kubernetes Secret that should be surfaced as an
/// environment variable when the resource is acquired.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretEnvRef {
    /// Name of the Kubernetes Secret (in the same namespace as the claim,
    /// or cluster-level for cluster-scoped resources — the operator resolves
    /// this in the operator's own namespace).
    pub secret_name: String,
    /// Key within the Secret's `data` map.
    pub secret_key: String,
    /// Name of the environment variable to expose to the workload.
    pub env_var_name: String,
}

/// QuantumResource is a cluster-scoped custom resource that describes a single
/// quantum backend available through QRMI.
///
/// Administrators create one QuantumResource per backend.  Workloads request
/// access by creating a `QuantumResourceClaim`.
#[derive(CustomResource, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[kube(
    group = "quantum.qrmi.io",
    version = "v1alpha1",
    kind = "QuantumResource",
    namespaced = false,
    shortname = "qr",
    printcolumn = r#"{"name":"Resource ID","type":"string","jsonPath":".spec.resourceId"}"#,
    printcolumn = r#"{"name":"Type","type":"string","jsonPath":".spec.resourceType"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct QuantumResourceSpec {
    /// Backend / resource identifier (e.g. `ibm_torino`, `eagle`).
    pub resource_id: String,
    /// QRMI resource type.
    pub resource_type: QrmiResourceType,
    /// Plain-text environment variables forwarded to the QRMI backend
    /// constructor and injected into the workload Secret on bind.
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    /// Secret-backed environment variables resolved at acquisition time.
    #[serde(default)]
    pub secret_refs: Vec<SecretEnvRef>,
}
