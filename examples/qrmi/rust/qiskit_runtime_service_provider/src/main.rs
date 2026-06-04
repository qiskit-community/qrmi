// This code is part of Qiskit.
//
// (C) Copyright IBM 2025-2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Example: list available IBM Qiskit Runtime Service resources via `ResourceProvider`.
//!
//! # Setup
//!
//! 1. Create a `qrmi_config.json` file:
//!
//! ```json
//! {
//!     "version": 2,
//!     "providers": [
//!         {
//!             "type": "qiskit-runtime-service",
//!             "environment": {
//!                 "QRMI_IBM_QRS_ENDPOINT":     "https://quantum.cloud.ibm.com/api/v1",
//!                 "QRMI_IBM_QRS_IAM_ENDPOINT": "https://iam.cloud.ibm.com",
//!                 "QRMI_IBM_QRS_IAM_APIKEY":   "<your_api_key>",
//!                 "QRMI_IBM_QRS_SERVICE_CRN":  "<your_service_crn>"
//!             }
//!         }
//!     ]
//! }
//! ```
//!
//! 2. Set the config file path:
//!
//! ```bash
//! export QRMI_RESOURCE_PROVIDER_CONFIG_FILE=/path/to/qrmi_config.json
//! ```
//!
//! 3. Run (no filter — list all resources):
//!
//! ```bash
//! cargo run --example qrmi-example-qiskit-runtime-service-provider
//! ```
//!
//! 4. Run with filters:
//!
//! ```bash
//! # 127+ qubit resources starting with "ibm_"
//! cargo run --example qrmi-example-qiskit-runtime-service-provider -- "num_qubits=127&name=ibm_*"
//!
//! # Online resources only (calls get_backend_status per resource in parallel)
//! cargo run --example qrmi-example-qiskit-runtime-service-provider -- "status=online"
//!
//! # Include simulators (default excludes them)
//! cargo run --example qrmi-example-qiskit-runtime-service-provider -- "is_simulator=true"
//! ```

use qrmi::ibm::IBMQiskitRuntimeServiceProvider;
use qrmi::ResourceProvider;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Optional filter string from the first CLI argument.
    // Example: "num_qubits=127&name=ibm_*"
    let filters: Option<String> = env::args().nth(1);

    match &filters {
        Some(f) => println!("Filters: {f}"),
        None => println!("No filters specified — listing all resources."),
    }

    let provider = IBMQiskitRuntimeServiceProvider::new()?;

    // --- resources() ---
    let resources = provider.resources(filters.clone()).await?;

    if resources.is_empty() {
        println!("No resources found matching the given filters.");
        return Ok(());
    }

    println!("\nAvailable resources ({} found):", resources.len());
    println!("{:-<40}", "");

    for mut r in resources {
        let id = r.resource_id().await?;
        let resource_type = r.resource_type().await?;
        let accessible = r.is_accessible().await.unwrap_or(false);
        println!(
            "  {:<30} type={:<25} accessible={}",
            id,
            resource_type.as_str(),
            accessible,
        );
    }

    // --- least_busy() ---
    println!("\nLeast busy resource:");
    match provider.least_busy(filters).await? {
        Some(mut r) => println!("  {}", r.resource_id().await?),
        None => println!("  (none)"),
    }

    Ok(())
}
