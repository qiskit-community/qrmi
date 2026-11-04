// This code is part of Qiskit.
//
// (C) Copyright IBM, Qoro 2026
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
    maestro::MaestroLocal, models::Payload, models::TaskStatus, QrmiError, QuantumResource,
    ResourceStatusCode,
};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::{thread, time};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI for Maestro Local - Example")]
struct Args {
    /// Backend name (device identifier)
    #[arg(short, long)]
    backend: String,

    /// QASM input file
    #[arg(short, long)]
    input: String,

    /// Job type ('execute' or 'estimate')
    #[arg(short = 'j', long, default_value = "execute")]
    job_type: String,

    /// Number of qubits
    #[arg(short, long, default_value_t = 5)]
    qubits: u32,

    /// Simulator type (0 = aer, 1 = qcsim)
    #[arg(short, long, default_value_t = 0)]
    simulator_type: u32,

    /// Simulation method (0 = statevector, 1 = matrix_product_state)
    #[arg(short = 'm', long, default_value_t = 0)]
    simulation_method: u32,

    /// Observables (pauli strings separated by ";"), required for 'estimate' job type
    #[arg(short, long, default_value = "")]
    observables: String,

    /// Task configuration, in JSON format
    #[arg(short, long, default_value = "{}")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    dotenv().ok();

    let mut qrmi = MaestroLocal::new(&args.backend)?;
    println!(
        "Selected resource: id={} type={}",
        qrmi.resource_id().await?,
        qrmi.resource_type().await?.as_str()
    );

    if qrmi.status().await?.status != ResourceStatusCode::Online {
        println!("Maestro local is not accessible"); // Checks for real QPU
    }

    let lock = qrmi.acquire().await?;
    println!("acquisition token = {}", lock);

    // Set lock as env variable with name  <backend>_QRMI_JOB_ACQUISITION_TOKEN
    let token_var = format!("{}_QRMI_JOB_ACQUISITION_TOKEN", args.backend);
    std::env::set_var(&token_var, &lock);

    println!("{:#?}", qrmi.metadata().await);

    match qrmi.target().await {
        Ok(target) => println!("{}", target.value),
        Err(QrmiError::UnsupportedFunction(_)) => {
            println!("Target information is unavailable for Maestro Local")
        }
        Err(error) => return Err(error.into()),
    }

    let f = File::open(args.input).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let payload = Payload::MaestroLocal {
        input: contents,
        job_type: args.job_type,
        qubits: args.qubits,
        simulator_type: args.simulator_type,
        simulation_method: args.simulation_method,
        observables: args.observables,
        config: args.config,
    };

    let job_id = qrmi.task_start(payload).await?;
    println!("Job ID: {}", job_id);
    let one_sec = time::Duration::from_millis(1000);
    loop {
        let status = qrmi.task_status(&job_id).await?;
        println!("{:?}", status);
        if matches!(status, TaskStatus::Completed) {
            println!("{}", qrmi.task_result(&job_id).await?.value);
            break;
        } else if matches!(status, TaskStatus::Failed | TaskStatus::Cancelled) {
            match qrmi.task_logs(&job_id).await {
                Ok(logs) => println!("{logs}"),
                Err(QrmiError::UnsupportedFunction(_)) => {
                    println!("Task logs are unavailable for Maestro Local")
                }
                Err(error) => return Err(error.into()),
            }
            break;
        }
        thread::sleep(one_sec);
    }
    let _ = qrmi.task_stop(&job_id).await;

    let _ = qrmi.release(&lock).await;
    Ok(())
}
