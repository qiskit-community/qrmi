use crate::QrmiErrorKind;

use super::PasqalLocal;
use crate::models::Payload;
use crate::QuantumResource;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

/// Serve one HTTP request with `body` and return the Warden URL and the request text.
fn serve_once(body: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let url = format!(
        "http://{}",
        listener.local_addr().expect("local_addr should succeed")
    );
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept should succeed");
        let mut buf = [0_u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write should succeed");
        String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase()
    });
    (url, server)
}

fn local_config(url: &str) -> HashMap<String, String> {
    HashMap::from([
        ("QRMI_WARDEN_URL".to_string(), url.to_string()),
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ])
}

#[test]
fn from_config_accepts_env_style_keys_without_backend_prefix() {
    let config = HashMap::from([
        (
            "QRMI_WARDEN_URL".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    assert!(PasqalLocal::from_config("test_backend", config).is_ok());
}

#[test]
fn from_config_accepts_env_style_keys_without_backend_prefix_lowercased() {
    let config = HashMap::from([
        (
            "qrmi_warden_url".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("qrmi_job_uid".to_string(), "1000".to_string()),
        ("qrmi_job_id".to_string(), "1".to_string()),
    ]);
    assert!(PasqalLocal::from_config("test_backend", config).is_ok());
}

#[test]
fn from_config_errors_on_missing_key() {
    let config = HashMap::from([
        ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    assert!(PasqalLocal::from_config("test_backend", config).is_err());
}

#[test]
fn job_uid_parsing_fail_raises_qrmi_error() {
    let config = HashMap::from([
        (
            "QRMI_WARDEN_URL".to_string(),
            "http://localhost:8006".to_string(),
        ),
        ("QRMI_JOB_UID".to_string(), "abcd".to_string()),
        ("QRMI_JOB_ID".to_string(), "1".to_string()),
    ]);
    let expected = QrmiErrorKind::ParseError;
    let res = PasqalLocal::from_config("backend_name", config);
    let Err(err) = res else {
        panic!("expected an error passing 'abcd' to QRMI_JOB_UID, but got OK");
    };
    assert_eq!(expected, err.kind())
}

#[tokio::test]
async fn release_revokes_the_given_session() {
    let (url, server) = serve_once(r#"{"id":"session-arg"}"#);
    let mut qrmi = PasqalLocal::from_config("release_arg", local_config(&url)).unwrap();
    qrmi.release("session-arg")
        .await
        .expect("release should succeed");
    assert!(server
        .join()
        .unwrap()
        .starts_with("delete /sessions/session-arg "));
}

#[tokio::test]
async fn release_without_id_uses_environment() {
    let (url, server) = serve_once(r#"{"id":"session-env"}"#);
    let mut qrmi = PasqalLocal::from_config("release_env", local_config(&url)).unwrap();
    std::env::set_var("release_env_QRMI_JOB_ACQUISITION_TOKEN", "session-env");
    let result = qrmi.release("").await;
    std::env::remove_var("release_env_QRMI_JOB_ACQUISITION_TOKEN");
    result.expect("release should succeed");
    assert!(server
        .join()
        .unwrap()
        .starts_with("delete /sessions/session-env "));
}

#[tokio::test]
async fn task_start_uses_config_map_token() {
    let (url, server) =
        serve_once(r#"{"id":7,"user_id":"1000","status":"PENDING","results":null}"#);
    let mut config = local_config(&url);
    config.insert(
        "QRMI_JOB_ACQUISITION_TOKEN".to_string(),
        "session-map".to_string(),
    );
    let mut qrmi = PasqalLocal::from_config("task_start_map", config).unwrap();
    let payload = Payload::PasqalCloud {
        sequence: "{}".to_string(),
        job_runs: 1,
    };
    assert_eq!(qrmi.task_start(payload).await.unwrap(), "7");
    assert!(server
        .join()
        .unwrap()
        .contains("x-warden-session: session-map"));
}
