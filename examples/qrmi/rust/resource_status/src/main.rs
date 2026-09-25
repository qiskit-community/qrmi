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

use clap::Parser;
use dotenv::dotenv;
use qrmi::{
    alice_bob::AliceBobFelis, ibm::IBMQiskitRuntimeService, ibm::IBMQuantumComputeService,
    ibm::IBMQuantumSystem, iqm::IQMServer, pasqal::PasqalCloud, pasqal::PasqalLocal,
};
use qrmi::QuantumResource;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI Resource status API - Example")]
struct Args {
    /// resource type
    #[arg(short, long)]
    resource_type: String,

    /// resource id
    #[arg(short, long)]
    resource_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    dotenv().ok();
    println!("{}", dotenv().unwrap().display());

    let mut qrmi: Box<dyn QuantumResource> = match args.resource_type.as_str() {
        "ibm-quantum-system" => Box::new(IBMQuantumSystem::new(&args.resource_id)?),
        "qiskit-runtime-service" => Box::new(IBMQiskitRuntimeService::new(&args.resource_id)?),
        "ibm-quantum-compute-service" => {
            Box::new(IBMQuantumComputeService::new(&args.resource_id)?)
        }
        "pasqal-cloud" => Box::new(PasqalCloud::new(&args.resource_id)?),
        "pasqal-local" => Box::new(PasqalLocal::new(&args.resource_id)?),
        "alice-bob-felis" => Box::new(AliceBobFelis::new(&args.resource_id)?),
        "iqm-server" => Box::new(IQMServer::new(&args.resource_id)?),
        _ => unreachable!("args.resource_type should be validated before this match"),
    };

    println!(
        "Selected resource: id={} type={}",
        qrmi.resource_id().await?,
        qrmi.resource_type().await?.as_str()
    );

    let status = qrmi.status().await?;
    println!("{:#?}", status);
    Ok(())
}
