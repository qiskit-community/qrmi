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

mod common;
use assert_json_diff::assert_json_include;
use direct_access_api::{
    models::Job, models::JobStatus, models::Jobs, models::ProgramId, ClientBuilder,
};
use serde_json::json;

/// Test Client.list_jobs().
/// This test will compare the deserialized values in Jobs object with expected values.
/// All comparisons should be succeeded.
#[tokio::test]
async fn test_list_jobs() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "jobs": [
            {
                "id": "77b65b44-07bc-4557-80e2-7f73a6be8ea4",
                "program_id": "sampler",
                "backend": "backend_1",
                "timeout_secs": 10000,
                "storage": {
                    "input": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/input-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                    "results": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/results-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                    "logs": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/logs-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                },
                "status": "Completed",
                "created_time": "2024-11-05T17:21:58.011168Z",
                "end_time": "2024-11-05T17:22:37.439083Z",
                "usage": {
                    "quantum_nanoseconds": 4902824512i64,
                },
            },
            {
                "id": "d7f06eda-ddfe-412e-a94b-747da412a955",
                "program_id": "estimator",
                "backend": "backend_2",
                "timeout_secs": 10000,
                "storage": {
                    "input": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/input-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                    "results": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/results-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                    "logs": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/logs-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                },
                "status": "Failed",
                "reason_message": "Reason why this job was failed.",
                "reason_code": 1517,
                "created_time": "2024-11-04T20:47:34.46209Z",
                "end_time": "2024-11-04T20:47:38.203455Z",
                "usage": {},
            },
        ],
    });

    server
        .mock("GET", "/v1/jobs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client.list_jobs::<serde_json::Value>().await.unwrap();
    assert_json_include!(actual: actual, expected: expected);

    let actual_jobs = client.list_jobs::<Jobs>().await.unwrap();
    let jobs = actual_jobs.jobs;
    assert_eq!("77b65b44-07bc-4557-80e2-7f73a6be8ea4", jobs[0].id);
    assert_eq!(ProgramId::Sampler, jobs[0].program_id);
    assert_eq!("backend_1", jobs[0].backend);
    assert_eq!(JobStatus::Completed, jobs[0].status);
    assert!(jobs[0].usage.quantum_nanoseconds.is_some());

    assert_eq!("d7f06eda-ddfe-412e-a94b-747da412a955", jobs[1].id);
    assert_eq!(ProgramId::Estimator, jobs[1].program_id);
    assert_eq!("backend_2", jobs[1].backend);
    assert_eq!(JobStatus::Failed, jobs[1].status);
    assert!(jobs[1].usage.quantum_nanoseconds.is_none());
}

/// Test Client.get_job().
/// This test will compare the deserialized values in Jobs object with expected values.
/// All comparisons should be succeeded.
#[tokio::test]
async fn test_get_job() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "jobs": [
            {
                "id": "77b65b44-07bc-4557-80e2-7f73a6be8ea4",
                "program_id": "sampler",
                "backend": "backend_1",
                "timeout_secs": 10000,
                "storage": {
                    "input": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/input-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                    "results": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/results-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                    "logs": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/logs-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                },
                "status": "Completed",
                "created_time": "2024-11-05T17:21:58.011168Z",
                "end_time": "2024-11-05T17:22:37.439083Z",
                "usage": {
                    "quantum_nanoseconds": 4902824512i64,
                },
            },
            {
                "id": "d7f06eda-ddfe-412e-a94b-747da412a955",
                "program_id": "estimator",
                "backend": "backend_2",
                "timeout_secs": 10000,
                "storage": {
                    "input": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/input-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                    "results": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/results-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                    "logs": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/logs-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                },
                "status": "Failed",
                "reason_message": "Reason why this job was failed.",
                "reason_code": 1517,
                "created_time": "2024-11-04T20:47:34.46209Z",
                "end_time": "2024-11-04T20:47:38.203455Z",
                "usage": {},
            },
        ],
    });

    server
        .mock("GET", "/v1/jobs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    // get a job with deserializing to JSON object. All comparison should be succeeded.
    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let actual = client
        .get_job::<serde_json::Value>("77b65b44-07bc-4557-80e2-7f73a6be8ea4")
        .await
        .unwrap();
    let mut expected_job = json!({});
    if let Some(jobs) = expected["jobs"].as_array() {
        for job in jobs {
            if job["id"] == "77b65b44-07bc-4557-80e2-7f73a6be8ea4" {
                expected_job = job.clone();
            }
        }
    }
    assert_json_include!(actual: actual, expected: expected_job);

    // get a job with deserializing to Job object. All comparison should be succeeded.
    let actual_job = client
        .get_job::<Job>("77b65b44-07bc-4557-80e2-7f73a6be8ea4")
        .await
        .unwrap();
    assert_eq!("77b65b44-07bc-4557-80e2-7f73a6be8ea4", actual_job.id);
    assert_eq!(ProgramId::Sampler, actual_job.program_id);
    assert_eq!("backend_1", actual_job.backend);
    assert_eq!(JobStatus::Completed, actual_job.status);
    assert!(actual_job.usage.quantum_nanoseconds.is_some());

    // get a job with unknown job id. Client.get_job() should be failed with error message.
    let failed = client.get_job::<serde_json::Value>("unknown").await;
    assert!(failed.is_err());
}

/// Test Client.get_job_status().
#[tokio::test]
async fn test_get_job_status() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    let expected = json!({
        "jobs": [
            {
                "id": "77b65b44-07bc-4557-80e2-7f73a6be8ea4",
                "program_id": "sampler",
                "backend": "backend_1",
                "timeout_secs": 10000,
                "storage": {
                    "input": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/input-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                    "results": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/results-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                    "logs": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/logs-77b65b44-07bc-4557-80e2-7f73a6be8ea4"
                    },
                },
                "status": "Completed",
                "created_time": "2024-11-05T17:21:58.011168Z",
                "end_time": "2024-11-05T17:22:37.439083Z",
                "usage": {
                    "quantum_nanoseconds": 4902824512i64,
                },
            },
            {
                "id": "d7f06eda-ddfe-412e-a94b-747da412a955",
                "program_id": "estimator",
                "backend": "backend_2",
                "timeout_secs": 10000,
                "storage": {
                    "input": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/input-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                    "results": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/results-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                    "logs": {
                        "type": "s3_compatible",
                        "presigned_url": "https://s3endpoint/s3bucket/logs-d7f06eda-ddfe-412e-a94b-747da412a955"
                    },
                },
                "status": "Failed",
                "reason_message": "Reason why this job was failed.",
                "reason_code": 1517,
                "created_time": "2024-11-04T20:47:34.46209Z",
                "end_time": "2024-11-04T20:47:38.203455Z",
                "usage": {},
            },
        ],
    });

    server
        .mock("GET", "/v1/jobs")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(expected.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();

    let mut actual = client
        .get_job_status("77b65b44-07bc-4557-80e2-7f73a6be8ea4")
        .await
        .unwrap();
    assert_eq!(JobStatus::Completed, actual);

    actual = client
        .get_job_status("d7f06eda-ddfe-412e-a94b-747da412a955")
        .await
        .unwrap();
    assert_eq!(JobStatus::Failed, actual);

    // get a job status by unknown job id. Client.get_job_status() should be failed with error message.
    let failed = client.get_job_status("unknown").await;
    assert!(failed.is_err());
}

/// Test Client.cancel_job().
#[tokio::test]
async fn test_cancel_job() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    server
        .mock(
            "POST",
            "/v1/jobs/77b65b44-07bc-4557-80e2-7f73a6be8ea4/cancel",
        )
        .with_status(204)
        .with_header("content-type", "application/json")
        .create_async()
        .await;

    let not_found = json!({
        "status_code": 404,
        "title": "Job not found. Job ID: unknown",
        "trace": "",
        "errors": [
            {
                "code": "1291",
                "message": "Job not found. Job ID: 77b65b44-07bc-4557-80e2-7f73a6be8ea4",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            },
        ],
    });
    server
        .mock("POST", "/v1/jobs/unknown/cancel")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(not_found.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let mut result = client
        .cancel_job("77b65b44-07bc-4557-80e2-7f73a6be8ea4", false)
        .await;
    assert!(result.is_ok());

    result = client.cancel_job("unknown", false).await;
    assert!(result.is_err());
}

/// Test Client.cancel_job().
#[tokio::test]
async fn test_cancel_and_delete_job() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    server
        .mock(
            "POST",
            "/v1/jobs/77b65b44-07bc-4557-80e2-7f73a6be8ea4/cancel",
        )
        .with_status(204)
        .with_header("content-type", "application/json")
        .create_async()
        .await;

    server
        .mock("DELETE", "/v1/jobs/77b65b44-07bc-4557-80e2-7f73a6be8ea4")
        .with_status(204)
        .with_header("content-type", "application/json")
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let result = client
        .cancel_job("77b65b44-07bc-4557-80e2-7f73a6be8ea4", true)
        .await;
    assert!(result.is_ok());
}

/// Test Client.delete_job().
#[tokio::test]
async fn test_delete_job() {
    common::setup();

    let mut server = mockito::Server::new_async().await;

    server
        .mock("DELETE", "/v1/jobs/77b65b44-07bc-4557-80e2-7f73a6be8ea4")
        .with_status(204)
        .with_header("content-type", "application/json")
        .create_async()
        .await;

    let not_found = json!({
        "status_code": 404,
        "title": "Job not found. Job ID: unknown",
        "trace": "",
        "errors": [
            {
                "code": "1291",
                "message": "Job not found. Job ID: 77b65b44-07bc-4557-80e2-7f73a6be8ea4",
                "more_info": "https://cloud.ibm.com/apidocs/quantum-computing#error-handling",
            },
        ],
    });
    server
        .mock("DELETE", "/v1/jobs/unknown")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(not_found.to_string())
        .create_async()
        .await;

    let base_url = server.url();
    let client = ClientBuilder::new(&base_url).build().unwrap();
    let mut result = client
        .delete_job("77b65b44-07bc-4557-80e2-7f73a6be8ea4")
        .await;
    assert!(result.is_ok());

    result = client.delete_job("unknown").await;
    assert!(result.is_err());
}
