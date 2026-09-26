# OQTOPUS QRMI - Examples in Python

## Prerequisites

* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {device_id}_QRMI_OQTOPUS_BASE_URL | OQTOPUS Cloud API endpoint URL |
| {device_id}_QRMI_OQTOPUS_API_TOKEN | OQTOPUS Cloud API token |

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/oqtopus) to generate.

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] device_id

An example of IBM Quantum System QRMI

positional arguments:
  device_id   OQTOPUS device ID
  program     QASM program file
  job_type    job type

options:
  -h, --help  show this help message and exit
```
For example,
```shell-session
export qulacs_QRMI_OQTOPUS_BASE_URL=https://demo-api.oqtopus.io
export qulacs_QRMI_OQTOPUS_API_TOKEN=your api token

python example.py qulacs ../../../task_runner/oqtopus/bell_state.txt sampling 
```
