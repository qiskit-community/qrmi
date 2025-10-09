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

mod common;
use direct_access_api::ClientBuilder;
use serde_json::json;
use uuid::Uuid;

/// Test Client.run_job().
#[tokio::test]
async fn test_run_job() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let id = Uuid::new_v4().to_string();

    let payload = json!({
        "id": id,
        "backend": "test_backend",
        "program_id": "estimator",
        "log_level": "info",
        "timeout_secs": 10000,
        "storage": {
            "input": {
                "presigned_url": "http://localhost:9000/test/params_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=aB7R0W5XLlo0iwd3yUCY6F2XvVg%3D&Expires=1730043158",
                "type": "s3_compatible"
            },
            "results": {
                "presigned_url": "http://localhost:9000/test/results_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=MABxhJ2gV6RvWin6llS64jZwY2M%3D&Expires=1730043158",
                "type": "s3_compatible"
            },
            "logs": {
                "presigned_url": "http://localhost:9000/test/logs_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=bH3QxADk5r2ojcaXqs46VKfjM1s%3D&Expires=1730043158",
                "type": "s3_compatible"
            }
        }
    });

    server
        .mock("POST", "/v1/jobs")
        .with_status(204)
        .with_header("content-type", "application/json")
        .match_body(format!(r#"{}"#, payload).as_str())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let result = client.run_job(&payload).await.unwrap();
    assert_eq!(id, result);
}

/// Test Client.run_job() with error.
#[tokio::test]
async fn test_run_job_failed() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let id = Uuid::new_v4().to_string();

    let payload = json!({
        "id": id,
        "backend": "test_backend",
        "program_id": "estimator",
        "log_level": "info",
        "timeout_secs": 10000,
        "storage": {
            "input": {
                "presigned_url": "http://localhost:9000/test/params_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=aB7R0W5XLlo0iwd3yUCY6F2XvVg%3D&Expires=1730043158",
                "type": "s3_compatible"
            },
            "results": {
                "presigned_url": "http://localhost:9000/test/results_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=MABxhJ2gV6RvWin6llS64jZwY2M%3D&Expires=1730043158",
                "type": "s3_compatible"
            },
            "logs": {
                "presigned_url": "http://localhost:9000/test/logs_40f48592-45ab-475d-97d7-f264c638b236?AWSAccessKeyId=minio&Signature=bH3QxADk5r2ojcaXqs46VKfjM1s%3D&Expires=1730043158",
                "type": "s3_compatible"
            }
        }
    });

    let error = json!({
        "status_code": 423,
        "title": "The requested backend is currently reserved and cannot run jobs outside of the reservation.",
        "trace": Uuid::new_v4().to_string(),
        "correlation_id": common::generate_random_string(20),
        "errors": [
            {
                "code": "1233",
                "message": "The requested backend is currently reserved and cannot run jobs outside of the reservation.",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling"
            }
        ]
    });

    server
        .mock("POST", "/v1/jobs")
        .with_status(423)
        .with_header("content-type", "application/json")
        .with_body(error.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let result = client.run_job(&payload).await;
    assert!(result.is_err());

    let _ = result.map_err(|e| {
        println!("{}", e);
    });
}
