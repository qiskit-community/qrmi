# OQTOPUS QRMI - Examples in Python

## Prerequisites

* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
$ source ~/py312_oqtopus_venv/bin/activate
$ pip install oqtopus_client
```

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

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] device_id job_spec job_type

An example of OQTOPUS QRMI

positional arguments:
  device_id   OQTOPUS device ID
  job_spec    Job spec
  job_type    Job type

options:
  -h, --help  show this help message and exit
```
For example,
```shell-session
# .env
qulacs_QRMI_OQTOPUS_URL=https://demo-api.oqtopus.io
qulacs_QRMI_OQTOPUS_API_TOKEN=your api token
PY_BRIDGE_PATH=<path/to/liboqtopus_py_bridge.so>

python example.py qulacs ../../../task_runner/oqtopus/sampling_input_qulacs.json sampling
```
