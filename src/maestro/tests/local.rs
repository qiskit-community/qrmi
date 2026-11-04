use super::MaestroLocal;
use crate::models::{Payload, ResourceType, TaskStatus};
use crate::{QRMIService, QrmiErrorKind, QuantumResource};
use futures::executor::block_on;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// The socket setting is process-wide. All environment changes made by these
// tests are serialized and restored, including on an assertion failure.
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    name: String,
    previous: Option<OsString>,
}

impl EnvGuard {
    fn set(name: &str, value: Option<&std::ffi::OsStr>) -> Self {
        let previous = std::env::var_os(name);
        match value {
            Some(value) => std::env::set_var(name, value),
            None => std::env::remove_var(name),
        }
        Self {
            name: name.into(),
            previous,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(&self.name, value),
            None => std::env::remove_var(&self.name),
        }
    }
}

struct Server {
    path: PathBuf,
    replies: Arc<Mutex<VecDeque<(String, String)>>>,
    errors: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    _env: EnvGuard,
}

impl Server {
    fn new(replies: &[(&str, &str)]) -> Self {
        let path = std::env::temp_dir().join(format!("qrmi-{}.sock", uuid::Uuid::new_v4()));
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let replies: Arc<Mutex<VecDeque<(String, String)>>> = Arc::new(Mutex::new(
            replies
                .iter()
                .map(|(command, reply)| (command.to_string(), reply.to_string()))
                .collect(),
        ));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (pending, failures, stopping) = (replies.clone(), errors.clone(), stop.clone());
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let mut command = String::new();
                        if let Err(error) = BufReader::new(&mut stream).read_line(&mut command) {
                            failures.lock().unwrap().push(error.to_string());
                            continue;
                        }
                        let reply = match pending.lock().unwrap().pop_front() {
                            Some((expected, reply)) if command.trim_end() == expected => reply,
                            unexpected => {
                                failures
                                    .lock()
                                    .unwrap()
                                    .push(format!("Received {command:?}, expected {unexpected:?}"));
                                "ERROR unexpected command".into()
                            }
                        };
                        if let Err(error) = writeln!(stream, "{reply}") {
                            failures.lock().unwrap().push(error.to_string());
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("Mock server: {error}"),
                }
            }
        });
        let socket_env = EnvGuard::set("QRMI_MAESTRO_SOCKET", Some(path.as_os_str()));
        Self {
            path,
            replies,
            errors,
            stop,
            worker: Some(worker),
            _env: socket_env,
        }
    }

    fn assert_done(&self) {
        let errors = self.errors.lock().unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        let replies = self.replies.lock().unwrap();
        assert!(replies.is_empty(), "Unconsumed requests: {replies:?}");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.worker.take().unwrap().join().unwrap();
        std::fs::remove_file(&self.path).unwrap();
    }
}

fn payload() -> Payload {
    Payload::MaestroLocal {
        input: "OPENQASM 2.0;\nqreg q[1];".into(),
        job_type: "execute".into(),
        qubits: 1,
        simulator_type: 0,
        simulation_method: 0,
        observables: String::new(),
        config: "{}".into(),
    }
}

fn resource(session: Option<u32>) -> MaestroLocal {
    MaestroLocal {
        backend_name: "test_maestro".into(),
        session_id: session,
        request_client: Default::default(),
        terminal_tasks: Default::default(),
    }
}

#[test]
fn explicit_config_is_isolated_from_environment() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _socket = EnvGuard::set("QRMI_MAESTRO_SOCKET", Some("/wrong.sock".as_ref()));
    let _token = EnvGuard::set(
        "configured_QRMI_JOB_ACQUISITION_TOKEN",
        Some("invalid".as_ref()),
    );
    let defaults = MaestroLocal::from_config("configured", Default::default()).unwrap();
    assert_eq!(
        defaults.request_client.socket_path(),
        std::path::Path::new("/run/maestro.sock")
    );
    assert_eq!(defaults.session_id, None);
    let configured = MaestroLocal::from_config(
        "configured",
        std::collections::HashMap::from([
            ("qrmi_maestro_socket".into(), "/explicit.sock".into()),
            ("qrmi_job_acquisition_token".into(), "42".into()),
        ]),
    )
    .unwrap();
    assert_eq!(
        configured.request_client.socket_path(),
        std::path::Path::new("/explicit.sock")
    );
    assert_eq!(configured.session_id, Some(42));
    assert_eq!(std::env::var("QRMI_MAESTRO_SOCKET").unwrap(), "/wrong.sock");
    for invalid in ["invalid", "-1", "4294967296", ""] {
        let error = MaestroLocal::from_config(
            "configured",
            std::collections::HashMap::from([
                ("QRMI_JOB_ACQUISITION_TOKEN".into(), invalid.into()),
                ("qrmi_job_acquisition_token".into(), "42".into()),
            ]),
        )
        .err()
        .unwrap();
        assert_eq!(error.kind(), QrmiErrorKind::ParseError);
    }
}

#[test]
fn config_factory_preserves_session_and_status_behavior() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        (
            "API {\"version\":2,\"command\":\"status\"}",
            "ERROR Unknown command",
        ),
        ("PING", "OK YES"),
        (
            "API {\"version\":2,\"command\":\"status\"}",
            "ERROR Unknown command",
        ),
        ("PING", "OK NO"),
        (
            "API {\"version\":2,\"command\":\"status\"}",
            "ERROR Unknown command",
        ),
        ("PING", "ERROR unavailable"),
        ("SESSION 42 EXISTS", "OK YES"),
        ("SESSION 42 DELETE", "OK YES"),
    ]);
    let mut resource = crate::common::create_resource_from_config(
        &ResourceType::MaestroLocal,
        "configured",
        std::collections::HashMap::from([
            (
                "QRMI_MAESTRO_SOCKET".into(),
                server.path.to_str().unwrap().into(),
            ),
            ("QRMI_JOB_ACQUISITION_TOKEN".into(), "42".into()),
        ]),
    )
    .unwrap();
    let _socket = EnvGuard::set("QRMI_MAESTRO_SOCKET", Some("/wrong.sock".as_ref()));
    block_on(async {
        let status = resource.status().await.unwrap();
        assert_eq!(status.status, crate::ResourceStatusCode::Online);
        assert_eq!(status.healthy, None);
        assert_eq!(status.capacity, None);
        assert_eq!(
            resource.status().await.unwrap().status,
            crate::ResourceStatusCode::Offline
        );
        assert!(resource
            .status()
            .await
            .unwrap_err()
            .to_string()
            .contains("unavailable"));
        assert_eq!(resource.acquire().await.unwrap(), "42");
        resource.release("42").await.unwrap();
    });
    server.assert_done();
}

fn request_payload(input: String) -> Payload {
    Payload::MaestroLocal {
        input,
        job_type: "request".into(),
        qubits: 0,
        simulator_type: 0,
        simulation_method: 0,
        observables: String::new(),
        config: "{}".into(),
    }
}

#[test]
fn native_request_negotiates_and_submits_without_legacy_setters() {
    let _lock = ENV_LOCK.lock().unwrap();
    let input = serde_json::json!({
        "schema_version": 2, "operation": "execute",
        "circuit": {"source": "OPENQASM 2.0;\n// preserve\nqreg q[1];", "num_qubits": 1},
        "simulator": {"backend": "distributed_gpu", "method": "statevector",
                      "distribution": {"devices": [0, 1]}}
    })
    .to_string();
    let submit = format!(
        "API {{\"version\":2,\"command\":\"submit\",\"session_id\":3,\"request\":{input}}}"
    );
    let server = Server::new(&[
        (
            "API {\"version\":2,\"command\":\"capabilities\"}",
            "OK {\"version\":2,\"ok\":true,\"native\":{\"schema_version\":2}}",
        ),
        (&submit, "OK {\"version\":2,\"ok\":true,\"task_id\":9}"),
    ]);
    let mut qrmi = resource(Some(3));
    assert_eq!(
        block_on(qrmi.task_start(request_payload(input))).unwrap(),
        "9"
    );
    server.assert_done();
}

#[test]
fn native_target_and_failure_logs_preserve_vendor_documents() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("API {\"version\":2,\"command\":\"capabilities\"}",
         "OK {\"version\":2,\"ok\":true,\"native\":{\"schema_version\":2,\"noise_channels\":[\"thermal_relaxation\"]}}"),
        ("API {\"version\":2,\"command\":\"logs\",\"session_id\":4,\"task_id\":8}",
         "OK {\"version\":2,\"ok\":true,\"logs\":\"native diagnostic\\n\",\"error\":{\"code\":\"backend_unavailable\",\"message\":\"No GPUs\"}}"),
    ]);
    let mut qrmi = resource(Some(4));
    let target: serde_json::Value =
        serde_json::from_str(&block_on(qrmi.target()).unwrap().value).unwrap();
    assert_eq!(target["native"]["noise_channels"][0], "thermal_relaxation");
    let logs: serde_json::Value =
        serde_json::from_str(&block_on(qrmi.task_logs("8")).unwrap()).unwrap();
    assert_eq!(logs["error"]["code"], "backend_unavailable");
    assert_eq!(logs["logs"], "native diagnostic\n");
    server.assert_done();
}

#[test]
fn native_logs_preserve_cancelled_task_history_and_truncation_metadata() {
    let _lock = ENV_LOCK.lock().unwrap();
    let expected = serde_json::json!({
        "version": 2, "ok": true, "session_id": 4, "task_id": 8,
        "logs": "startup\n[... Maestro log output truncated ...]\nshutdown",
        "logs_truncated": true, "logs_omitted_bytes": 12345,
        "error": null, "failure_message": null,
        "context": {"operation": "execute", "simulator": {"backend": "gpu"},
                    "launch": {"profile": "cluster", "ranks": 2}},
        "events": [
            {"timestamp_unix_ms": 1700000000000_u64, "state": "queued"},
            {"timestamp_unix_ms": 1700000000001_u64, "state": "started"},
            {"timestamp_unix_ms": 1700000000002_u64, "state": "cancelled"}
        ],
        "events_truncated": false,
    });
    let server = Server::new(&[(
        r#"API {"version":2,"command":"logs","session_id":4,"task_id":8}"#,
        &format!("OK {expected}"),
    )]);
    let mut qrmi = resource(Some(4));
    qrmi.terminal_tasks.insert((4, 8), TaskStatus::Cancelled);
    let actual: serde_json::Value =
        serde_json::from_str(&block_on(qrmi.task_logs("8")).unwrap()).unwrap();
    assert_eq!(actual, expected);
    server.assert_done();
}

#[test]
fn old_server_cannot_silently_accept_native_features() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[(
        "API {\"version\":2,\"command\":\"capabilities\"}",
        "ERROR Unknown command",
    )]);
    let mut qrmi = resource(Some(3));
    let error = block_on(qrmi.task_start(request_payload(
        r#"{"schema_version":2,"operation":"execute"}"#.into(),
    )))
    .unwrap_err();
    assert_eq!(error.kind(), QrmiErrorKind::UnsupportedFunction);
    server.assert_done();
}

#[test]
fn unrecognized_submission_replies_keep_task_creation_uncertain() {
    // task_start promises "no task" for every kind except Other. After the
    // request was sent, only a well-formed API v2 refusal can keep that promise.
    let _lock = ENV_LOCK.lock().unwrap();
    let request = r#"{"schema_version":2,"operation":"execute"}"#;
    let submit = format!(
        "API {{\"version\":2,\"command\":\"submit\",\"session_id\":3,\"request\":{request}}}"
    );
    for (reply, kind) in [
        (r#"OK {"ok":true,"task_id":73}"#, QrmiErrorKind::Other),
        ("garbled response", QrmiErrorKind::Other),
        ("ERROR Unknown command", QrmiErrorKind::Other),
        (
            r#"OK {"version":3,"ok":true,"task_id":73}"#,
            QrmiErrorKind::Other,
        ),
        (r#"OK {"version":2,"ok":true}"#, QrmiErrorKind::Other),
        (
            r#"OK {"version":2,"ok":false,"error":{"code":"invalid_input","message":"review probe"}}"#,
            QrmiErrorKind::InvalidInput,
        ),
        (
            r#"OK {"version":2,"ok":false,"error":{"code":"unsupported_capability","message":"review probe"}}"#,
            QrmiErrorKind::UnsupportedFunction,
        ),
    ] {
        let server = Server::new(&[
            (
                "API {\"version\":2,\"command\":\"capabilities\"}",
                "OK {\"version\":2,\"ok\":true,\"native\":{\"schema_version\":2}}",
            ),
            (&submit, reply),
        ]);
        let mut qrmi = resource(Some(3));
        let error = block_on(qrmi.task_start(request_payload(request.into()))).unwrap_err();
        assert_eq!(error.kind(), kind, "{reply}");
        if kind == QrmiErrorKind::Other {
            assert!(
                error.to_string().contains("may have been created"),
                "{error}"
            );
        }
        server.assert_done();
    }
}

#[test]
fn malformed_native_request_does_not_create_a_task() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[]);
    let mut qrmi = resource(Some(3));
    for input in ["{", "[]", r#"{"schema_version":1}"#] {
        assert_eq!(
            block_on(qrmi.task_start(request_payload(input.into())))
                .unwrap_err()
                .kind(),
            QrmiErrorKind::InvalidInput
        );
    }
    server.assert_done();
}

#[test]
fn classified_configuration_and_input_errors_do_not_contact_server() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[]);
    let _token = EnvGuard::set(
        "test_maestro_QRMI_JOB_ACQUISITION_TOKEN",
        Some("invalid".as_ref()),
    );
    let error = MaestroLocal::new("test_maestro").err().unwrap();
    assert_eq!(error.kind(), QrmiErrorKind::ParseError);
    block_on(async {
        let mut qrmi = resource(None);
        assert_eq!(
            qrmi.task_status("invalid").await.unwrap_err().kind(),
            QrmiErrorKind::InvalidInput
        );
        assert_eq!(
            qrmi.task_status("1").await.unwrap_err().kind(),
            QrmiErrorKind::InvalidConfig
        );
        assert_eq!(
            qrmi.release("invalid").await.unwrap_err().kind(),
            QrmiErrorKind::InvalidInput
        );
        let wrong_payload = Payload::QiskitPrimitive {
            input: "{}".into(),
            program_id: "sampler".into(),
        };
        assert_eq!(
            qrmi.task_start(wrong_payload).await.unwrap_err().kind(),
            QrmiErrorKind::UnsupportedPayload
        );
        for variant in [
            "bad_type",
            "missing_observables",
            "bad_json",
            "array_options",
            "zero_qubits",
        ] {
            let mut input = payload();
            if let Payload::MaestroLocal {
                job_type,
                config,
                qubits,
                ..
            } = &mut input
            {
                match variant {
                    "bad_type" => *job_type = "unknown".into(),
                    "missing_observables" => *job_type = "estimate".into(),
                    "bad_json" => *config = "{".into(),
                    "array_options" => *config = "[]".into(),
                    "zero_qubits" => *qubits = 0,
                    _ => unreachable!(),
                }
            }
            assert_eq!(
                qrmi.task_start(input).await.unwrap_err().kind(),
                QrmiErrorKind::InvalidInput,
                "{variant}"
            );
        }
        assert_eq!(
            qrmi.task_logs("1").await.unwrap_err().kind(),
            QrmiErrorKind::InvalidConfig
        );
    });
    server.assert_done();
}

#[test]
fn acquire_replaces_stale_token_reuses_session_and_release_clears_it() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION 7 EXISTS", "OK NO"),
        ("SESSION CREATE", "OK 8"),
        ("SESSION 8 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 EXISTS", "OK NO"),
        ("SESSION 8 DELETE", "OK YES"),
    ]);
    let _token = EnvGuard::set(
        "test_maestro_QRMI_JOB_ACQUISITION_TOKEN",
        Some("7".as_ref()),
    );
    block_on(async {
        let mut qrmi = MaestroLocal::new("test_maestro").unwrap();
        assert_eq!(qrmi.acquire().await.unwrap(), "8");
        assert_eq!(qrmi.acquire().await.unwrap(), "8");
        assert_eq!(
            qrmi.task_status("1").await.unwrap_err().kind(),
            QrmiErrorKind::TaskNotFound
        );
        qrmi.release("8").await.unwrap();
        assert_eq!(
            qrmi.task_status("1").await.unwrap_err().kind(),
            QrmiErrorKind::InvalidConfig
        );
    });
    server.assert_done();
}

#[test]
fn release_honors_explicit_token_and_preserves_different_active_session() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION 9 DELETE", "OK YES"),
        ("SESSION 8 EXISTS", "OK YES"),
    ]);
    block_on(async {
        let mut qrmi = resource(Some(8));
        qrmi.release("9").await.unwrap();
        assert_eq!(qrmi.acquire().await.unwrap(), "8");
    });
    server.assert_done();
}

#[test]
fn acquisition_errors_keep_server_reason_and_do_not_create_another_session() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION CREATE", "ERROR capacity exhausted"),
        ("SESSION 8 EXISTS", "ERROR permission denied"),
    ]);
    block_on(async {
        assert!(resource(None)
            .acquire()
            .await
            .unwrap_err()
            .to_string()
            .contains("capacity exhausted"));
        assert!(resource(Some(8))
            .acquire()
            .await
            .unwrap_err()
            .to_string()
            .contains("permission denied"));
    });
    server.assert_done();
}

#[test]
fn submission_and_consumed_results_keep_completed_status() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION 8 TASK CREATE", "OK 1"),
        ("SESSION 8 TASK 1 SET_TYPE EXECUTE", "OK YES"),
        ("SESSION 8 TASK 1 SET_QUBITS 1", "OK YES"),
        ("SESSION 8 TASK 1 SET_SIM_TYPE 0", "OK YES"),
        ("SESSION 8 TASK 1 SET_METHOD 0", "OK YES"),
        (
            "SESSION 8 TASK 1 SET_QASM OPENQASM 2.0; qreg q[1];",
            "OK YES",
        ),
        ("SESSION 8 TASK 1 SET_OPTIONS {}", "OK YES"),
        ("SESSION 8 TASK 1 EXECUTE", "OK YES"),
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 FINISHED", "OK NO"),
        ("SESSION 8 TASK 1 RUNNING", "OK NO"),
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 FINISHED", "OK NO"),
        ("SESSION 8 TASK 1 RUNNING", "OK YES"),
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 FINISHED", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 GET_RESULTS", "OK {\"counts\":{\"0\":4}}"),
        ("SESSION 8 TASK 1 EXISTS", "OK NO"),
    ]);
    block_on(async {
        let mut qrmi = resource(Some(8));
        assert_eq!(qrmi.task_start(payload()).await.unwrap(), "1");
        assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Queued);
        assert_eq!(
            qrmi.task_result("1").await.unwrap_err().kind(),
            QrmiErrorKind::TaskNotReady
        );
        assert_eq!(
            qrmi.task_result("1").await.unwrap().value,
            "{\"counts\":{\"0\":4}}"
        );
        assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Completed);
        qrmi.task_stop("1").await.unwrap();
        assert_eq!(
            qrmi.task_result("1").await.unwrap_err().kind(),
            QrmiErrorKind::TaskNotFound
        );
    });
    server.assert_done();
}

#[test]
fn rejected_submission_is_classified_and_cleaned_up() {
    let _lock = ENV_LOCK.lock().unwrap();
    for cancellation in ["OK YES", "OK NO", "ERROR worker cleanup failed"] {
        let server = Server::new(&[
            ("SESSION 8 TASK CREATE", "OK 1"),
            ("SESSION 8 TASK 1 SET_TYPE EXECUTE", "OK YES"),
            ("SESSION 8 TASK 1 SET_QUBITS 1", "OK NO"),
            ("SESSION 8 TASK 1 CANCEL", cancellation),
        ]);
        block_on(async {
            let error = resource(Some(8)).task_start(payload()).await.unwrap_err();
            assert_eq!(error.kind(), QrmiErrorKind::Other);
            assert!(error.to_string().contains("task 1 was created"));
            assert!(error.to_string().contains("set qubits"));
        });
        server.assert_done();
    }
}

#[test]
fn cancellation_acknowledgement_is_required_and_success_is_remembered() {
    let _lock = ENV_LOCK.lock().unwrap();
    for running in ["OK NO", "OK YES"] {
        let active = [
            ("SESSION 8 TASK 1 EXISTS", "OK YES"),
            ("SESSION 8 TASK 1 FAILED", "OK NO"),
            ("SESSION 8 TASK 1 FINISHED", "OK NO"),
            ("SESSION 8 TASK 1 RUNNING", running),
        ];
        let mut replies = active.to_vec();
        replies.push(("SESSION 8 TASK 1 CANCEL", "OK NO"));
        replies.extend(active);
        replies.extend(active);
        replies.push(("SESSION 8 TASK 1 CANCEL", "OK YES"));
        let server = Server::new(&replies);
        block_on(async {
            let mut qrmi = resource(Some(8));
            assert!(qrmi
                .task_stop("1")
                .await
                .unwrap_err()
                .to_string()
                .contains("refused to cancel active task"));
            qrmi.task_stop("1").await.unwrap();
            assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Cancelled);
            qrmi.task_stop("1").await.unwrap();
            assert_eq!(
                qrmi.task_result("1").await.unwrap_err().kind(),
                QrmiErrorKind::TaskNotReady
            );
        });
        server.assert_done();
    }
}

#[test]
fn cancellation_errors_preserve_queued_and_running_tasks() {
    let _lock = ENV_LOCK.lock().unwrap();
    for (running, status) in [
        ("OK NO", TaskStatus::Queued),
        ("OK YES", TaskStatus::Running),
    ] {
        let active = [
            ("SESSION 8 TASK 1 EXISTS", "OK YES"),
            ("SESSION 8 TASK 1 FAILED", "OK NO"),
            ("SESSION 8 TASK 1 FINISHED", "OK NO"),
            ("SESSION 8 TASK 1 RUNNING", running),
        ];
        let mut replies = active.to_vec();
        replies.push(("SESSION 8 TASK 1 CANCEL", "ERROR worker cleanup failed"));
        replies.extend(active);
        replies.extend(active);
        let server = Server::new(&replies);
        block_on(async {
            let mut qrmi = resource(Some(8));
            let error = qrmi.task_stop("1").await.unwrap_err();
            assert_eq!(error.kind(), QrmiErrorKind::Other);
            assert!(error.to_string().contains("worker cleanup failed"));
            assert_eq!(qrmi.task_status("1").await.unwrap(), status);
        });
        server.assert_done();
    }
}

#[test]
fn cancellation_race_with_completion_is_benign() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 FINISHED", "OK NO"),
        ("SESSION 8 TASK 1 RUNNING", "OK YES"),
        ("SESSION 8 TASK 1 CANCEL", "OK NO"),
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 FINISHED", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
    ]);
    block_on(async {
        let mut qrmi = resource(Some(8));
        qrmi.task_stop("1").await.unwrap();
        assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Completed);
    });
    server.assert_done();
}

#[test]
fn failed_and_missing_tasks_are_not_reported_as_cancelled() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK YES"),
        ("SESSION 8 TASK 2 EXISTS", "OK NO"),
        ("SESSION 8 TASK 2 EXISTS", "OK NO"),
        ("SESSION 8 TASK 3 EXISTS", "OK YES"),
        ("SESSION 8 TASK 3 FAILED", "ERROR removed"),
        ("SESSION 8 TASK 3 EXISTS", "OK NO"),
    ]);
    block_on(async {
        let mut qrmi = resource(Some(8));
        assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Failed);
        assert!(qrmi
            .task_result("1")
            .await
            .unwrap_err()
            .to_string()
            .contains("Failed"));
        qrmi.task_stop("1").await.unwrap();
        assert_eq!(
            qrmi.task_status("2").await.unwrap_err().kind(),
            QrmiErrorKind::TaskNotFound
        );
        assert_eq!(
            qrmi.task_stop("2").await.unwrap_err().kind(),
            QrmiErrorKind::TaskNotFound
        );
        assert_eq!(
            qrmi.task_status("3").await.unwrap_err().kind(),
            QrmiErrorKind::TaskNotFound
        );
    });
    server.assert_done();
}

#[test]
fn service_discovers_maestro_filters_unavailable_resources_and_keeps_session() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        (
            "API {\"version\":2,\"command\":\"status\"}",
            "ERROR Unknown command",
        ),
        ("PING", "OK YES"),
        (
            "API {\"version\":2,\"command\":\"status\"}",
            "ERROR Unknown command",
        ),
        ("PING", "OK NO"),
        ("SESSION CREATE", "OK 42"),
        ("SESSION 42 EXISTS", "OK YES"),
    ]);
    let _env: Vec<_> = [
        (
            "QRMI_JOB_QPU_RESOURCES",
            Some("sync_test_maestro,offline_maestro"),
        ),
        ("QRMI_JOB_QPU_TYPES", Some("maestro-local,maestro-local")),
        ("QRMI_LIST_DELIMITER", Some(",")),
        ("QRMI_PLUGIN_ERROR", None),
        ("sync_test_maestro_QRMI_JOB_ACQUISITION_TOKEN", None),
        ("offline_maestro_QRMI_JOB_ACQUISITION_TOKEN", None), // pragma: allowlist secret
    ]
    .into_iter()
    .map(|(key, value)| EnvGuard::set(key, value.map(std::ffi::OsStr::new)))
    .collect();
    block_on(async {
        let resource_type: ResourceType = serde_json::from_str("\"maestro-local\"").unwrap();
        assert_eq!(resource_type.as_str(), "maestro-local");
        let mut service = QRMIService::new().await.unwrap();
        assert_eq!(service.resources().len(), 1);
        assert!(service.resource("offline_maestro").is_none());
        let resource = service.resource("sync_test_maestro").unwrap();
        assert_eq!(resource.resource_id().await.unwrap(), "sync_test_maestro");
        assert_eq!(
            resource.resource_type().await.unwrap(),
            ResourceType::MaestroLocal
        );
        assert_eq!(resource.acquire().await.unwrap(), "42");
        assert_eq!(
            service
                .resource("sync_test_maestro")
                .unwrap()
                .acquire()
                .await
                .unwrap(),
            "42"
        );
    });
    server.assert_done();
}

#[test]
fn failure_published_between_status_queries_is_not_cached_as_success() {
    let _lock = ENV_LOCK.lock().unwrap();
    let server = Server::new(&[
        ("SESSION 8 TASK 1 EXISTS", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK NO"),
        ("SESSION 8 TASK 1 FINISHED", "OK YES"),
        ("SESSION 8 TASK 1 FAILED", "OK YES"),
    ]);
    block_on(async {
        let mut qrmi = resource(Some(8));
        assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Failed);
        assert_eq!(qrmi.task_status("1").await.unwrap(), TaskStatus::Failed);
    });
    server.assert_done();
}

#[test]
fn native_error_codes_have_actionable_qrmi_kinds() {
    let _lock = ENV_LOCK.lock().unwrap();
    for (code, kind) in [
        ("unsupported_capability", QrmiErrorKind::UnsupportedFunction),
        ("unsupported_operation", QrmiErrorKind::UnsupportedFunction),
        ("invalid_config", QrmiErrorKind::InvalidConfig),
        ("session_not_found", QrmiErrorKind::InvalidConfig),
        ("invalid_input", QrmiErrorKind::InvalidInput),
        ("task_not_found", QrmiErrorKind::TaskNotFound),
    ] {
        let response = serde_json::json!({"version":2,"ok":false,"error":{"code":code,"message":"review probe"}});
        let reply = format!("OK {response}");
        let server = Server::new(&[(
            r#"API {"version":2,"command":"logs","session_id":4,"task_id":8}"#,
            &reply,
        )]);
        let mut qrmi = resource(Some(4));
        let error = block_on(qrmi.task_logs("8")).unwrap_err();
        assert_eq!(error.kind(), kind, "{code}");
        assert!(error.to_string().contains("review probe"));
        server.assert_done();
    }
}

#[test]
fn resource_retains_negotiation_across_native_submissions() {
    let _lock = ENV_LOCK.lock().unwrap();
    let request = r#"{"schema_version":2,"operation":"execute"}"#;
    let submit =
        format!(r#"API {{"version":2,"command":"submit","session_id":4,"request":{request}}}"#);
    let server = Server::new(&[
        (
            r#"API {"version":2,"command":"capabilities"}"#,
            r#"OK {"version":2,"ok":true,"native":{"schema_version":2}}"#,
        ),
        (&submit, r#"OK {"version":2,"ok":true,"task_id":1}"#),
        (&submit, r#"OK {"version":2,"ok":true,"task_id":2}"#),
    ]);
    let mut qrmi = resource(Some(4));
    for task in ["1", "2"] {
        assert_eq!(
            block_on(qrmi.task_start(request_payload(request.into()))).unwrap(),
            task
        );
    }
    server.assert_done();
}

#[test]
fn resource_keeps_one_socket_and_native_submission_clears_reused_status() {
    let _lock = ENV_LOCK.lock().unwrap();
    let input = r#"{"schema_version":2,"operation":"execute"}"#;
    let submit = format!(
        "API {{\"version\":2,\"command\":\"submit\",\"session_id\":5,\"request\":{input}}}"
    );
    let original = Server::new(&[
        (
            "API {\"version\":2,\"command\":\"status\"}",
            "ERROR Unknown command",
        ),
        ("PING", "OK"),
        ("SESSION CREATE", "OK 5"),
        (
            "API {\"version\":2,\"command\":\"capabilities\"}",
            "OK {\"version\":2,\"ok\":true,\"native\":{\"schema_version\":2}}",
        ),
        (&submit, "OK {\"version\":2,\"ok\":true,\"task_id\":7}"),
        ("SESSION 5 TASK 7 EXISTS", "OK YES"),
        ("SESSION 5 TASK 7 FAILED", "OK NO"),
        ("SESSION 5 TASK 7 FINISHED", "OK NO"),
        ("SESSION 5 TASK 7 RUNNING", "OK YES"),
        (
            "API {\"version\":2,\"command\":\"logs\",\"session_id\":5,\"task_id\":7}",
            "OK {\"version\":2,\"ok\":true,\"logs\":\"captured\"}",
        ),
        ("SESSION 5 TASK 7 EXISTS", "OK YES"),
        ("SESSION 5 TASK 7 FAILED", "OK NO"),
        ("SESSION 5 TASK 7 FINISHED", "OK NO"),
        ("SESSION 5 TASK 7 RUNNING", "OK YES"),
        ("SESSION 5 TASK 7 CANCEL", "OK YES"),
        (&submit, "OK {\"version\":2,\"ok\":true,\"task_id\":7}"),
        ("SESSION 5 TASK 7 EXISTS", "OK YES"),
        ("SESSION 5 TASK 7 FAILED", "OK NO"),
        ("SESSION 5 TASK 7 FINISHED", "OK YES"),
        ("SESSION 5 TASK 7 FAILED", "OK NO"),
        ("SESSION 5 TASK 7 EXISTS", "OK YES"),
        ("SESSION 5 TASK 7 GET_RESULTS", "OK {\"counts\":{\"0\":1}}"),
        ("SESSION 5 DELETE", "OK YES"),
    ]);
    let mut qrmi = resource(None);
    let other = Server::new(&[]); // Changes the process-wide socket environment.
    assert_eq!(
        block_on(qrmi.status()).unwrap().status,
        crate::ResourceStatusCode::Online
    );
    let token = block_on(qrmi.acquire()).unwrap();
    block_on(qrmi.target()).unwrap();
    qrmi.terminal_tasks.insert((5, 7), TaskStatus::Completed);
    assert_eq!(
        block_on(qrmi.task_start(request_payload(input.into()))).unwrap(),
        "7"
    );
    assert_eq!(
        block_on(qrmi.task_status("7")).unwrap(),
        TaskStatus::Running
    );
    assert!(block_on(qrmi.task_logs("7")).unwrap().contains("captured"));
    block_on(qrmi.task_stop("7")).unwrap();
    assert_eq!(
        block_on(qrmi.task_start(request_payload(input.into()))).unwrap(),
        "7"
    );
    assert!(block_on(qrmi.task_result("7"))
        .unwrap()
        .value
        .contains("counts"));
    block_on(qrmi.release(&token)).unwrap();
    original.assert_done();
    other.assert_done();
}

#[test]
fn resource_status_reports_queue_without_acquiring_and_keeps_failures_visible() {
    let _lock = ENV_LOCK.lock().unwrap();
    const COMMAND: &str = "API {\"version\":2,\"command\":\"status\"}";
    for (accepting, healthy, reason) in [
        (true, true, None),
        (false, false, Some("runner_not_running")),
    ] {
        let response = format!(
            "OK {}",
            serde_json::json!({"version":2,"ok":true,"status":{
            "accepting_jobs":accepting,"healthy":healthy,"busy":true,
            "pending_job_count":3,"status_reason":reason}})
        );
        let server = Server::new(&[(COMMAND, &response)]);
        let mut resource = MaestroLocal::new("status_test").unwrap();
        let status = block_on(resource.status()).unwrap();
        assert_eq!(
            status.status == crate::models::ResourceStatusCode::Online,
            accepting
        );
        assert_eq!(status.healthy, Some(healthy));
        assert_eq!(status.busy, Some(true));
        assert_eq!(status.pending_job_count, Some(3));
        assert_eq!(status.status_reason.as_deref(), reason);
        assert_eq!(status.capacity, None);
        server.assert_done();
    }
    for response in [
        "OK {\"version\":2,\"ok\":true}",
        "OK {\"version\":2,\"ok\":true,\"status\":{\"accepting_jobs\":true,\"healthy\":true,\"busy\":false,\"pending_job_count\":-1}}",
        "OK {\"version\":2,\"ok\":false,\"error\":{\"code\":\"native_failure\",\"message\":\"broken\"}}",
        "OK {\"version\":3,\"ok\":true}",
        "OK not-json", "ERROR internal failure",
    ] {
        let server = Server::new(&[(COMMAND, response)]);
        assert!(block_on(MaestroLocal::new("status_test").unwrap().status()).is_err());
        server.assert_done();
    }
    let server = Server::new(&[
        (COMMAND, "OK {\"version\":2,\"ok\":false,\"error\":{\"code\":\"unsupported_operation\",\"message\":\"Unknown API command\"}}"),
        ("PING", "OK"),
    ]);
    let status = block_on(MaestroLocal::new("status_test").unwrap().status()).unwrap();
    assert_eq!(status.healthy, None);
    assert_eq!(status.busy, None);
    assert_eq!(status.pending_job_count, None);
    server.assert_done();
    let resource = MaestroLocal::from_config(
        "status_test",
        std::collections::HashMap::from([(
            "QRMI_MAESTRO_SOCKET".into(),
            "/nonexistent/maestro-status.sock".into(),
        )]),
    )
    .unwrap();
    assert!(block_on({
        let mut resource = resource;
        async move { resource.status().await }
    })
    .is_err());
}
