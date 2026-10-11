// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::super::IBMQuantumSystem;
use super::s3_env;
use crate::error::QrmiError;
use crate::models::{Payload, ResourceType};
use crate::QuantumResource;
use std::collections::HashMap;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "test_eagle";
    let mut qrmi = IBMQuantumSystem::from_config(BACKEND_NAME, required_only_config())
        .expect("construction should succeed");

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");
    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, BACKEND_NAME);
    assert_eq!(resource_type, ResourceType::IBMQuantumSystem);
}

fn required_only_config() -> HashMap<String, String> {
    HashMap::from([
        (
            "QRMI_IBM_QS_ENDPOINT".to_string(),
            "http://localhost".to_string(),
        ),
        ("QRMI_IBM_QS_IAM_APIKEY".to_string(), "dummy".to_string()),
        ("QRMI_IBM_QS_SERVICE_CRN".to_string(), "dummy".to_string()),
        (
            "QRMI_IBM_QS_IAM_ENDPOINT".to_string(),
            "http://localhost".to_string(),
        ),
    ])
}

#[test]
fn from_config_works_without_s3_keys() {
    assert!(IBMQuantumSystem::from_config("test_eagle", required_only_config()).is_ok());
}

#[test]
fn from_config_accepts_env_style_keys_lowercased() {
    let config: HashMap<String, String> = required_only_config()
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    assert!(IBMQuantumSystem::from_config("test_eagle", config).is_ok());
}

#[test]
fn from_config_requires_all_s3_keys_together() {
    let mut config = required_only_config();
    // Only one of the five S3 keys set -- should be treated as "no S3", not an error.
    config.insert("QRMI_IBM_QS_S3_BUCKET".to_string(), "my-bucket".to_string());
    assert!(IBMQuantumSystem::from_config("test_eagle", config).is_ok());
}

#[test]
fn from_config_missing_required_key_errors() {
    let config = HashMap::from([(
        "QRMI_IBM_QS_ENDPOINT".to_string(),
        "http://localhost".to_string(),
    )]);
    assert!(IBMQuantumSystem::from_config("test_eagle", config).is_err());
}

fn sampler_payload() -> Payload {
    Payload::QiskitPrimitive {
        input: "{}".to_string(),
        program_id: "sampler".to_string(),
    }
}

#[tokio::test]
async fn task_start_reads_job_timeout_from_config() {
    let mut qrmi = IBMQuantumSystem::from_config("test_eagle", required_only_config())
        .expect("construction should succeed");
    // Fails before any network access.
    let err = qrmi.task_start(sampler_payload()).await.unwrap_err();
    assert!(
        matches!(err, QrmiError::MissingConfigKey(ref key) if key == "QRMI_JOB_TIMEOUT_SECONDS")
    );
}

#[tokio::test]
async fn task_start_rejects_malformed_job_timeout() {
    let mut config = required_only_config();
    config.insert("QRMI_JOB_TIMEOUT_SECONDS".to_string(), "abc".to_string());
    // Construction doesn't read the timeout, so it still succeeds.
    let mut qrmi =
        IBMQuantumSystem::from_config("test_eagle", config).expect("construction should succeed");
    let err = qrmi.task_start(sampler_payload()).await.unwrap_err();
    assert!(
        matches!(err, QrmiError::ParseError { ref name, .. } if name == "QRMI_JOB_TIMEOUT_SECONDS")
    );
}

#[test]
fn s3_env_reads_from_config() {
    let mut config = required_only_config();
    for (key, value) in [
        ("QRMI_IBM_QS_S3_BUCKET", "my-bucket"),
        ("QRMI_IBM_QS_S3_ENDPOINT", "http://localhost:9000"),
        ("QRMI_IBM_QS_AWS_ACCESS_KEY_ID", "dummy"),
        ("QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY", "dummy"),
        ("QRMI_IBM_QS_S3_REGION", "us-east-1"),
    ] {
        config.insert(key.to_string(), value.to_string());
    }
    let qrmi =
        IBMQuantumSystem::from_config("test_eagle", config).expect("construction should succeed");
    let s3 = s3_env(&qrmi.settings).expect("S3 settings should be read from config");
    assert_eq!(s3.bucket, "my-bucket");
}

#[tokio::test]
async fn task_result_reads_s3_settings_from_config() {
    let mut qrmi = IBMQuantumSystem::from_config("test_eagle", required_only_config())
        .expect("construction should succeed");
    // Fails before any network access.
    let err = qrmi
        .task_result("some-task")
        .await
        .err()
        .expect("task_result should fail");
    assert!(matches!(err, QrmiError::MissingConfigKey(ref key) if key == "QRMI_IBM_QS_S3_BUCKET"));
}

/// Tests for recording job status to S3 tags and deleting finished jobs.
///
/// Both the Quantum System API service and S3 are mocked by a single mockito
/// server. Each test uses its own backend name.
mod job_cleanup {
    use super::super::{effective_status, IBMQuantumSystem, S3Store, Settings};
    use crate::models::{Payload, TaskStatus};
    use crate::QuantumResource;
    use mockito::{Matcher, Mock, ServerGuard};
    use quantum_system_api::models::{Job, JobStatus};
    use quantum_system_api::utils::s3::S3Client;
    use quantum_system_api::ClientBuilder;
    use serde_json::{json, Value};

    const BUCKET: &str = "qrmi-test-bucket";
    const NO_SUCH_KEY: &str = r#"<?xml version="1.0" encoding="UTF-8"?><Error><Code>NoSuchKey</Code><Message>The specified key does not exist.</Message></Error>"#;
    const OTHER_UID: &str = "99999";

    fn my_uid() -> String {
        // SAFETY: getuid() is always successful and has no side effects.
        unsafe { libc::getuid() }.to_string()
    }

    fn ago(secs: i64) -> String {
        (chrono::Utc::now() - chrono::Duration::seconds(secs)).to_rfc3339()
    }

    /// Metrics of a job finished `finished_ago` seconds ago.
    fn metrics(finished_ago: i64, circuits_execution_time_ns: Option<i64>) -> Value {
        let mut m = json!({"timestamps": {"created": ago(finished_ago + 10), "finished": ago(finished_ago)}});
        if let Some(ns) = circuits_execution_time_ns {
            m["circuits_execution_time_ns"] = ns.into();
        }
        m
    }

    fn job(id: &str, backend: &str, status: &str, metrics: Value) -> Value {
        json!({
            "id": id,
            "backend": backend,
            "created_time": ago(60),
            "program_id": "sampler",
            "status": status,
            "storage": {},
            "metrics": metrics,
        })
    }

    fn tagging_xml(tags: &[(&str, &str)]) -> String {
        let tags: String = tags
            .iter()
            .map(|(k, v)| format!("<Tag><Key>{k}</Key><Value>{v}</Value></Tag>"))
            .collect();
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?><Tagging><TagSet>{tags}</TagSet></Tagging>"#
        )
    }

    /// Returns a QRMI instance for `backend` whose API client and S3 bucket
    /// both point at `server`.
    fn qrmi(backend: &str, server: &ServerGuard) -> IBMQuantumSystem {
        IBMQuantumSystem {
            api_client: ClientBuilder::new(server.url())
                .with_s3bucket(
                    "access_key",
                    "secret",
                    server.url(),
                    BUCKET,
                    "us-east-1",
                    None::<String>,
                )
                .build()
                .unwrap(),
            backend_name: backend.to_string(),
            settings: Settings::env(backend),
            s3: Some(S3Store {
                client: S3Client::new(server.url(), "access_key", "secret", "us-east-1"),
                bucket: BUCKET.to_string(),
            }),
        }
    }

    async fn mock_list_jobs(server: &mut ServerGuard, jobs: &[&Value]) -> Mock {
        server
            .mock("GET", "/v1/jobs")
            .with_status(200)
            .with_body(json!({ "jobs": jobs }).to_string())
            .create_async()
            .await
    }

    async fn mock_get_job(server: &mut ServerGuard, job: &Value) -> Mock {
        server
            .mock(
                "GET",
                format!("/v1/jobs/{}", job["id"].as_str().unwrap()).as_str(),
            )
            .with_status(200)
            .with_body(job.to_string())
            .create_async()
            .await
    }

    async fn mock_job_not_found(server: &mut ServerGuard, id: &str) -> Mock {
        server
            .mock("GET", format!("/v1/jobs/{id}").as_str())
            .with_status(404)
            .with_body(r#"{"errors":[{"code":"1291","message":"Job not found."}]}"#)
            .create_async()
            .await
    }

    async fn mock_delete_job(server: &mut ServerGuard, id: &str, expect: usize) -> Mock {
        server
            .mock("DELETE", format!("/v1/jobs/{id}").as_str())
            .with_status(204)
            .expect(expect)
            .create_async()
            .await
    }

    /// Mocks GetObjectTagging of `input_<id>.json`. `None` means the object does not exist.
    async fn mock_input_tags(
        server: &mut ServerGuard,
        id: &str,
        tags: Option<&[(&str, &str)]>,
    ) -> Mock {
        let mock = server
            .mock("GET", format!("/{BUCKET}/input_{id}.json").as_str())
            .match_query(Matcher::Regex("tagging".into()));
        match tags {
            Some(tags) => mock.with_status(200).with_body(tagging_xml(tags)),
            None => mock.with_status(404).with_body(NO_SUCH_KEY),
        }
        .create_async()
        .await
    }

    /// Mocks PutObjectTagging of `input_<id>.json`, expecting the body to contain all of `tags`.
    async fn mock_put_input_tags(
        server: &mut ServerGuard,
        id: &str,
        tags: &[(&str, &str)],
        expect: usize,
    ) -> Mock {
        let matchers = tags
            .iter()
            .map(|(k, v)| Matcher::Regex(format!("<Key>{k}</Key><Value>{v}</Value>")))
            .collect();
        server
            .mock("PUT", format!("/{BUCKET}/input_{id}.json").as_str())
            .match_query(Matcher::Regex("tagging".into()))
            .match_body(Matcher::AllOf(matchers))
            .with_status(200)
            .expect(expect)
            .create_async()
            .await
    }

    // ---- effective_status() ----

    fn effective(status: &str, metrics: Value) -> JobStatus {
        let job: Job = serde_json::from_value(job("j", "b", status, metrics)).unwrap();
        effective_status(&job)
    }

    #[test]
    fn effective_status_completed_with_execution_time() {
        assert_eq!(
            effective("Completed", metrics(5, Some(1))),
            JobStatus::Completed
        );
    }

    #[test]
    fn effective_status_completed_without_execution_time_is_running() {
        assert_eq!(effective("Completed", metrics(5, None)), JobStatus::Running);
    }

    #[test]
    fn effective_status_completed_without_execution_time_after_timeout() {
        assert_eq!(
            effective("Completed", metrics(2 * 60 * 60, None)),
            JobStatus::Completed
        );
    }

    #[test]
    fn effective_status_uses_created_if_finished_is_missing() {
        let recent = json!({"timestamps": {"created": ago(5)}});
        let old = json!({"timestamps": {"created": ago(2 * 60 * 60)}});
        assert_eq!(effective("Completed", recent), JobStatus::Running);
        assert_eq!(effective("Completed", old), JobStatus::Completed);
    }

    #[test]
    fn effective_status_completed_without_metrics() {
        assert_eq!(effective("Completed", Value::Null), JobStatus::Completed);
    }

    #[test]
    fn effective_status_completed_with_invalid_timestamp() {
        let m = json!({"timestamps": {"created": "invalid", "finished": "invalid"}});
        assert_eq!(effective("Completed", m), JobStatus::Completed);
    }

    #[test]
    fn effective_status_other_statuses_are_returned_as_is() {
        assert_eq!(effective("Running", metrics(5, None)), JobStatus::Running);
        assert_eq!(effective("Failed", metrics(5, None)), JobStatus::Failed);
        assert_eq!(
            effective("Cancelled", metrics(5, None)),
            JobStatus::Cancelled
        );
    }

    // ---- delete_completed_jobs() ----

    #[tokio::test]
    async fn delete_completed_jobs_deletes_only_own_finished_jobs() {
        const B: &str = "cleanup_own_only";
        let me = my_uid();
        let mut server = mockito::Server::new_async().await;
        let done = job("done", B, "Completed", metrics(5, Some(123)));
        let failed = job("failed", B, "Failed", metrics(5, None));
        let waiting = job("waiting", B, "Completed", metrics(5, None));
        let running = job("running", B, "Running", metrics(5, None));
        let other_user = job("other_user", B, "Completed", metrics(5, Some(1)));
        let no_uid = job("no_uid", B, "Completed", metrics(5, Some(1)));
        let no_input = job("no_input", B, "Completed", metrics(5, Some(1)));
        let other_backend = job("other_backend", "another", "Completed", metrics(5, Some(1)));
        mock_list_jobs(
            &mut server,
            &[
                &done,
                &failed,
                &waiting,
                &running,
                &other_user,
                &no_uid,
                &no_input,
                &other_backend,
            ],
        )
        .await;

        mock_input_tags(&mut server, "done", Some(&[("qrmi:uid", &me)])).await;
        mock_input_tags(&mut server, "failed", Some(&[("qrmi:uid", &me)])).await;
        mock_input_tags(&mut server, "other_user", Some(&[("qrmi:uid", OTHER_UID)])).await;
        mock_input_tags(&mut server, "no_uid", Some(&[])).await;
        mock_input_tags(&mut server, "no_input", None).await;

        let mut expected = vec![
            mock_put_input_tags(
                &mut server,
                "done",
                &[
                    ("qrmi:uid", &me),
                    ("qrmi:status", "Completed"),
                    ("qrmi:status:circuits_execution_time_ns", "123"),
                    ("qrmi:status:created", ".+"),
                    ("qrmi:status:finished", ".+"),
                ],
                1,
            )
            .await,
            mock_put_input_tags(&mut server, "failed", &[("qrmi:status", "Failed")], 1).await,
            mock_delete_job(&mut server, "done", 1).await,
            mock_delete_job(&mut server, "failed", 1).await,
        ];
        for id in [
            "waiting",
            "running",
            "other_user",
            "no_uid",
            "no_input",
            "other_backend",
        ] {
            expected.push(mock_put_input_tags(&mut server, id, &[], 0).await);
            expected.push(mock_delete_job(&mut server, id, 0).await);
        }

        qrmi(B, &server).delete_completed_jobs().await;
        for mock in expected {
            mock.assert_async().await;
        }
    }

    #[tokio::test]
    async fn delete_completed_jobs_deletes_job_even_if_tagging_fails() {
        const B: &str = "cleanup_tagging_fails";
        let mut server = mockito::Server::new_async().await;
        let done = job("done", B, "Completed", metrics(5, Some(1)));
        mock_list_jobs(&mut server, &[&done]).await;
        mock_input_tags(&mut server, "done", Some(&[("qrmi:uid", &my_uid())])).await;
        server
            .mock("PUT", format!("/{BUCKET}/input_done.json").as_str())
            .match_query(Matcher::Regex("tagging".into()))
            .with_status(500)
            .create_async()
            .await;
        let delete = mock_delete_job(&mut server, "done", 1).await;

        qrmi(B, &server).delete_completed_jobs().await;
        delete.assert_async().await;
    }

    #[tokio::test]
    async fn delete_completed_jobs_without_s3_config_deletes_nothing() {
        let mut server = mockito::Server::new_async().await;
        let done = job("done", "cleanup_no_s3", "Completed", metrics(5, Some(1)));
        mock_list_jobs(&mut server, &[&done]).await;
        let delete = server
            .mock("DELETE", Matcher::Any)
            .expect(0)
            .create_async()
            .await;

        // S3 is not configured for this backend.
        let qrmi = IBMQuantumSystem {
            api_client: ClientBuilder::new(server.url()).build().unwrap(),
            backend_name: "cleanup_no_s3".to_string(),
            settings: Settings::env("cleanup_no_s3"),
            s3: None,
        };
        qrmi.delete_completed_jobs().await;
        delete.assert_async().await;
    }

    // ---- task_status() ----

    #[tokio::test]
    async fn task_status_waits_for_circuits_execution_time() {
        const B: &str = "status_waits";
        let mut server = mockito::Server::new_async().await;
        let waiting = job("j", B, "Completed", metrics(5, None));
        mock_get_job(&mut server, &waiting).await;
        mock_list_jobs(&mut server, &[&waiting]).await;
        let put = mock_put_input_tags(&mut server, "j", &[], 0).await;
        let delete = mock_delete_job(&mut server, "j", 0).await;

        let status = qrmi(B, &server).task_status("j").await.unwrap();
        assert_eq!(status, TaskStatus::Running);
        put.assert_async().await;
        delete.assert_async().await;
    }

    #[tokio::test]
    async fn task_status_records_and_deletes_completed_job() {
        const B: &str = "status_completed";
        let mut server = mockito::Server::new_async().await;
        let done = job("j", B, "Completed", metrics(5, Some(42)));
        mock_get_job(&mut server, &done).await;
        mock_list_jobs(&mut server, &[&done]).await;
        mock_input_tags(&mut server, "j", Some(&[("qrmi:uid", &my_uid())])).await;
        let put = mock_put_input_tags(
            &mut server,
            "j",
            &[
                ("qrmi:status", "Completed"),
                ("qrmi:status:circuits_execution_time_ns", "42"),
            ],
            1,
        )
        .await;
        let delete = mock_delete_job(&mut server, "j", 1).await;

        let status = qrmi(B, &server).task_status("j").await.unwrap();
        assert_eq!(status, TaskStatus::Completed);
        put.assert_async().await;
        delete.assert_async().await;
    }

    #[tokio::test]
    async fn task_status_falls_back_to_s3_after_deletion() {
        const B: &str = "status_fallback";
        let mut server = mockito::Server::new_async().await;
        mock_job_not_found(&mut server, "j").await;
        mock_list_jobs(&mut server, &[]).await;
        mock_input_tags(
            &mut server,
            "j",
            Some(&[("qrmi:uid", &my_uid()), ("qrmi:status", "Cancelled")]),
        )
        .await;

        let status = qrmi(B, &server).task_status("j").await.unwrap();
        assert_eq!(status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn task_status_unknown_job_errors() {
        const B: &str = "status_unknown";
        let mut server = mockito::Server::new_async().await;
        mock_job_not_found(&mut server, "j").await;
        mock_list_jobs(&mut server, &[]).await;
        mock_input_tags(&mut server, "j", None).await;

        assert!(qrmi(B, &server).task_status("j").await.is_err());
    }

    // ---- task_result() ----

    #[tokio::test]
    async fn task_result_after_deletion_reads_results_from_s3() {
        const B: &str = "result_fallback";
        let mut server = mockito::Server::new_async().await;
        mock_job_not_found(&mut server, "j").await;
        mock_list_jobs(&mut server, &[]).await;
        mock_input_tags(
            &mut server,
            "j",
            Some(&[("qrmi:uid", &my_uid()), ("qrmi:status", "Completed")]),
        )
        .await;
        server
            .mock("GET", format!("/{BUCKET}/results_j.json").as_str())
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(r#"{"results":[]}"#)
            .create_async()
            .await;

        let result = qrmi(B, &server).task_result("j").await.unwrap();
        assert_eq!(result.value, r#"{"results":[]}"#);
    }

    #[tokio::test]
    async fn task_result_is_not_ready_until_circuits_execution_time_is_available() {
        const B: &str = "result_waits";
        let mut server = mockito::Server::new_async().await;
        let waiting = job("j", B, "Completed", metrics(5, None));
        mock_get_job(&mut server, &waiting).await;
        mock_list_jobs(&mut server, &[&waiting]).await;

        let err = qrmi(B, &server).task_result("j").await.unwrap_err();
        assert!(err.to_string().contains("task is running"), "{err}");
    }

    // ---- task_stop() ----

    #[tokio::test]
    async fn task_stop_cancels_running_job_and_records_cancelled() {
        const B: &str = "stop_running";
        let mut server = mockito::Server::new_async().await;
        mock_get_job(&mut server, &job("j", B, "Running", metrics(5, None))).await;
        let cancel = server
            .mock("POST", "/v1/jobs/j/cancel")
            .with_status(204)
            .expect(1)
            .create_async()
            .await;
        // Cancellation is synchronous: the job is listed as Cancelled afterwards.
        mock_list_jobs(&mut server, &[&job("j", B, "Cancelled", metrics(0, None))]).await;
        mock_input_tags(&mut server, "j", Some(&[("qrmi:uid", &my_uid())])).await;
        let put = mock_put_input_tags(&mut server, "j", &[("qrmi:status", "Cancelled")], 1).await;
        let delete = mock_delete_job(&mut server, "j", 1).await;

        qrmi(B, &server).task_stop("j").await.unwrap();
        cancel.assert_async().await;
        put.assert_async().await;
        delete.assert_async().await;
    }

    #[tokio::test]
    async fn task_stop_does_not_wait_for_circuits_execution_time() {
        const B: &str = "stop_no_wait";
        let mut server = mockito::Server::new_async().await;
        let waiting = job("j", B, "Completed", metrics(5, None));
        mock_get_job(&mut server, &waiting).await;
        let cancel = server
            .mock("POST", "/v1/jobs/j/cancel")
            .expect(0)
            .create_async()
            .await;
        mock_list_jobs(&mut server, &[&waiting]).await;
        mock_input_tags(&mut server, "j", Some(&[("qrmi:uid", &my_uid())])).await;
        let put = mock_put_input_tags(&mut server, "j", &[("qrmi:status", "Completed")], 1).await;
        let delete = mock_delete_job(&mut server, "j", 1).await;

        qrmi(B, &server).task_stop("j").await.unwrap();
        cancel.assert_async().await;
        put.assert_async().await;
        delete.assert_async().await;
    }

    // ---- task_start() ----

    #[tokio::test]
    async fn task_start_tags_input_object() {
        const B: &str = "start_tags";
        let me = my_uid();
        // SAFETY: getgid() is always successful and has no side effects.
        let gid = unsafe { libc::getgid() }.to_string();
        let mut server = mockito::Server::new_async().await;
        std::env::set_var(format!("{B}_QRMI_JOB_TIMEOUT_SECONDS"), "60");
        mock_list_jobs(&mut server, &[]).await;
        let input = server
            .mock(
                "PUT",
                Matcher::Regex(format!("^/{BUCKET}/input_[^/]+\\.json$")),
            )
            .match_query(Matcher::Any)
            .match_header(
                "x-amz-tagging",
                Matcher::AllOf(vec![
                    Matcher::Regex(format!("qrmi%3Auid={me}(&|$)")),
                    Matcher::Regex(format!("qrmi%3Agid={gid}(&|$)")),
                    Matcher::Regex("qrmi%3Aprogram_type=sampler(&|$)".into()),
                ]),
            )
            .with_status(200)
            .expect(1)
            .create_async()
            .await;
        server
            .mock("POST", "/v1/jobs")
            .with_status(204)
            .create_async()
            .await;

        let payload = Payload::QiskitPrimitive {
            input: "{}".to_string(),
            program_id: "sampler".to_string(),
        };
        qrmi(B, &server).task_start(payload).await.unwrap();
        input.assert_async().await;
    }
}
