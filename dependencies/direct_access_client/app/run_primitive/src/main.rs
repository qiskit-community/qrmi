//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use clap::Parser;
use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use std::time::Duration;

use direct_access_api::{AuthMethod, ClientBuilder};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "Direct Access API client library - Example")]
struct Args {
    /// backend name
    #[arg(short, long)]
    backend_name: String,

    /// job input file
    #[arg(short, long)]
    job: String,

    /// program id
    #[arg(short, long)]
    program_id: String,

    /// logging level
    #[arg(short, long)]
    log_level: String,
}

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    env_logger::init();

    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
        .jitter(Jitter::Bounded)
        .base(2)
        .build_with_max_retries(5);

    let client = ClientBuilder::new("http://0.0.0.0:8290")
        .with_auth(AuthMethod::IbmCloudIam {
            apikey: "demoapikey1".to_string(),
            service_crn: "crn:v1:local:daa_sim".to_string(),
            iam_endpoint_url: "http://0.0.0.0:8290".to_string(),
        })
        //.with_auth(AuthMethod::IbmCloudAppId {
        //    username: "demo".to_string(),
        //    password: "demopass".to_string(),
        //})
        .with_timeout(Duration::from_secs(60))
        .with_retry_policy(retry_policy)
        .with_s3bucket(
            "minioadmin",
            "minioadmin",
            "http://localhost:9000",
            "test",
            "us-east-1",
            None::<String>,
        )
        .build()
        .unwrap();

    let f = File::open(args.job).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let job: serde_json::Value = serde_json::from_str(contents.as_str()).unwrap();

    let primitive_job = client
        .run_primitive(
            &args.backend_name,
            args.program_id.parse().unwrap(),
            86400,
            args.log_level.parse().unwrap(),
            &job,
            None,
        )
        .await?;
    println!("Running a job: {}", primitive_job.job_id);
    match primitive_job.wait_for_final_state(Some(1800.0)).await {
        Ok(retval) => {
            println!("{}", serde_json::to_string_pretty(&retval).unwrap());
        }
        Err(e) => {
            println!(
                "Error occurred while waiting for final state: {:?}",
                e.to_string()
            );
            primitive_job.cancel(false).await?;
            println!("cancelled a job: {}", primitive_job.job_id);
        }
    }

    println!("fetching result");
    match primitive_job.get_result::<serde_json::Value>().await {
        Ok(retval) => {
            println!("{}", serde_json::to_string_pretty(&retval).unwrap());
        }
        Err(e) => {
            println!(
                "Error occurred while fetching job result from S3 bucket: {:?}",
                e.to_string()
            );
        }
    }

    println!("fetching logs");
    match primitive_job.get_logs().await {
        Ok(retval) => {
            println!("{}", retval);
        }
        Err(e) => {
            println!(
                "Error occurred while fetching job logs from S3 bucket: {:?}",
                e.to_string()
            );
        }
    }

    client.delete_job(&primitive_job.job_id).await?;
    println!("deleted a job: {}", primitive_job.job_id);

    Ok(())
}
