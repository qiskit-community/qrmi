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

use clap::Parser;
use dotenv::dotenv;
use qrmi::{iqm::IQMServer, models::Payload, models::TaskStatus, QuantumResource};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::{thread, time};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI for IQM Server - Example")]
struct Args {
    /// QC alias name
    #[arg(short, long)]
    qc_alias: String,

    /// IQM JSON file
    #[arg(short, long)]
    iqmjson: String,

    /// Job type('circuit','run', or 'sweep')
    #[arg(short, long)]
    job_type: String,

    /// use_timeslot
    #[arg(short, long)]
    use_timeslot: Option<bool>,

    /// tag
    #[arg(short, long)]
    tag: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    dotenv().ok();
    println!("{}", dotenv().unwrap().display());

    let mut qrmi = IQMServer::new(&args.qc_alias)?;
    println!(
        "Selected resource: id={} type={}",
        qrmi.resource_id().await?,
        qrmi.resource_type().await?.as_str()
    );

    let accessible = qrmi.is_accessible().await?;
    if !accessible {
        panic!("{} is not accessible", args.qc_alias);
    }

    let lock = qrmi.acquire().await?;

    println!("{:#?}", qrmi.metadata().await);

    let target = qrmi.target().await;
    if let Ok(v) = target {
        println!("{}", v.value);
    }

    let f = File::open(args.iqmjson).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let payload = Payload::IQMServer {
        iqmjson: contents,
        job_type: args.job_type,
        use_timeslot: args.use_timeslot,
        tag: args.tag,
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
            println!("{}", qrmi.task_logs(&job_id).await?);
            break;
        }
        thread::sleep(one_sec);
    }
    let _ = qrmi.task_stop(&job_id).await;

    let _ = qrmi.release(&lock).await;
    Ok(())
}
