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

//! Example: list available IBM Quantum System resources via `ResourceProvider`.
//!
//! # Setup
//!
//! 1. Create a `qrmi_config.json` file:
//!
//! ```json
//! {
//!     "resources": [
//!         {
//!             "name": "ibm_qs_prod",
//!             "type": "ibm-quantum-system",
//!             "is_dynamic": true,
//!             "environment": {
//!                 "QRMI_IBM_QS_ENDPOINT":     "http://localhost:8080",
//!                 "QRMI_IBM_QS_IAM_ENDPOINT": "https://iam.cloud.ibm.com",
//!                 "QRMI_IBM_QS_IAM_APIKEY":   "<your_api_key>",
//!                 "QRMI_IBM_QS_SERVICE_CRN":  "<your_service_crn>"
//!             }
//!         }
//!     ]
//! }
//! ```
//!
//! 2. Run (no filter — list all resources):
//!
//! ```bash
//! cargo run --example qrmi-example-ibm-quantum-system-provider -- \
//!     /path/to/qrmi_config.json ibm_qs_prod
//! ```
//!
//! 3. Run with filters:
//!
//! ```bash
//! cargo run --example qrmi-example-ibm-quantum-system-provider -- \
//!     /path/to/qrmi_config.json ibm_qs_prod -f "num_qubits=127&name=test_*"
//! ```

use clap::Parser;
use qrmi::ibm::IBMQuantumSystemProvider;
use qrmi::models::Config;
use qrmi::ResourceProvider;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI Provider for IBM Quantum System - Example")]
struct Args {
    /// Path to qrmi_config.json
    config_file: String,

    /// Name of the dynamic resource definition
    resource_name: String,

    /// Optional filter string e.g. "num_qubits=127&name=test_*"
    #[arg(short, long)]
    filters: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    match &args.filters {
        Some(f) => println!("Filters: {f}"),
        None => println!("No filters specified — listing all resources."),
    }

    let config = Config::load(&args.config_file)?;
    let resource_def = config
        .resource_map
        .get(&args.resource_name)
        .ok_or_else(|| format!("Resource '{}' not found in config", args.resource_name))?;

    let provider = IBMQuantumSystemProvider::new(resource_def)?;

    let resources = provider.resources(args.filters.clone()).await?;

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

    println!("\nLeast busy resource:");
    match provider.least_busy(args.filters).await? {
        Some(mut r) => println!("  {}", r.resource_id().await?),
        None => println!("  (none)"),
    }

    Ok(())
}
