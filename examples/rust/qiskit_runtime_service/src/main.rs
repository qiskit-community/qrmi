// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
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
use qrmi::{ibm::IBMQiskitRuntimeService, models::Payload, models::TaskStatus, QuantumResource};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::{thread, time};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRMI for IBM Qiskit Runtime Service - Example")]
struct Args {
    /// primitive input file
    #[arg(short, long)]
    input: String,

    /// program id
    #[arg(short, long)]
    program_id: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    dotenv().ok();
    println!("{}", dotenv().unwrap().display());

    let backend_name = env::var("QRMI_RESOURCE_ID").expect("QRMI_RESOURCE_ID");

    let mut qrmi = IBMQiskitRuntimeService::default();

    let accessible = qrmi.is_accessible(&backend_name);
    if !accessible {
        panic!("{} is not accessible", backend_name);
    }

    let lock = qrmi.acquire(&backend_name).unwrap();

    println!("{:#?}", qrmi.metadata());

    let target = qrmi.target(&backend_name);
    if let Ok(v) = target {
        println!("{}", v.value);
    }

    let f = File::open(args.input).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let payload = Payload::QiskitPrimitive {
        input: contents,
        program_id: args.program_id,
    };

    let job_id = qrmi.task_start(payload).unwrap();
    println!("Job ID: {}", job_id);
    let one_sec = time::Duration::from_millis(1000);
    loop {
        let status = qrmi.task_status(&job_id).unwrap();
        println!("{:?}", status);
        if matches!(status, TaskStatus::Completed) {
            println!("{}", qrmi.task_result(&job_id).unwrap().value);
            break;
        } else if matches!(status, TaskStatus::Failed | TaskStatus::Cancelled) {
            break;
        }
        thread::sleep(one_sec);
    }
    let _ = qrmi.task_stop(&job_id);

    let _ = qrmi.release(&lock);
    Ok(())
}
