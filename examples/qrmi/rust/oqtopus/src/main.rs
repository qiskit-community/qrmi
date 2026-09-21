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
use qrmi::{oqtopus::Oqtopus, models::Payload, models::TaskStatus, QuantumResource};
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

    let accessible = qrmi.is_accessible().await?;
    if !accessible {
        panic!("{} is not accessible", args.device_id);
    }

    Ok(())
}
