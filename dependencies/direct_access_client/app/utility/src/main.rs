//
// (C) Copyright IBM 2024
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use direct_access_api::utils::s3::S3Client;
use direct_access_api::utils::uuid;

static S3_BUCKET: &str = "test";
static S3_ENDPOINT: &str = "http://localhost:9000";
static S3_REGION: &str = "us-east-1";
static AWS_ACCESS_KEY_ID: &str = "minioadmin";
static AWS_SECRET_ACCESS_KEY: &str = "minioadmin";

fn ask_confirm(question: &str) -> bool {
    loop {
        println!("{}", question);
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input).unwrap();
        input.truncate(input.len() - 1);
        input = input.to_ascii_uppercase();
        if input == "Y" {
            return true;
        } else if input == "N" {
            return false;
        }
    }
}

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uuid = uuid::new_v4();
    println!("UUIDv4: {}", uuid);

    let s3 = S3Client::new(
        S3_ENDPOINT,
        AWS_ACCESS_KEY_ID,
        AWS_SECRET_ACCESS_KEY,
        S3_REGION,
    );
    let s3_object_key = format!("{}.txt", uuid);

    let get_url = s3
        .get_presigned_url_for_get(S3_BUCKET, &s3_object_key, 86400)
        .await?;
    println!("presigned url for GetObject: {}", get_url);
    let put_url = s3
        .get_presigned_url_for_put(S3_BUCKET, &s3_object_key, 86400)
        .await?;
    println!("presigned url for PutObject: {}", put_url);

    let content = String::from("Hello, World.");
    s3.put_object(S3_BUCKET, &s3_object_key, &content.into_bytes())
        .await?;
    println!("PutObject: {}", s3_object_key);

    let retrieved = s3.get_object(S3_BUCKET, &s3_object_key).await?;
    let retrieved_txt = String::from_utf8(retrieved).unwrap();
    println!("GetObject: {}", retrieved_txt);

    println!("Listing objects in a bucket");
    let objects = s3.list_objects(S3_BUCKET).await?;
    for obj in &objects {
        println!("{}", obj);
    }

    let yes_no = ask_confirm("Are you sure to delete all objects? [y/n]");
    if yes_no {
        println!("Deleting objects in a bucket");
        for obj in &objects {
            s3.delete_object(S3_BUCKET, obj.clone()).await?;
        }
    }
    Ok(())
}
