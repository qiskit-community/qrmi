# IQM Server `GET /api/v1/quantum-computers/{qc}` response fixtures

Loaded with `include_str!` by the `payload_*` tests in `../iqm_server.rs`.
Each file is one raw response body.

| File | Source | Shape |
|---|---|---|
| `new_server_online_healthy.json` | IQM Server API spec (v1) | top-level `operational_status`, `health`, `queue_length` |
| `new_server_online_unhealthy.json` | same | `health.healthy: false` |
| `new_server_maintenance.json` | same | `operational_status: "maintenance"`, `health: null` |
| `reported_ornl_payload.json` | **Reported in the issue** (ORNL on-prem, 20 qubits) | no `operational`/`operational_status`, health under `status.health`, no `queue_length` |
| `old_server_ornl_unhealthy.json` | `reported_ornl_payload.json` with `healthy: false` | same as above |
| `reported_stub_online_healthy.json` | **Issue's reproduction stub** (`iqm_status_stub.py`) | model-serialized shape: top-level `operational`, `health`, `queue_length` |
| `reported_stub_maintenance_health_null.json` | same | `operational: "maintenance"`, `health: null` |
| `reported_stub_online_unhealthy.json` | same | `health.healthy: false` |

The issue elided the ORNL payload's values (`<uuid>`, `<str>`, `<int>`,
`<bool>`); they are placeholders here. The key layout and `updated_at` are as
reported.

## Checking a real server

Capture a response and overwrite the fixture of the matching case:

```sh
curl -s -H "Authorization: Bearer $TOKEN" \
  "$QRMI_IQM_ISA_ENDPOINT/api/v1/quantum-computers/<alias>" \
  > src/iqm/tests/fixtures/reported_ornl_payload.json
cargo test --lib iqm::server::tests::payload
```

The expected `status` / `healthy` / `pending_job_count` / `is_accessible`
per fixture are in `PAYLOAD_CASES` in `../iqm_server.rs`; adjust them if the
captured device is in a different state (e.g. unhealthy or under
maintenance).
