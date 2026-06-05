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
//!     "version": 2,
//!     "providers": [
//!         {
//!             "type": "ibm-quantum-system",
//!             "environment": {
//!                 "QRMI_IBM_QS_ENDPOINT": "http://localhost:8080",
//!                 "QRMI_IBM_QS_IAM_ENDPOINT": "https://iam.test.cloud.ibm.com",
//!                 "QRMI_IBM_QS_IAM_APIKEY": "<YOUR IAM APIKEY FOR THIS BACKEND>",
//!                 "QRMI_IBM_QS_SERVICE_CRN": "<YOUR DIRECT ACCESS INSTANCE CRN>",
//!                 "QRMI_IBM_QS_AWS_ACCESS_KEY_ID": "<YOUR AWS ACCESS KEY TO ACCESS S3 BUCKET>",
//!                 "QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY": "<YOUR AWS SECRET ACCESS KEY TO ACCESS S3 BUCKET>",
//!                 "QRMI_IBM_QS_S3_ENDPOINT": "https://s3.us-east.cloud-object-storage.appdomain.cloud",
//!                 "QRMI_IBM_QS_S3_BUCKET": "<YOUR S3 BUCKET NAME>",
//!                 "QRMI_IBM_QS_S3_REGION": "us-east"
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
//! cargo run --example qrmi-example-ibm-quantum-system-provider
//! ```
//!
//! 4. Run with filters:
//!
//! ```bash
//! # 127+ qubit resources starting with "ibm_"
//! cargo run --example qrmi-example-ibm-quantum-system-provider -- "-f num_qubits=127&name=ibm_*"
//!
//! # Online resources only (calls get_backend_status per resource in parallel)
//! cargo run --example qrmi-example-ibm-quantum-system-provider -- "-f status=online"
//!
//! # Include simulators (default excludes them)
//! cargo run --example qrmi-example-ibm-quantum-system-provider -- "-f is_simulator=true"
//! ```

use clap::Parser;
use qrmi::ibm::IBMQuantumSystemProvider;
use qrmi::ResourceProvider;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI Provider for IBM Quantum System - Example")]
struct Args {
    /// A filter specification using comma-separated key-value pairs
    #[arg(short, long)]
    filters: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Optional filter string from the first CLI argument.
    // Example: "num_qubits=127&name=ibm_*"
    let args = Args::parse();

    match &args.filters {
        Some(f) => println!("Filters: {f}"),
        None => println!("No filters specified — listing all resources."),
    }

    let provider = IBMQuantumSystemProvider::new()?;

    // --- resources() ---
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

    // --- least_busy() ---
    println!("\nLeast busy resource:");
    match provider.least_busy(args.filters).await? {
        Some(mut r) => println!("  {}", r.resource_id().await?),
        None => println!("  (none)"),
    }

    Ok(())
}
