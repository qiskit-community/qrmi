// This code is part of Qiskit.
//
// (C) Copyright Alice and Bob, Pasqal 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::AliceBobFelis;
use crate::models::Payload;
use crate::{QrmiErrorKind, QuantumResource};
use alice_bob_felis::apis::configuration;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn felis(base_path: String) -> AliceBobFelis {
    let mut config = configuration::Configuration::new();
    config.base_path = base_path;
    AliceBobFelis {
        config,
        backend_name: "ab_emu_40q_physical_cats".to_string(),
        felis_target: "EMU:40Q:PHYSICAL_CATS".to_string(),
    }
}

fn payload(input_params: &str) -> Payload {
    Payload::AliceBobFelis {
        human_qir: "qir".to_string(),
        input_params: input_params.to_string(),
    }
}

#[test]
fn from_config_rejects_malformed_api_key() {
    let config = HashMap::from([
        (
            "QRMI_AB_FELIS_API_KEY".to_string(),
            "not base64!".to_string(),
        ),
        (
            "QRMI_AB_FELIS_BASE_ENDPOINT".to_string(),
            "http://127.0.0.1:9".to_string(),
        ),
    ]);
    let err = AliceBobFelis::from_config("ab_emu_40q_physical_cats", config)
        .err()
        .expect("malformed API key should fail");
    assert_eq!(err.kind(), QrmiErrorKind::InvalidConfig);
    assert!(!err.to_string().contains("not base64!"));
}

#[tokio::test]
async fn task_start_rejects_invalid_input_params_before_any_request() {
    // Port 9 (discard) is never contacted: parsing must fail first.
    let err = felis("http://127.0.0.1:9".to_string())
        .task_start(payload("not json"))
        .await
        .expect_err("invalid input_params should fail");
    assert_eq!(err.kind(), QrmiErrorKind::InvalidInput);
}

#[tokio::test]
async fn task_start_reports_upload_failure_after_job_creation_as_other() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let addr = listener.local_addr().expect("local_addr should succeed");
    // The job is created, then the input upload is rejected with 400.
    let server = thread::spawn(move || {
        let responses = [
            (
                "201 Created",
                r#"{"inputDataFormat":"HUMAN_QIR","outputDataFormat":"HISTOGRAM","target":"EMU:40Q:PHYSICAL_CATS","inputParams":{},"id":"job-1","userName":"u","userId":"1","organizationName":"o","events":[],"errors":[]}"#,
            ),
            ("400 Bad Request", r#"{"detail":"bad input"}"#),
        ];
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().expect("accept should succeed");
            let mut buf = [0_u8; 4096];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write should succeed");
        }
    });

    let err = felis(format!("http://{addr}"))
        .task_start(payload("{}"))
        .await
        .expect_err("upload failure should fail task_start");
    server.join().expect("mock server should finish");
    assert_eq!(err.kind(), QrmiErrorKind::Other);
    assert!(format!("{err:#}").contains("job-1"));
}
