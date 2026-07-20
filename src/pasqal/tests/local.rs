use super::PasqalLocal;
use crate::pasqal::env_lock;
use crate::QuantumResource;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const BACKEND: &str = "PASQAL_TEST";

/// Starts a server that answers every request with `503 Service Unavailable`,
/// counting the requests it receives. A 503 is a transient failure, so a client
/// with retries will come back, and one without will not.
fn spawn_failing_server() -> (SocketAddr, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local_addr should succeed");
    let requests = Arc::new(AtomicUsize::new(0));

    let counter = Arc::clone(&requests);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut buf = [0_u8; 4096];
            let _ = stream.read(&mut buf);
            counter.fetch_add(1, Ordering::SeqCst);

            let body = r#"{"detail":"service unavailable"}"#;
            let response = format!(
                "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    (addr, requests)
}

/// Builds a `PasqalLocal` pointed at `addr` via `construct`, with the retry env
/// vars cleared and `extra_env` applied on top.
///
/// The environment is process-wide, so it is set under `env_lock`. Retries are
/// decided at construction time, so the lock is released before the caller
/// awaits any request.
fn build_qrmi(
    addr: SocketAddr,
    extra_env: &[(String, &str)],
    construct: fn(&str) -> anyhow::Result<PasqalLocal>,
) -> PasqalLocal {
    let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());

    std::env::set_var(format!("{BACKEND}_QRMI_URL"), format!("http://{addr}"));
    std::env::set_var("QRMI_JOB_UID", "1000");
    std::env::set_var("QRMI_JOB_ID", "test-job");
    std::env::remove_var("QRMI_PASQAL_RETRIES_DISABLED");
    std::env::remove_var(format!("{BACKEND}_QRMI_PASQAL_RETRIES_DISABLED"));
    std::env::remove_var("QRMI_PASQAL_MAX_RETRY_COUNT");
    std::env::remove_var(format!("{BACKEND}_QRMI_PASQAL_MAX_RETRY_COUNT"));
    for (key, value) in extra_env {
        std::env::set_var(key, value);
    }

    let qrmi = construct(BACKEND).expect("PasqalLocal should build");

    for (key, _) in extra_env {
        std::env::remove_var(key);
    }
    qrmi
}

/// Constructs a `PasqalLocal` with the URL var set and `QRMI_JOB_UID` /
/// `QRMI_JOB_ID` set or removed per the arguments, restoring both afterwards.
///
/// Used to check that `PasqalLocal::new` reports a missing or malformed job
/// environment as an `Err` rather than panicking, which would surface to
/// Python users as a raw `PanicException` and could abort across the C
/// bindings.
fn build_with_job_env(job_uid: Option<&str>, job_id: Option<&str>) -> anyhow::Result<()> {
    let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());

    let vars = [("QRMI_JOB_UID", job_uid), ("QRMI_JOB_ID", job_id)];
    let saved: Vec<_> = vars
        .iter()
        .map(|(var, _)| (*var, std::env::var(var).ok()))
        .collect();

    std::env::set_var(format!("{BACKEND}_QRMI_URL"), "http://127.0.0.1:1");
    for (var, value) in vars {
        match value {
            Some(v) => std::env::set_var(var, v),
            None => std::env::remove_var(var),
        }
    }

    let result = PasqalLocal::new(BACKEND);

    for (var, value) in saved {
        match value {
            Some(v) => std::env::set_var(var, v),
            None => std::env::remove_var(var),
        }
    }
    result.map(|_| ())
}

#[test]
fn new_errors_when_job_uid_is_not_set() {
    let err = build_with_job_env(None, Some("test-job")).expect_err("build should fail");
    assert_eq!(
        err.to_string(),
        "QRMI_JOB_UID environment variable is not set"
    );
}

#[test]
fn new_errors_when_job_uid_is_not_an_integer() {
    let err =
        build_with_job_env(Some("not-a-number"), Some("test-job")).expect_err("build should fail");
    assert_eq!(
        err.to_string(),
        "QRMI_JOB_UID environment variable is not a valid integer: 'not-a-number'"
    );
}

#[test]
fn new_errors_when_job_id_is_not_set() {
    let err = build_with_job_env(Some("1000"), None).expect_err("build should fail");
    assert_eq!(
        err.to_string(),
        "QRMI_JOB_ID environment variable is not set"
    );
}

/// The C bindings (and hence the Slurm SPANK plugin) construct resources with
/// `PasqalLocal::new`, which must fail fast rather than stall job launch.
#[tokio::test]
async fn new_does_not_retry() {
    let (addr, requests) = spawn_failing_server();
    let mut qrmi = build_qrmi(addr, &[], PasqalLocal::new);

    let result = tokio::time::timeout(Duration::from_secs(5), qrmi.is_accessible()).await;

    assert!(
        result.is_ok(),
        "is_accessible should return promptly without retrying"
    );
    assert!(result.unwrap().is_err(), "a 503 should surface as an error");
    assert_eq!(
        requests.load(Ordering::SeqCst),
        1,
        "the non-retrying client should have made exactly one request"
    );
}

/// The Python bindings construct resources with `PasqalLocal::new_with_retries`,
/// so a job's own requests ride out a transient outage.
#[tokio::test]
async fn new_with_retries_retries() {
    let (addr, requests) = spawn_failing_server();
    let mut qrmi = build_qrmi(addr, &[], PasqalLocal::new_with_retries);

    // Exhausting the default five retries takes on the order of 20s of backoff,
    // so rather than wait it out we give the client long enough to make its
    // first retry (the backoff starts at one second) and then drop the request.
    let _ = tokio::time::timeout(Duration::from_secs(4), qrmi.is_accessible()).await;

    assert!(
        requests.load(Ordering::SeqCst) > 1,
        "the retrying client should have retried the failed request, but made only {} request(s)",
        requests.load(Ordering::SeqCst)
    );
}

/// Retries stay configurable for the Python bindings: a job can opt a backend
/// out with `<backend>_QRMI_PASQAL_RETRIES_DISABLED`.
#[tokio::test]
async fn new_with_retries_honors_env_opt_out() {
    let (addr, requests) = spawn_failing_server();
    let opt_out = [(format!("{BACKEND}_QRMI_PASQAL_RETRIES_DISABLED"), "true")];
    let mut qrmi = build_qrmi(addr, &opt_out, PasqalLocal::new_with_retries);

    let result = tokio::time::timeout(Duration::from_secs(5), qrmi.is_accessible()).await;

    assert!(result.is_ok(), "the opted-out client should not retry");
    assert_eq!(
        requests.load(Ordering::SeqCst),
        1,
        "the opted-out client should have made exactly one request"
    );
}

/// The retry count stays configurable for the Python bindings: a job can cap how
/// many times a backend is retried with `<backend>_QRMI_PASQAL_MAX_RETRY_COUNT`.
#[tokio::test]
async fn new_with_retries_honors_env_max_retry_count() {
    let (addr, requests) = spawn_failing_server();
    let one_retry = [(format!("{BACKEND}_QRMI_PASQAL_MAX_RETRY_COUNT"), "1")];
    let mut qrmi = build_qrmi(addr, &one_retry, PasqalLocal::new_with_retries);

    // The single retry waits out the 1s base backoff, so 10s is ample for the
    // client to exhaust its retry count and return.
    let result = tokio::time::timeout(Duration::from_secs(10), qrmi.is_accessible()).await;

    assert!(result.is_ok(), "the client should stop after its one retry");
    assert_eq!(
        requests.load(Ordering::SeqCst),
        2,
        "the client should have made the original request and exactly one retry"
    );
}
