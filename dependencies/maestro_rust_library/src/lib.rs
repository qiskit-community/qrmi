// This code is part of Qiskit.
//
// (C) Copyright Qoro Quantum 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

// This crate is a library
#![crate_type = "lib"]
// The library is named "maestro-local-api"
#![crate_name = "maestro_local_api"]

pub mod maestro;

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use std::time::Duration;

pub mod maestro_lib {
    use super::*;

    #[derive(Debug)]
    pub enum Response {
        OK(bool),
        ERROR(String),
        OkResponse(String),
    }

    pub fn ping() -> Response {
        send_command_close("PING\n")
    }

    pub fn socket_path() -> std::path::PathBuf {
        std::env::var_os("QRMI_MAESTRO_SOCKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "/run/maestro.sock".into())
    }

    pub fn send_command_close(command: &str) -> Response {
        send_command_at(&socket_path(), command)
    }

    pub fn send_command_at(socket_path: &Path, command: &str) -> Response {
        match exchange(socket_path, command) {
            Ok(response) => parse_response(&response),
            Err(error) => {
                Response::ERROR(format!("Maestro socket {}: {error}", socket_path.display()))
            }
        }
    }

    /// Largest task result a Maestro server sends (its `server.max_result_bytes`):
    /// Maestro's 64 MiB bound on a whole native result plus metadata headroom.
    pub const MAX_RESULT_BYTES: u64 = 64 * 1024 * 1024 + 64 * 1024;
    /// A reply line wraps a result in `OK ` and a newline; the rest is spare.
    pub const MAX_RESPONSE_BYTES: u64 = MAX_RESULT_BYTES + 4096;

    // Unit tests shorten both timeouts so delayed replies can be exercised quickly.
    const REPLY_TIMEOUT: Duration = Duration::from_secs(if cfg!(test) { 1 } else { 15 });
    // The server answers CANCEL and session DELETE once the active worker has
    // stopped. An MPI launcher gets 20 s to clean up remote ranks and 5 s more
    // before it is reaped; this allows that budget plus transport headroom.
    const CLEANUP_REPLY_TIMEOUT: Duration = Duration::from_secs(if cfg!(test) { 3 } else { 60 });

    /// How long to wait for the reply to `command`.
    fn reply_timeout(command: &str) -> Duration {
        match command.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["SESSION", _, "DELETE"] | ["SESSION", _, "TASK", _, "CANCEL"] => CLEANUP_REPLY_TIMEOUT,
            _ => REPLY_TIMEOUT,
        }
    }

    pub(crate) fn exchange(socket_path: &Path, command: &str) -> std::io::Result<String> {
        let mut stream = UnixStream::connect(socket_path)?;
        stream.set_read_timeout(Some(reply_timeout(command)))?;
        stream.set_write_timeout(Some(REPLY_TIMEOUT))?;
        stream.write_all(command.as_bytes())?;
        stream.shutdown(std::net::Shutdown::Write)?;
        let mut response = String::new();
        if BufReader::new(stream)
            .take(MAX_RESPONSE_BYTES + 1)
            .read_line(&mut response)?
            == 0
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "server closed the connection without a response",
            ));
        }
        if response.len() as u64 > MAX_RESPONSE_BYTES || !response.ends_with('\n') {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Oversized or truncated Maestro response",
            ));
        }
        Ok(response)
    }

    fn parse_response(response: &str) -> Response {
        let response = response.trim();
        if response == "OK" {
            return Response::OK(true);
        }
        if let Some(rest) = response.strip_prefix("OK ") {
            match rest.trim() {
                "YES" | "" => Response::OK(true),
                "NO" => Response::OK(false),
                value => Response::OkResponse(value.to_string()),
            }
        } else if let Some(rest) = response.strip_prefix("ERROR ") {
            Response::ERROR(rest.trim().to_string())
        } else {
            Response::ERROR(format!("Unexpected response: {response:?}"))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::unix::net::UnixListener;

        // Replies once to `command` with `reply` after `delay`, and returns what
        // the client received.
        fn delayed_reply(
            name: &str,
            command: &str,
            delay: Duration,
            reply: Vec<u8>,
        ) -> std::io::Result<String> {
            let path = std::env::temp_dir().join(format!(
                "maestro-response-{}-{name}.sock",
                std::process::id()
            ));
            let _ = std::fs::remove_file(&path);
            let listener = UnixListener::bind(&path)?;
            let server = std::thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                let mut command = String::new();
                BufReader::new(socket.try_clone().unwrap())
                    .read_line(&mut command)
                    .unwrap();
                std::thread::sleep(delay);
                // The client stops reading at its limit or timeout; a closed socket is fine.
                let _ = socket.write_all(&reply);
            });
            let response = exchange(&path, command);
            server.join().unwrap();
            std::fs::remove_file(path)?;
            response
        }

        fn exchange_reply(name: &str, reply: Vec<u8>) -> std::io::Result<String> {
            delayed_reply(
                name,
                "SESSION 1 TASK 1 GET_RESULTS\n",
                Duration::ZERO,
                reply,
            )
        }

        #[test]
        fn cleanup_replies_get_the_server_cleanup_budget() {
            assert_eq!(
                reply_timeout("SESSION 3 TASK 4 CANCEL\n"),
                CLEANUP_REPLY_TIMEOUT
            );
            assert_eq!(reply_timeout("SESSION 3 DELETE\n"), CLEANUP_REPLY_TIMEOUT);
            assert_eq!(reply_timeout("SESSION 3 TASK 4 EXISTS\n"), REPLY_TIMEOUT);
            assert_eq!(reply_timeout("PING\n"), REPLY_TIMEOUT);
            assert!(CLEANUP_REPLY_TIMEOUT > REPLY_TIMEOUT);

            // A cancellation or deletion acknowledged after the ordinary reply
            // timeout, while the server still cleans up, is not lost.
            let late = REPLY_TIMEOUT + Duration::from_millis(500);
            for (name, command) in [
                ("late-cancel", "SESSION 3 TASK 4 CANCEL\n"),
                ("late-delete", "SESSION 3 DELETE\n"),
            ] {
                let response = delayed_reply(name, command, late, b"OK YES\n".to_vec()).unwrap();
                assert!(matches!(parse_response(&response), Response::OK(true)));
            }
            // A genuine cleanup failure is reported, not mistaken for success.
            let response = delayed_reply(
                "failed-cancel",
                "SESSION 3 TASK 4 CANCEL\n",
                late,
                b"ERROR Could not cancel task: cleanup_unconfirmed\n".to_vec(),
            )
            .unwrap();
            assert!(
                matches!(parse_response(&response), Response::ERROR(message) if message.contains("cleanup_unconfirmed"))
            );
            // Other commands keep the short timeout.
            let error = delayed_reply(
                "late-exists",
                "SESSION 3 TASK 4 EXISTS\n",
                late,
                b"OK YES\n".to_vec(),
            )
            .unwrap_err();
            assert!(
                matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ),
                "{error}"
            );
        }

        #[test]
        fn accepts_a_result_at_the_server_result_bound() {
            // A quote-heavy result at the bound, framed exactly as the server does.
            let size = MAX_RESULT_BYTES as usize;
            let mut reply = Vec::with_capacity(size + 4);
            reply.extend_from_slice(b"OK {\"k\":\"");
            while reply.len() < size + 3 - 2 {
                reply.extend_from_slice(b"\\\"");
            }
            reply.extend_from_slice(b"\"}\n");
            assert_eq!(reply.len(), size + 4);
            let response = exchange_reply("bound", reply).unwrap();
            assert_eq!(response.len(), size + 4);
            assert!(
                matches!(parse_response(&response), Response::OkResponse(value) if value.len() == size)
            );

            let oversized = vec![b'x'; MAX_RESPONSE_BYTES as usize + 1];
            let error = exchange_reply("oversized", oversized).unwrap_err();
            assert!(error.to_string().contains("Oversized"), "{error}");
        }

        #[test]
        fn parses_acknowledgements_values_and_errors() {
            assert!(matches!(parse_response("OK YES\r\n"), Response::OK(true)));
            assert!(matches!(parse_response("OK\n"), Response::OK(true)));
            assert!(matches!(parse_response("OK NO\n"), Response::OK(false)));
            assert!(matches!(parse_response("OK 123\n"), Response::OkResponse(id) if id == "123"));
            assert!(
                matches!(parse_response("ERROR refused\n"), Response::ERROR(message) if message == "refused")
            );
            assert!(
                matches!(parse_response("broken\n"), Response::ERROR(message) if message.contains("Unexpected response"))
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires a running Maestro server at /run/maestro.sock"]
    fn simple_test() {
        let response = maestro_lib::send_command_close("SESSION 0 EXISTS");
        assert!(
            matches!(response, maestro_lib::Response::OK(false)),
            "Expected OK NO response, got: {:?}",
            response
        );

        let ping_response = maestro_lib::ping();
        assert!(
            matches!(ping_response, maestro_lib::Response::OK(true)),
            "Expected OK YES response, got: {:?}",
            ping_response
        );
    }
}
