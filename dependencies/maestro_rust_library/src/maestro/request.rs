//! Native Maestro request API v2. The synchronous client captures its socket
//! path; asynchronous applications should run exchanges on a blocking executor.
use crate::maestro_lib;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub const VERSION: u32 = 2;
pub const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Error {
    pub code: String,
    pub message: String,
}
impl Error {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}

/// Server runner status; device readiness and acquisition capacity are unknown.
#[derive(Debug, Clone)]
pub struct Status {
    pub accepting_jobs: bool,
    pub healthy: Option<bool>,
    pub busy: Option<bool>,
    pub pending_job_count: Option<u64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Client {
    socket: PathBuf,
    // Share only successful version negotiation across clones, not a stale
    // backend catalog. The server validates every submitted feature/config.
    negotiated: Arc<AtomicBool>,
}
impl Default for Client {
    fn default() -> Self {
        Self::new(
            std::env::var_os("QRMI_MAESTRO_SOCKET")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/run/maestro.sock")),
        )
    }
}
impl Client {
    pub fn new(socket: impl Into<PathBuf>) -> Self {
        Self {
            socket: socket.into(),
            negotiated: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket
    }

    fn call(&self, document: json::JsonValue) -> Result<json::JsonValue, Error> {
        self.call_with(document, parse_response)
    }

    fn call_with(
        &self,
        document: json::JsonValue,
        parse: fn(&str) -> Result<json::JsonValue, Error>,
    ) -> Result<json::JsonValue, Error> {
        let command = format!("API {}\n", document.dump());
        if command.len() > MAX_REQUEST_BYTES {
            return Err(Error::new(
                "invalid_input",
                "Request exceeds the 16 MiB transport limit",
            ));
        }
        let result = maestro_lib::exchange(&self.socket, &command)
            .map_err(|error| Error::new("transport", error.to_string()))
            .and_then(|response| parse(&response));
        if result.as_ref().is_err_and(|error| {
            matches!(
                error.code.as_str(),
                "transport" | "protocol" | "unsupported_api" | "unsupported_version"
            )
        }) {
            self.negotiated.store(false, Ordering::SeqCst);
        }
        result
    }

    pub fn capabilities(&self) -> Result<String, Error> {
        // Explicit discovery always refreshes the catalog and negotiation.
        self.negotiated.store(false, Ordering::SeqCst);
        let value = self.call(json::object! { version: VERSION, command: "capabilities" })?;
        if value["native"]["schema_version"].as_u32() != Some(VERSION) {
            return Err(Error::new(
                "unsupported_api",
                "Server lacks Maestro native request schema 2",
            ));
        }
        self.negotiated.store(true, Ordering::SeqCst);
        Ok(value.dump())
    }

    pub fn submit(&self, session_id: u32, request: &str) -> Result<u32, Error> {
        let request =
            json::parse(request).map_err(|error| Error::new("invalid_input", error.to_string()))?;
        if !request.is_object() || request["schema_version"].as_u32() != Some(VERSION) {
            return Err(Error::new(
                "invalid_input",
                "Native request must be an object with schema_version=2",
            ));
        }
        if !self.negotiated.load(Ordering::SeqCst) {
            self.capabilities()?;
        }
        // Legacy-server detection belongs to negotiation above. Once the request
        // has been sent, only a well-formed v2 refusal proves that no task exists.
        let response = self.call_with(
            json::object! {
                version: VERSION, command: "submit", session_id: session_id, request: request,
            },
            parse_submission_response,
        )?;
        response["task_id"].as_u32().ok_or_else(|| {
            self.negotiated.store(false, Ordering::SeqCst);
            Error::new(
                "protocol",
                "Missing task_id; the task may have been created",
            )
        })
    }

    /// Returns the full diagnostics document, including structured native errors.
    pub fn logs(&self, session_id: u32, task_id: u32) -> Result<String, Error> {
        self.call(json::object! { version: VERSION, command: "logs", session_id: session_id, task_id: task_id })
            .map(|value| value.dump())
    }

    /// Queries status without creating or renewing a session. Only an explicitly
    /// unsupported command falls back to legacy PING; failures stay visible.
    pub fn status(&self) -> Result<Status, Error> {
        let response = maestro_lib::exchange(
            &self.socket,
            &format!(
                "API {}\n",
                json::object! {version: VERSION, command: "status"}.dump()
            ),
        )
        .map_err(|error| Error::new("transport", error.to_string()))?;
        let parsed = parse_response(&response);
        if response.trim() == "ERROR Unknown command"
            || parsed
                .as_ref()
                .is_err_and(|error| error.code == "unsupported_operation")
        {
            let ping = maestro_lib::exchange(&self.socket, "PING\n")
                .map_err(|error| Error::new("transport", error.to_string()))?;
            let accepting_jobs = match ping.trim() {
                "OK" | "OK YES" => true,
                "OK NO" => false,
                _ => {
                    return Err(Error::new(
                        "protocol",
                        format!("Invalid legacy PING response: {}", ping.trim()),
                    ));
                }
            };
            return Ok(Status {
                accepting_jobs,
                healthy: None,
                busy: None,
                pending_job_count: None,
                reason: None,
            });
        }
        let value = parsed?;
        let status = &value["status"];
        let malformed = || Error::new("protocol", "Invalid server status response");
        let reason = match &status["status_reason"] {
            json::JsonValue::Null => None,
            value => Some(value.as_str().ok_or_else(malformed)?.to_owned()),
        };
        Ok(Status {
            accepting_jobs: status["accepting_jobs"].as_bool().ok_or_else(malformed)?,
            healthy: Some(status["healthy"].as_bool().ok_or_else(malformed)?),
            busy: Some(status["busy"].as_bool().ok_or_else(malformed)?),
            pending_job_count: Some(status["pending_job_count"].as_u64().ok_or_else(malformed)?),
            reason,
        })
    }

    pub fn keepalive(&self, session_id: u32) -> Result<(), Error> {
        self.call(json::object! { version: VERSION, command: "keepalive", session_id: session_id })
            .map(|_| ())
    }
}

/// Replies to a submission. A reply that is not a well-formed API v2 envelope
/// does not show that the server refused the request, so it is a protocol
/// failure rather than an unsupported API: the task may already exist.
fn parse_submission_response(response: &str) -> Result<json::JsonValue, Error> {
    let envelope = response
        .strip_prefix("OK ")
        .and_then(|document| json::parse(document.trim()).ok());
    if envelope.is_none_or(|value| value["version"].as_u32() != Some(VERSION)) {
        let reply: String = response.trim().chars().take(256).collect();
        return Err(Error::new(
            "protocol",
            format!("Unrecognized reply to a submission; the task may have been created: {reply}"),
        ));
    }
    parse_response(response)
}

fn parse_response(response: &str) -> Result<json::JsonValue, Error> {
    let Some(document) = response.strip_prefix("OK ") else {
        return Err(Error::new(
            "unsupported_api",
            format!(
                "Maestro server does not support request API v2: {}",
                response.trim()
            ),
        ));
    };
    let value = json::parse(document.trim())
        .map_err(|error| Error::new("protocol", format!("Malformed API response: {error}")))?;
    if value["version"].as_u32() != Some(VERSION) {
        return Err(Error::new(
            "unsupported_api",
            "Incompatible Maestro public API version",
        ));
    }
    match value["ok"].as_bool() {
        Some(true) => Ok(value),
        Some(false) => Err(Error::new(
            value["error"]["code"].as_str().unwrap_or("server_error"),
            value["error"]["message"]
                .as_str()
                .unwrap_or("Unspecified Maestro error"),
        )),
        None => Err(Error::new("protocol", "Missing response status")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn multiline_request_survives_negotiation_and_atomic_submission() {
        let path = std::env::temp_dir().join(format!(
            "maestro-v2-{}-{}.sock",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let request = json::object! {schema_version:2, operation:"execute",
        circuit:{source:"OPENQASM 2.0;\n// comment\nqreg q[1];", num_qubits:1}};
        let expected = request.clone();
        let server = std::thread::spawn(move || {
            for command in ["capabilities", "submit"] {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                    .unwrap();
                let mut line = String::new();
                BufReader::new(socket.try_clone().unwrap())
                    .read_line(&mut line)
                    .unwrap();
                let message = json::parse(line.strip_prefix("API ").unwrap()).unwrap();
                assert_eq!(message["command"].as_str(), Some(command));
                let reply = if command == "submit" {
                    assert_eq!(message["request"], expected);
                    assert_eq!(message["session_id"].as_u32(), Some(5));
                    json::object! {version:2, ok:true, task_id:7}
                } else {
                    json::object! {version:2, ok:true, native:{schema_version:2}}
                };
                writeln!(socket, "OK {}", reply.dump()).unwrap();
            }
        });
        let result = Client::new(&path).submit(5, &request.dump());
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn oversized_frames_fail_locally_and_version_errors_keep_their_code() {
        let error = Client::new("/nonexistent/socket")
            .call(
                json::object! {version:2, command:"submit", request:"x".repeat(MAX_REQUEST_BYTES)},
            )
            .unwrap_err();
        assert_eq!(error.code, "invalid_input");
        let error = parse_response(r#"OK {"version":2,"ok":false,"error":{"code":"unsupported_version","message":"version retired"}}"#).unwrap_err();
        assert_eq!(error.code, "unsupported_version");
    }

    #[test]
    fn old_servers_errors_and_invalid_envelopes_are_explicit() {
        assert_eq!(
            parse_response("ERROR Unknown command\n").unwrap_err().code,
            "unsupported_api"
        );
        assert_eq!(parse_response("OK {\"version\":2,\"ok\":false,\"error\":{\"code\":\"invalid_input\",\"message\":\"unknown noise\"}}\n")
            .unwrap_err().message, "unknown noise");
        assert_eq!(
            parse_response("OK {\"version\":2}\n").unwrap_err().code,
            "protocol"
        );
        assert_eq!(parse_response("OK broken\n").unwrap_err().code, "protocol");
        assert!(
            Client::new("/nonexistent")
                .submit(1, "{}")
                .unwrap_err()
                .code
                == "invalid_input"
        );
    }

    #[test]
    fn malformed_submission_replies_never_claim_that_no_task_exists() {
        // Only a well-formed v2 refusal proves that submission created no task.
        for (reply, code) in [
            (r#"OK {"ok":true,"task_id":73}"#, "protocol"),
            ("garbled response", "protocol"),
            ("ERROR Unknown command", "protocol"),
            (r#"OK {"version":3,"ok":true,"task_id":73}"#, "protocol"),
            (r#"OK {"version":2,"ok":true}"#, "protocol"),
            (r#"OK {"version":2}"#, "protocol"),
            ("OK broken", "protocol"),
            (
                r#"OK {"version":2,"ok":false,"error":{"code":"invalid_input","message":"bad arity"}}"#,
                "invalid_input",
            ),
            (
                r#"OK {"version":2,"ok":false,"error":{"code":"unsupported_capability","message":"no GPU"}}"#,
                "unsupported_capability",
            ),
        ] {
            let path = std::env::temp_dir().join(format!(
                "maestro-submit-{}-{}.sock",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let listener = UnixListener::bind(&path).unwrap();
            let server = std::thread::spawn(move || {
                for (command, answer) in [
                    (
                        "capabilities",
                        r#"OK {"version":2,"ok":true,"native":{"schema_version":2}}"#,
                    ),
                    ("submit", reply),
                ] {
                    let (mut socket, _) = listener.accept().unwrap();
                    let mut line = String::new();
                    BufReader::new(socket.try_clone().unwrap())
                        .read_line(&mut line)
                        .unwrap();
                    let value = json::parse(line.strip_prefix("API ").unwrap()).unwrap();
                    assert_eq!(value["command"].as_str(), Some(command));
                    writeln!(socket, "{answer}").unwrap();
                }
            });
            let client = Client::new(&path);
            let error = client
                .submit(1, r#"{"schema_version":2,"operation":"execute"}"#)
                .unwrap_err();
            server.join().unwrap();
            std::fs::remove_file(path).unwrap();
            assert_eq!(error.code, code, "{reply}");
            assert!(!client.negotiated.load(Ordering::SeqCst) || code != "protocol");
        }
        // Negotiation still identifies a legacy server explicitly.
        assert_eq!(
            parse_response("ERROR Unknown command\n").unwrap_err().code,
            "unsupported_api"
        );
    }

    #[test]
    fn negotiation_is_shared_but_invalidated_after_server_or_transport_failure() {
        for failure in [
            None,
            Some("ERROR Unknown command"),
            Some(r#"OK {"version":1,"ok":true}"#),
            Some(r#"OK {"version":2,"ok":true}"#),
        ] {
            let path = std::env::temp_dir().join(format!(
                "maestro-cache-{}-{}.sock",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            let listener = UnixListener::bind(&path).unwrap();
            let server = std::thread::spawn(move || {
                let caps = r#"OK {"version":2,"ok":true,"native":{"schema_version":2}}"#;
                let submitted = r#"OK {"version":2,"ok":true,"task_id":7}"#;
                for (command, reply) in [
                    ("capabilities", Some(caps)),
                    ("submit", Some(submitted)),
                    ("submit", Some(submitted)),
                    ("submit", failure),
                    ("capabilities", Some(caps)),
                    ("submit", Some(submitted)),
                    ("capabilities", Some(caps)),
                ] {
                    let (mut socket, _) = listener.accept().unwrap();
                    socket
                        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                        .unwrap();
                    let mut line = String::new();
                    BufReader::new(socket.try_clone().unwrap())
                        .read_line(&mut line)
                        .unwrap();
                    let value = json::parse(line.strip_prefix("API ").unwrap()).unwrap();
                    assert_eq!(value["command"].as_str(), Some(command));
                    if let Some(reply) = reply {
                        writeln!(socket, "{reply}").unwrap();
                    }
                }
            });
            let client = Client::new(&path);
            let request = r#"{"schema_version":2,"operation":"execute"}"#;
            assert_eq!(client.submit(1, request).unwrap(), 7);
            assert_eq!(client.clone().submit(1, request).unwrap(), 7);
            assert!(client.submit(1, request).is_err()); // No automatic resubmission.
            assert_eq!(client.submit(1, request).unwrap(), 7);
            client.capabilities().unwrap(); // Explicit discovery remains fresh.
            server.join().unwrap();
            std::fs::remove_file(path).unwrap();
        }
    }
}
