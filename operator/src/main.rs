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

//! QRMI Kubernetes Operator
//!
//! Runs three concurrent controllers inside a single binary:
//!
//! - **QuantumResourceClaim controller** — manages the QRMI acquire/release
//!   lifecycle and the associated credential Secret.
//! - **Job controller** — watches annotated Jobs, suspends them until a claim
//!   is bound, then injects the Secret via `envFrom` and unsuspends.
//!
//! Additionally, the `generate-crds` subcommand writes CRD YAML to stdout for
//! use in `deploy/crds/`.

use std::sync::Arc;

use anyhow::Result;
use rustls;
use futures::future::join_all;
use futures::StreamExt;
use k8s_openapi::api::batch::v1::Job;
use kube::runtime::watcher::Config as WatcherConfig;
use kube::runtime::Controller;
use kube::{Api, Client, CustomResourceExt};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod acquire;
mod controllers;
mod crd;
mod error;

use crd::{QuantumResource, QuantumResourceClaim};

#[tokio::main]
async fn main() -> Result<()> {
    // rustls requires an explicit crypto provider when multiple are compiled in.
    // Install ring before any TLS connections are made (e.g. kube Client::try_default).
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install rustls ring crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new("qrmi_operator=info,kube_client=error,kube_runtime=error,tower=error,hyper=error,warn")
            }),
        )
        .with_target(false)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("generate-crds") {
        return generate_crds();
    }

    run_operator().await
}

async fn run_operator() -> Result<()> {
    let client = Client::try_default().await?;
    info!("connected to Kubernetes API server");

    // --- QuantumResourceClaim controller ---
    let claim_ctx = Arc::new(controllers::resource_claim::Context {
        client: client.clone(),
    });
    let claims_api: Api<QuantumResourceClaim> = Api::all(client.clone());
    let claim_controller = Controller::new(claims_api, WatcherConfig::default())
        .run(
            controllers::resource_claim::reconcile,
            controllers::resource_claim::error_policy,
            claim_ctx,
        )
        .for_each(|res| async move {
            match res {
                Ok((obj, action)) => {
                    tracing::debug!("claim reconciled: {} → {action:?}", obj.name)
                }
                Err(e) => tracing::warn!("claim reconcile error: {e}"),
            }
        });

    // --- Job controller ---
    // Watches all Jobs; reconciler skips those without the annotation.
    // Note: Kubernetes watch/label selectors cannot filter on annotations,
    // so we must watch all jobs and filter inside the reconcile function.
    let job_ctx = Arc::new(controllers::job::Context {
        client: client.clone(),
    });
    let jobs_api: Api<Job> = Api::all(client.clone());
    let job_controller = Controller::new(jobs_api, WatcherConfig::default())
        .run(
            controllers::job::reconcile,
            controllers::job::error_policy,
            job_ctx,
        )
        .for_each(|res| async move {
            match res {
                Ok((obj, action)) => {
                    tracing::debug!(job = %obj.name, "reconciled → {action:?}")
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("not found in local store") {
                        tracing::debug!("job reconcile skipped (object deleted before cache update): {msg}");
                    } else {
                        tracing::warn!("job reconcile error: {e}");
                    }
                }
            }
        });

    info!("all controllers started");

    join_all(vec![
        tokio::spawn(claim_controller),
        tokio::spawn(job_controller),
    ])
    .await;

    Ok(())
}

/// Write CRD YAML for all custom resources to stdout.
fn generate_crds() -> Result<()> {
    let crds = vec![
        serde_yaml::to_string(&QuantumResource::crd())?,
        serde_yaml::to_string(&QuantumResourceClaim::crd())?,
    ];
    for crd in crds {
        println!("---");
        print!("{crd}");
    }
    Ok(())
}
