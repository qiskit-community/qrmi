# OQTOPUS QRMI - Examples in Rust

## Prerequisites

* [QRMI Rust library](../../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {device_id}_QRMI_OQTOPUS_URL | OQTOPUS Cloud API endpoint |
| {device_id}_QRMI_OQTOPUS_API_TOKEN | OQTOPUS Cloud API token |
| {device_id}_QRMI_OQTOPUS_PROXY | Proxy. Optional. |
| {device_id}_QRMI_OQTOPUS_TIMEOUT | HTTP request timeout seconds. (e.g. 30.0). Optional. |
| {device_id}_QRMI_OQTOPUS_RETRY_BACKOFF_SECONDS | Exponential backoff base seconds.(e.g. 0.2). Optional |
| {device_id}_QRMI_OQTOPUS_RETRY_STATUS_CODES | HTTP status codes treated as retryable. (e.g. 429,500,502,503). Optional. |
| {device_id}_QRMI_OQTOPUS_RETRY_METHODS | HTTP methods treated as retryable.(e.g. GET,POST). Optional. |

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/oqtopus) to generate. You can customize quantum circuits by editing the code.

## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/qrmi-example-oqtopus -h
QRMI for OQTOPUS - Example

Usage: qrmi-example-oqtopus --device-id <DEVICE_ID> --job-spec <JOB_SPEC>

Options:
  -d, --device-id <DEVICE_ID>  device ID
  -j, --job-spec <JOB_SPEC>    Job spec JSON file
  -h, --help                   Print help
  -V, --version                Print version
```

For example,
```shell-session
# .env
qulacs_QRMI_OQTOPUS_BASE_URL=https://demo-api.oqtopus.io
qulacs_QRMI_OQTOPUS_API_TOKEN=your api token
PY_BRIDGE_PATH=<path/to/liboqtopus_py_bridge.so>

../target/release/qrmi-example-oqtopus --device-id qulacs --job-spec ../../../task_runner/oqtopus/sampling_input_qulacs.json 
```
