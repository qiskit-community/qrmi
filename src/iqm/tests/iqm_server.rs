// This code is part of Qiskit.
//
// (C) Copyright IBM 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use super::super::IQMServer;
use crate::models::ResourceType;
use crate::{QrmiError, QuantumResource};
use iqm_server_api::apis::configuration;
use std::collections::HashMap;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "sirius:mock";

    // Create a service instance with dummy configuration for testing
    // Note: The configuration values are not used in this test since we are only
    // testing the resource_id and resource_type methods.
    // hence their values don't matter
    let mut qrmi = IQMServer {
        config: configuration::Configuration::new(),
        backend_name: BACKEND_NAME.to_string(),
        acquisition_token: None,
        calibration_set_id: "default".to_string(),
    };

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");
    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, BACKEND_NAME);
    assert_eq!(resource_type, ResourceType::IQMServer);
}

fn valid_config() -> HashMap<String, String> {
    HashMap::from([
        (
            "QRMI_IQM_ISA_ENDPOINT".to_string(),
            "http://localhost:8080".to_string(),
        ),
        ("QRMI_IQM_ISA_TOKEN".to_string(), "test-token".to_string()),
    ])
}

#[test]
fn from_config_builds_resource_from_map() {
    let qrmi =
        IQMServer::from_config("sirius_mock", valid_config()).expect("from_config should succeed");
    assert_eq!(qrmi.calibration_set_id, "default");
    assert_eq!(qrmi.acquisition_token, None);
}

#[test]
fn from_config_accepts_env_style_keys_lowercased() {
    let config: HashMap<String, String> = valid_config()
        .into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    let qrmi = IQMServer::from_config("sirius_mock", config)
        .expect("from_config should succeed with lowercase keys");
    assert_eq!(qrmi.calibration_set_id, "default");
    assert_eq!(qrmi.acquisition_token, None);
}

#[test]
fn from_config_parses_calibration_set_id_from_backend_name() {
    let config = valid_config();

    let qrmi =
        IQMServer::from_config("sirius_mock,custom", config).expect("from_config should succeed");
    assert_eq!(qrmi.backend_name, "sirius:mock");
    assert_eq!(qrmi.calibration_set_id, "custom");
}

#[test]
fn from_config_honors_acquisition_token() {
    let mut config = valid_config();
    config.insert(
        "QRMI_JOB_ACQUISITION_TOKEN".to_string(),
        "tok-123".to_string(),
    );

    let qrmi = IQMServer::from_config("sirius_mock", config).expect("from_config should succeed");
    assert_eq!(qrmi.acquisition_token, Some("tok-123".to_string()));
}

#[test]
fn from_config_missing_isa_endpoint() {
    let mut config = valid_config();
    config.remove("QRMI_IQM_ISA_ENDPOINT");
    let err = IQMServer::from_config("sirius_mock", config)
        .map(|_| ())
        .unwrap_err();
    assert!(matches!(err, QrmiError::MissingConfigKey(key) if key == "QRMI_IQM_ISA_ENDPOINT"));
}

// ---------------------------------------------------------------------------
// status(): parsing of `GET /api/v1/quantum-computers/{qc}` responses
// ---------------------------------------------------------------------------

use crate::models::ResourceStatusCode;
use serde_json::{json, Value};

/// A response in the IQM Server API spec shape (every required field
/// present), with the given `operational`, `health` and `queue_length`.
fn spec_qc_details(operational: &str, health: Value, queue_length: i64) -> Value {
    json!({
        "id": "0b5a0f36-1a8b-4d57-8a0e-0f6f1f3f6c01",
        "alias": "garnet",
        "display_name": "Garnet",
        "description": "20-qubit QPU",
        "backend_type": "qpu",
        "number_of_qubits": 20,
        "pulla_enabled": false,
        "limits": {
            "max_instructions_per_circuit": 10000,
            "max_shots_per_circuit": 100000,
            "max_circuits_per_job": 200,
            "max_executions_per_job": 1000000,
            "max_queued_jobs_per_user": 10
        },
        "operational_status": operational,
        "health": health,
        "queue_length": queue_length,
        "pricing": {"payg_price_per_second": "0.30"},
        "additional_info": {
            "academy_url": "https://example.com/academy",
            "best_suited_for": [],
            "documentation_url": "https://example.com/docs",
            "mock_url": "https://example.com/mock"
        }
    })
}

/// The older on-premises shape reported in the issue (ORNL): health under
/// `status.health`, no top-level `operational`, `health` or `queue_length`.
fn legacy_qc_details(health: Value) -> Value {
    json!({
        "id": "0b5a0f36-1a8b-4d57-8a0e-0f6f1f3f6c02",
        "alias": "default",
        "display_name": "On-prem QPU",
        "description": "20-qubit QPU",
        "backend_type": "qpu",
        "number_of_qubits": 20,
        "pulla_enabled": true,
        "limits": {
            "max_instructions_per_circuit": 10000,
            "max_shots_per_circuit": 100000,
            "max_circuits_per_job": 200,
            "max_executions_per_job": 1000000,
            "max_queued_jobs_per_user": 10
        },
        "status": {"health": health},
        "pricing": {"payg_price_per_second": "0"},
        "additional_info": {
            "academy_url": "https://example.com/academy",
            "best_suited_for": [],
            "documentation_url": "https://example.com/docs",
            "mock_url": "https://example.com/mock"
        }
    })
}

fn health(healthy: bool) -> Value {
    json!({"healthy": healthy, "updated_at": "2026-09-28T15:16:40.483662Z"})
}

#[test]
fn status_spec_online_healthy() {
    let st = IQMServer::resource_status_from_qc_details(spec_qc_details("online", health(true), 3))
        .expect("spec response should parse");
    assert_eq!(st.status, ResourceStatusCode::Online);
    assert_eq!(st.healthy, Some(true));
    assert_eq!(st.pending_job_count, Some(3));
    assert!(st.status_reason.is_some());
}

#[test]
fn status_spec_online_unhealthy_stays_online() {
    let st =
        IQMServer::resource_status_from_qc_details(spec_qc_details("online", health(false), 0))
            .expect("spec response should parse");
    assert_eq!(st.status, ResourceStatusCode::Online);
    assert_eq!(st.healthy, Some(false));
}

#[test]
fn status_spec_maintenance_with_null_health_is_paused() {
    let st =
        IQMServer::resource_status_from_qc_details(spec_qc_details("maintenance", Value::Null, 0))
            .expect("null health is allowed by the spec");
    assert_eq!(st.status, ResourceStatusCode::Paused);
    assert_eq!(st.healthy, None);
    assert_eq!(st.status_reason, None);
}

#[test]
fn status_spec_health_absent_is_allowed() {
    // `health` is not in the spec's required list.
    let mut body = spec_qc_details("online", Value::Null, 1);
    body.as_object_mut().unwrap().remove("health");
    let st = IQMServer::resource_status_from_qc_details(body)
        .expect("absent health is allowed by the spec");
    assert_eq!(st.status, ResourceStatusCode::Online);
    assert_eq!(st.healthy, None);
}

#[test]
fn status_spec_accepts_operational_key() {
    let mut body = spec_qc_details("online", health(true), 0);
    let obj = body.as_object_mut().unwrap();
    obj.remove("operational_status");
    obj.insert("operational".into(), json!("maintenance"));
    let st = IQMServer::resource_status_from_qc_details(body).expect("should parse");
    assert_eq!(st.status, ResourceStatusCode::Paused);
}

#[test]
fn status_legacy_nested_health() {
    let st = IQMServer::resource_status_from_qc_details(legacy_qc_details(health(true)))
        .expect("legacy status.health shape should parse");
    assert_eq!(st.status, ResourceStatusCode::Online);
    assert_eq!(st.healthy, Some(true));
    assert_eq!(st.pending_job_count, None);
}

#[test]
fn status_legacy_nested_unhealthy() {
    let st = IQMServer::resource_status_from_qc_details(legacy_qc_details(health(false)))
        .expect("legacy status.health shape should parse");
    assert_eq!(st.healthy, Some(false));
}

#[test]
fn status_legacy_null_health() {
    let st = IQMServer::resource_status_from_qc_details(legacy_qc_details(Value::Null))
        .expect("legacy shape with null health should parse");
    assert_eq!(st.healthy, None);
}

#[test]
fn status_spec_missing_queue_length_without_legacy_health_is_error() {
    // Not the spec shape (queue_length is required) and not the legacy
    // shape (no status.health): must be reported, not guessed at.
    let mut body = spec_qc_details("online", health(true), 0);
    body.as_object_mut().unwrap().remove("queue_length");
    let err = IQMServer::resource_status_from_qc_details(body)
        .expect_err("a body matching neither shape should fail");
    assert!(err.to_string().contains("queue_length"), "{err}");
}

#[test]
fn status_legacy_malformed_health_is_error() {
    let err =
        IQMServer::resource_status_from_qc_details(legacy_qc_details(json!({"healthy": true})))
            .expect_err("health without updated_at should fail");
    assert!(err.to_string().contains("updated_at"), "{err}");
}

// ---------------------------------------------------------------------------
// status() / is_accessible() end to end, against a one-shot local HTTP server
// ---------------------------------------------------------------------------

/// Serves exactly one HTTP response with the given status line and JSON
/// body, and returns an `IQMServer` pointed at it plus a handle yielding the
/// request line the server received.
async fn one_shot_server(
    status_line: &'static str,
    body: Value,
) -> (IQMServer, tokio::task::JoinHandle<String>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 8192];
        let n = sock.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]).to_string();
        let body = body.to_string();
        let resp = format!(
            "HTTP/1.1 {status_line}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        sock.write_all(resp.as_bytes()).await.unwrap();
        request
    });
    let mut config = configuration::Configuration::new();
    config.base_path = format!("http://{addr}");
    config.bearer_access_token = Some("test-token".to_string());
    let qrmi = IQMServer {
        config,
        backend_name: "garnet".to_string(),
        acquisition_token: None,
        calibration_set_id: "default".to_string(),
    };
    (qrmi, handle)
}

#[tokio::test]
async fn status_fetches_qc_details_endpoint() {
    let (mut qrmi, server) =
        one_shot_server("200 OK", spec_qc_details("online", health(true), 2)).await;
    let st = qrmi.status().await.expect("status should succeed");
    assert_eq!(st.status, ResourceStatusCode::Online);
    assert_eq!(st.pending_job_count, Some(2));

    let request = server.await.unwrap();
    assert!(
        request.starts_with("GET /api/v1/quantum-computers/garnet HTTP/1.1"),
        "{request}"
    );
    assert!(
        request
            .to_ascii_lowercase()
            .contains("authorization: bearer test-token"),
        "{request}"
    );
}

#[tokio::test]
#[allow(deprecated)]
async fn is_accessible_false_when_online_but_unhealthy() {
    let (mut qrmi, _server) =
        one_shot_server("200 OK", spec_qc_details("online", health(false), 0)).await;
    assert!(!qrmi.is_accessible().await.expect("should succeed"));
}

#[tokio::test]
#[allow(deprecated)]
async fn is_accessible_true_on_legacy_server() {
    let (mut qrmi, _server) = one_shot_server("200 OK", legacy_qc_details(health(true))).await;
    assert!(qrmi.is_accessible().await.expect("should succeed"));
}

#[tokio::test]
async fn status_maps_404_to_resource_not_found() {
    let (mut qrmi, _server) = one_shot_server(
        "404 Not Found",
        json!({"message": "Quantum computer not found"}),
    )
    .await;
    let err = qrmi.status().await.expect_err("404 should fail");
    assert!(matches!(err, QrmiError::ResourceNotFound(_)), "{err:?}");
}

#[tokio::test]
async fn status_maps_401_to_authentication_failed() {
    let (mut qrmi, _server) =
        one_shot_server("401 Unauthorized", json!({"message": "Unauthorized"})).await;
    let err = qrmi.status().await.expect_err("401 should fail");
    assert!(matches!(err, QrmiError::AuthenticationFailed(_)), "{err:?}");
}

#[tokio::test]
#[allow(deprecated)]
async fn is_accessible_true_when_online_and_healthy() {
    let (mut qrmi, _server) =
        one_shot_server("200 OK", spec_qc_details("online", health(true), 0)).await;
    assert!(qrmi.is_accessible().await.expect("should succeed"));
}

#[tokio::test]
#[allow(deprecated)]
async fn is_accessible_false_when_online_but_health_null() {
    let (mut qrmi, _server) =
        one_shot_server("200 OK", spec_qc_details("online", Value::Null, 0)).await;
    assert!(!qrmi.is_accessible().await.expect("should succeed"));
}

#[tokio::test]
#[allow(deprecated)]
async fn is_accessible_false_under_maintenance() {
    let (mut qrmi, _server) =
        one_shot_server("200 OK", spec_qc_details("maintenance", Value::Null, 0)).await;
    assert!(!qrmi.is_accessible().await.expect("should succeed"));
}

#[tokio::test]
#[allow(deprecated)]
async fn is_accessible_false_on_unhealthy_legacy_server() {
    let (mut qrmi, _server) = one_shot_server("200 OK", legacy_qc_details(health(false))).await;
    assert!(!qrmi.is_accessible().await.expect("should succeed"));
}

// ---------------------------------------------------------------------------
// New vs old IQM Server payloads (`GET /api/v1/quantum-computers/{qc}`),
// loaded from JSON fixtures in `tests/fixtures/` (see its README.md). Replace
// a fixture with a response captured from a real server to check it.
// ---------------------------------------------------------------------------

/// New IQM Server (API spec v1): top-level `operational_status`, `health`
/// and `queue_length`.
const NEW_SERVER_ONLINE_HEALTHY: &str = include_str!("fixtures/new_server_online_healthy.json");

/// New IQM Server, online but unhealthy.
const NEW_SERVER_ONLINE_UNHEALTHY: &str = include_str!("fixtures/new_server_online_unhealthy.json");

/// New IQM Server under maintenance: `health` is `null`.
const NEW_SERVER_MAINTENANCE: &str = include_str!("fixtures/new_server_maintenance.json");

/// Old on-prem IQM Server: the payload reported in the issue from ORNL's
/// 20-qubit system, as posted. The issue elided the values (`<uuid>`,
/// `<str>`, `<int>`, `<bool>`); those are filled with placeholders here,
/// and the key layout and `updated_at` are exactly as reported:
///
/// ```text
/// {"id": <uuid>, "alias": "default", "display_name": <str>, "description": <str>,
///  "backend_type": "qpu", "number_of_qubits": 20, "pulla_enabled": <bool>,
///  "limits": {"max_instructions_per_circuit": <int>, "max_shots_per_circuit": <int>,
///             "max_circuits_per_job": <int>, "max_executions_per_job": <int>,
///             "max_queued_jobs_per_user": <int>},
///  "status": {"health": {"healthy": true, "updated_at": "2026-09-28T15:16:40.483662Z"}},
///  "pricing": {"payg_price_per_second": <str>},
///  "additional_info": {"academy_url": <str>, "best_suited_for": [],
///                      "documentation_url": <str>, "mock_url": <str>}}
/// ```
///
/// No `operational` / `operational_status`, health nested under
/// `status.health`, no `queue_length`. In 0.25.1 both `status()` and
/// `is_accessible()` failed on it with ``missing field `health` ``.
const REPORTED_ORNL_PAYLOAD: &str = include_str!("fixtures/reported_ornl_payload.json");

/// [`REPORTED_ORNL_PAYLOAD`] with `healthy: false`.
const OLD_SERVER_ORNL_UNHEALTHY: &str = include_str!("fixtures/old_server_ornl_unhealthy.json");

// The issue's reproduction stub (`iqm_status_stub.py`) served
// `GET /api/v1/quantum-computers/{alias}` "in the shape of QRMI's own
// IqmServerQuantumComputerDetails model", i.e. as that model serializes:
// status under `operational` (not `operational_status`). Reported 0.25.1
// output for each is noted on the constant.

/// Stub "online, healthy". 0.25.1: `status() -> {'status': 'online',
/// 'healthy': True}`, `is_accessible() -> True`.
const REPORTED_STUB_ONLINE_HEALTHY: &str =
    include_str!("fixtures/reported_stub_online_healthy.json");

/// Stub "maintenance, health null". 0.25.1: both `status()` and
/// `is_accessible()` raised `invalid type: null, expected struct
/// QcHealthDetail`.
const REPORTED_STUB_MAINTENANCE_HEALTH_NULL: &str =
    include_str!("fixtures/reported_stub_maintenance_health_null.json");

/// Stub "online, unhealthy". 0.25.1: `status() -> {'status': 'online',
/// 'healthy': False}`, but `is_accessible() -> True`.
const REPORTED_STUB_ONLINE_UNHEALTHY: &str =
    include_str!("fixtures/reported_stub_online_unhealthy.json");

fn parse_payload(raw: &str) -> Value {
    serde_json::from_str(raw).expect("hardcoded payload should be valid JSON")
}

/// (payload name, raw body, status, healthy, pending_job_count, is_accessible)
type PayloadCase = (
    &'static str,
    &'static str,
    ResourceStatusCode,
    Option<bool>,
    Option<u64>,
    bool,
);

const PAYLOAD_CASES: &[PayloadCase] = &[
    (
        "REPORTED_STUB_ONLINE_HEALTHY",
        REPORTED_STUB_ONLINE_HEALTHY,
        ResourceStatusCode::Online,
        Some(true),
        Some(0),
        true,
    ),
    (
        "REPORTED_STUB_MAINTENANCE_HEALTH_NULL",
        REPORTED_STUB_MAINTENANCE_HEALTH_NULL,
        ResourceStatusCode::Paused,
        None,
        Some(0),
        false,
    ),
    (
        "REPORTED_STUB_ONLINE_UNHEALTHY",
        REPORTED_STUB_ONLINE_UNHEALTHY,
        ResourceStatusCode::Online,
        Some(false),
        Some(0),
        false,
    ),
    (
        "NEW_SERVER_ONLINE_HEALTHY",
        NEW_SERVER_ONLINE_HEALTHY,
        ResourceStatusCode::Online,
        Some(true),
        Some(3),
        true,
    ),
    (
        "NEW_SERVER_ONLINE_UNHEALTHY",
        NEW_SERVER_ONLINE_UNHEALTHY,
        ResourceStatusCode::Online,
        Some(false),
        Some(0),
        false,
    ),
    (
        "NEW_SERVER_MAINTENANCE",
        NEW_SERVER_MAINTENANCE,
        ResourceStatusCode::Paused,
        None,
        Some(0),
        false,
    ),
    (
        "REPORTED_ORNL_PAYLOAD",
        REPORTED_ORNL_PAYLOAD,
        ResourceStatusCode::Online,
        Some(true),
        None,
        true,
    ),
    (
        "OLD_SERVER_ORNL_UNHEALTHY",
        OLD_SERVER_ORNL_UNHEALTHY,
        ResourceStatusCode::Online,
        Some(false),
        None,
        false,
    ),
];

/// New-server payloads parse directly into the generated (spec) model,
/// including `health: null`.
#[test]
fn payload_new_server_matches_spec_model() {
    use iqm_server_api::models::IqmServerQuantumComputerDetails;

    let qc: IqmServerQuantumComputerDetails =
        serde_json::from_str(NEW_SERVER_ONLINE_HEALTHY).expect("should match the spec model");
    assert_eq!(qc.operational, "online");
    assert!(qc.health.as_ref().is_some_and(|h| h.healthy));
    assert_eq!(qc.queue_length, 3);

    let qc: IqmServerQuantumComputerDetails =
        serde_json::from_str(NEW_SERVER_ONLINE_UNHEALTHY).expect("should match the spec model");
    assert!(qc.health.as_ref().is_some_and(|h| !h.healthy));

    let qc: IqmServerQuantumComputerDetails =
        serde_json::from_str(NEW_SERVER_MAINTENANCE).expect("should match the spec model");
    assert_eq!(qc.operational, "maintenance");
    assert!(qc.health.is_none());
}

/// The issue's stub payloads (model-serialized shape, `operational` key)
/// parse into the spec model too. The maintenance one is exactly what
/// failed in 0.25.1 with `invalid type: null, expected struct
/// QcHealthDetail`.
#[test]
fn payload_reported_stub_matches_spec_model() {
    use iqm_server_api::models::IqmServerQuantumComputerDetails;

    let qc: IqmServerQuantumComputerDetails =
        serde_json::from_str(REPORTED_STUB_ONLINE_HEALTHY).expect("should parse");
    assert_eq!(qc.operational, "online");
    assert!(qc.health.as_ref().is_some_and(|h| h.healthy));

    let qc: IqmServerQuantumComputerDetails =
        serde_json::from_str(REPORTED_STUB_MAINTENANCE_HEALTH_NULL)
            .expect("null health should parse");
    assert_eq!(qc.operational, "maintenance");
    assert!(qc.health.is_none());

    let qc: IqmServerQuantumComputerDetails =
        serde_json::from_str(REPORTED_STUB_ONLINE_UNHEALTHY).expect("should parse");
    assert!(qc.health.as_ref().is_some_and(|h| !h.healthy));
}

/// Old-server payloads do NOT match the spec model -- this is the
/// difference `status()` falls back for. Pins down how they differ.
#[test]
fn payload_old_server_does_not_match_spec_model() {
    use iqm_server_api::models::IqmServerQuantumComputerDetails;

    for (name, raw) in [
        ("REPORTED_ORNL_PAYLOAD", REPORTED_ORNL_PAYLOAD),
        ("OLD_SERVER_ORNL_UNHEALTHY", OLD_SERVER_ORNL_UNHEALTHY),
    ] {
        let err = serde_json::from_str::<IqmServerQuantumComputerDetails>(raw)
            .expect_err("old-server payload should not match the spec model");
        assert!(err.to_string().contains("queue_length"), "{name}: {err}");

        let body = parse_payload(raw);
        assert!(body.get("health").is_none(), "{name}: health is nested");
        assert!(body.pointer("/status/health").is_some(), "{name}");
        assert!(body.get("operational").is_none(), "{name}");
        assert!(body.get("operational_status").is_none(), "{name}");
        assert!(body.get("queue_length").is_none(), "{name}");
    }
}

/// status() parsing of every new/old payload.
#[test]
fn payload_status_new_and_old_server() {
    for (name, raw, status, healthy, pending, _) in PAYLOAD_CASES {
        let st = IQMServer::resource_status_from_qc_details(parse_payload(raw))
            .unwrap_or_else(|e| panic!("{name}: status() parse failed: {e}"));
        assert_eq!(st.status, *status, "{name}: status");
        assert_eq!(st.healthy, *healthy, "{name}: healthy");
        assert_eq!(st.pending_job_count, *pending, "{name}: pending_job_count");
    }
}

/// status() and is_accessible() over HTTP for every new/old payload.
#[tokio::test]
#[allow(deprecated)]
async fn payload_status_and_is_accessible_over_http() {
    for (name, raw, status, healthy, _, accessible) in PAYLOAD_CASES {
        let (mut qrmi, _server) = one_shot_server("200 OK", parse_payload(raw)).await;
        let st = qrmi
            .status()
            .await
            .unwrap_or_else(|e| panic!("{name}: status() failed: {e}"));
        assert_eq!(st.status, *status, "{name}: status");
        assert_eq!(st.healthy, *healthy, "{name}: healthy");

        let (mut qrmi, _server) = one_shot_server("200 OK", parse_payload(raw)).await;
        let got = qrmi
            .is_accessible()
            .await
            .unwrap_or_else(|e| panic!("{name}: is_accessible() failed: {e}"));
        assert_eq!(got, *accessible, "{name}: is_accessible");
    }
}
