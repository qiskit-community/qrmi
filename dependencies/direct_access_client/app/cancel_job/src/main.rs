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

use clap::Parser;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::Jitter;
use std::time::Duration;

use direct_access_api::{AuthMethod, ClientBuilder};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "Direct Access API client library - Example")]
struct Args {
    /// ID of job to be cancelled
    #[arg(short, long)]
    job_id: String,

    /// true if delete a job after cancelled.
    #[arg(short, long)]
    delete_job: bool,
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
        .build()
        .unwrap();

    client.cancel_job(&args.job_id, args.delete_job).await?;
    if args.delete_job {
        println!("cancelled & deleted a job: {}", args.job_id);
    } else {
        println!("cancelled a job: {}", args.job_id);
    }

    Ok(())
}
