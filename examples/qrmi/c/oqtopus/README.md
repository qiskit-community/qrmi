# OQTOPUS QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

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

Refer [this tool](../../../task_runner/oqtopus) to generate.

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/oqtopus
oqtopus <Device ID> <Job spec JSON file> <Job type('sampling','estimation', 'multi_manual' or 'sse')>
```
For example,
```shell-session
# .env
qulacs_QRMI_OQTOPUS_BASE_URL=https://demo-api.oqtopus.io
qulacs_QRMI_OQTOPUS_API_TOKEN=your api token
PY_BRIDGE_PATH=/shared/sandbox/OQTOPUS/qrmi/target/release/liboqtopus_py_bridge.so

./oqtopus qulacs ../../../../task_runner/oqtopus/sampling_input_qulacs.json sampling
```
