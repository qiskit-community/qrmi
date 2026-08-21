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

//! `QRMIService` example.
//!
//! Unlike the other examples in `examples/qrmi/rust`, which each construct a
//! single, specific `QuantumResource` implementation directly (e.g.
//! `IBMQuantumSystem::new("ibm_torino")`), this example uses `QRMIService`,
//! which discovers *all* of the QPU resources assigned to the current job
//! from the environment -- the same environment variables a Slurm QRMI
//! plugin would set -- and exposes the ones that are currently accessible.
//!
//! # Setup
//!
//! `QRMIService` reads `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES`
//! (falling back to the legacy `SLURM_JOB_QPU_RESOURCES` /
//! `SLURM_JOB_QPU_TYPES`), each a delimiter-separated list (delimiter: `,`
//! by default, overridable via `QRMI_LIST_DELIMITER`). The two lists must
//! be the same length and pair up positionally, e.g.:
//!
//! ```shell-session
//! export QRMI_JOB_QPU_RESOURCES=ibm_torino,my_pasqal_qpu
//! export QRMI_JOB_QPU_TYPES=qiskit-runtime-service,pasqal-cloud
//! ```
//!
//! Each resource named above also needs its own vendor-specific environment
//! variables set (see the other examples in this directory, e.g.
//! `../qiskit_runtime_service` or `../pasqal_cloud`, for what those are per
//! vendor). This example assumes a `.env` file with all of the above is
//! available in the current directory.
//!
//! # Run
//!
//! ```shell-session
//! # List every accessible resource assigned to this job.
//! cargo run --example qrmi-example-service
//!
//! # Acquire one specific resource by id, print its metadata, then release it.
//! cargo run --example qrmi-example-service -- --resource ibm_torino
//! ```

use clap::Parser;
use dotenv::dotenv;
use qrmi::QRMIService;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMIService - Example")]
struct Args {
    /// Resource identifier to acquire, e.g. a backend name. If omitted,
    /// this just lists every accessible resource assigned to the job.
    #[arg(short, long)]
    resource: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    dotenv().ok();

    // Discovers and filters the job's QPU resources. See this binary's
    // module docs (`cargo doc --open`, or the comment at the top of this
    // file) for the environment variables this reads.
    let mut service = QRMIService::new().await?;

    let resources = service.resources();
    if resources.is_empty() {
        println!("No accessible resources found for this job.");
        return Ok(());
    }

    println!("Accessible resources ({} found):", resources.len());
    println!("{:-<40}", "");
    for r in resources {
        println!(
            "  {:<30} type={}",
            r.resource_id().await?,
            r.resource_type().await?.as_str(),
        );
    }

    let Some(resource_id) = args.resource else {
        return Ok(());
    };

    let Some(resource) = service.resource(&resource_id) else {
        return Err(
            format!("'{resource_id}' was not found among this job's accessible resources").into(),
        );
    };

    println!("\nAcquiring '{resource_id}'...");
    let lock = resource.acquire().await?;
    println!("acquisition token = {lock}");

    println!("{:#?}", resource.metadata().await);
    if let Ok(target) = resource.target().await {
        println!("{}", target.value);
    }

    resource.release(&lock).await?;
    println!("Released '{resource_id}'.");

    Ok(())
}
