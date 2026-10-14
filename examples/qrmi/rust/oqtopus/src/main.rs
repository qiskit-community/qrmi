// (C) Copyright 2026 IBM
// (C) Copyright 2026 The University of Osaka
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
use qrmi::{models::Payload, models::TaskStatus, oqtopus::Oqtopus, QuantumResource};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::{thread, time};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI for OQTOPUS - Example")]
struct Args {
    /// device ID
    #[arg(short, long)]
    device_id: String,

    /// Job spec JSON file
    #[arg(short, long)]
    job_spec: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    dotenv().ok();
    println!("{}", dotenv().unwrap().display());

    let mut qrmi = Oqtopus::new(&args.device_id)?;
    println!(
        "Selected resource: id={} type={}",
        qrmi.resource_id().await?,
        qrmi.resource_type().await?.as_str()
    );

    let accessible = matches!(
        qrmi.status().await?.status,
        qrmi::models::ResourceStatusCode::Online
    );
    if !accessible {
        println!("{} is not accessible", args.device_id);
    }

    let lock = qrmi.acquire().await?;

    println!("{:#?}", qrmi.describe().await?);

    let target = qrmi.target().await;
    if let Ok(v) = target {
        println!("{}", v.value);
    }

    let f = File::open(args.job_spec).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let payload = Payload::Oqtopus { job_spec: contents };

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
            break;
        }
        thread::sleep(one_sec);
    }
    let _ = qrmi.release(&lock).await;
    Ok(())
}
