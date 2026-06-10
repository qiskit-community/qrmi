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

//! Unified provider example.
//!
//! Works with any supported provider type (`qiskit-runtime-service`,
//! `ibm-quantum-system`, etc.). The resource type is read from
//! `qrmi_config.json` — no code changes needed when switching between providers.
//!
//! # Setup
//!
//! Create a `qrmi_config.json` file with an `is_dynamic: true` entry:
//!
//! ```json
//! {
//!     "resources": [
//!         {
//!             "name": "ibm_inst1",
//!             "type": "qiskit-runtime-service",
//!             "is_dynamic": true,
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
//! # Run
//!
//! ```bash
//! cargo run --example qrmi-example-providers -- /path/to/qrmi_config.json ibm_inst1
//!
//! # With filters
//! cargo run --example qrmi-example-providers -- \
//!     /path/to/qrmi_config.json ibm_inst1 -f "num_qubits=127&name=ibm_*"
//! ```

use clap::Parser;
use qrmi::models::Config;
use qrmi::resource_provider::create_provider;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI Provider Example")]
struct Args {
    /// Path to qrmi_config.json
    config_file: String,

    /// Name of the dynamic resource definition (is_dynamic=true)
    resource_name: String,

    /// Optional filter string e.g. "num_qubits=127&name=ibm_*"
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

    // create_provider() selects the correct provider based on resource_def.type.
    let provider = create_provider(&resource_def.r#type, &resource_def.environment)?;

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
