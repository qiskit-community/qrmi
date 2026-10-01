# OQTOPUS QRMI - Examples in Lua

## Prerequisites

* [QRMI C library(libqrmi.so)](../../../../README.md#standalone-c-library)
* [QRMI Lua Module(qrmi.so)](../../../../lua/README.md)

## Setup

```bash
export LUA_CPATH="</path/to/qrmi.so-dir/>?.so;;"
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libqrmi.so-dir
```

Example:
```bash
export LUA_CPATH="/shared/qrmi/lua/build/?.so;;"
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/shared/qrmi/target/release
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below.

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

## How to run this example
```shell-session
lua example.lua <device_id> <job_spec file> <job_type('sampling','estimation', 'multi_manual' or 'sse')>
```
For example,
```shell-session
export qulacs_QRMI_OQTOPUS_URL=https://demo-api.oqtopus.io
export qulacs_QRMI_OQTOPUS_API_TOKEN=your api token
export PY_BRIDGE_PATH=<path/to/liboqtopus_py_bridge.so>

lua example.lua qulacs ../../../task_runner/oqtopus/sampling_input_qulacs.json sampling
```
