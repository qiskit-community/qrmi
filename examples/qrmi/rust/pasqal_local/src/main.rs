// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025
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
use qrmi::{models::Payload, models::TaskStatus, pasqal::PasqalLocal, QuantumResource};
use pasqal_local_api::{Client, ClientBuilder};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::{thread, time};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI for Pasqal Local - Example")]
struct Args {
    /// primitive input file
    #[arg(short, long)]
    input: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    // dotenv().ok();
    // println!("{}", dotenv().unwrap().display());

    let client = ClientBuilder::new().build().unwrap();

    

    let jobs = client.get_jobs().await?;
    for job in &jobs {
        println!("{}", job.id);
    }

    let mut qrmi = PasqalLocal::new()?;

    let payload = Payload::PasqalCloud {
                sequence: args.input,
                job_runs: 100,
            };

    let new_job = qrmi.task_start(payload).await?;
    println!("new job = {}", new_job);

    Ok(())
}
