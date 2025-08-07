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
use serde_json::json;
use std::time::Duration;

use direct_access_api::utils::{s3::S3Client, uuid};
use direct_access_api::{AuthMethod, ClientBuilder};

static S3_BUCKET: &str = "test";
static S3_ENDPOINT: &str = "http://localhost:9000";
static S3_REGION: &str = "us-east-1";
static S3_EXPIRES_IN: u64 = 86400;
static AWS_ACCESS_KEY_ID: &str = "minioadmin";
static AWS_SECRET_ACCESS_KEY: &str = "minioadmin";

static DA_STORAGE_TYPE: &str = "s3_compatible";
static DA_TIMEOUT: u64 = 3600;

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
        .build()
        .unwrap();

    let f = File::open(args.job).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;

    let id = uuid::new_v4();

    let input_key = format!("input-{}.json", id);
    let results_key = format!("results-{}.json", id);
    let logs_key = format!("logs-{}.json", id);

    let s3 = S3Client::new(
        S3_ENDPOINT,
        AWS_ACCESS_KEY_ID,
        AWS_SECRET_ACCESS_KEY,
        S3_REGION,
    );
    s3.put_object(S3_BUCKET, &input_key, &contents.into_bytes())
        .await?;

    let input_url = s3
        .get_presigned_url_for_get(S3_BUCKET, &input_key, S3_EXPIRES_IN)
        .await?;
    let results_url = s3
        .get_presigned_url_for_put(S3_BUCKET, &results_key, S3_EXPIRES_IN)
        .await?;
    let logs_url = s3
        .get_presigned_url_for_put(S3_BUCKET, &logs_key, S3_EXPIRES_IN)
        .await?;

    let job = json!({
        "id": id,
        "backend": args.backend_name.parse::<String>().unwrap(),
        "program_id": args.program_id.parse::<String>().unwrap(),
        "log_level": args.log_level.parse::<String>().unwrap(),
        "timeout_secs": DA_TIMEOUT,
        "storage": {
            "input": {
                "presigned_url": input_url,
                "type": DA_STORAGE_TYPE,
            },
            "results": {
                "presigned_url": results_url,
                "type": DA_STORAGE_TYPE,
            },
            "logs": {
                "presigned_url": logs_url,
                "type": DA_STORAGE_TYPE,
            },
        }
    });

    let job_id = client.run_job(&job).await?;
    println!("Running a job: {}", job_id);
    match client.wait_for_job_final_state(&id, Some(1800.0)).await {
        Ok(retval) => {
            println!("{}", serde_json::to_string_pretty(&retval).unwrap());
        }
        Err(e) => {
            println!(
                "Error occurred while waiting for final state: {:?}",
                e.to_string()
            );
            client.cancel_job(&id, false).await?;
            println!("cancelled a job: {}", id);
        }
    }

    client.delete_job(&id).await?;
    println!("deleted a job: {}", id);

    let results = s3.get_object(S3_BUCKET, &results_key).await?;
    let results_str = String::from_utf8(results).unwrap();
    let results_json: serde_json::Value = serde_json::from_str(&results_str).unwrap();
    println!("results -- ");
    println!("{}", serde_json::to_string_pretty(&results_json).unwrap());

    let logs = s3.get_object(S3_BUCKET, &logs_key).await?;
    let logs_str = String::from_utf8(logs).unwrap();
    println!("logs -- ");
    println!("{}", logs_str);
    Ok(())
}
