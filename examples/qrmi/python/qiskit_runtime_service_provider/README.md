# QRMI Provider for IBM Qiskit Runtime Service - Examples in Python

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install qrmi[ibm]
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_RESOURCE_PROVIDER_CONFIG_FILE | qrmi config v2 file |

## How to run

```shell-session
$ python example.py
```
For example,
```shell-session
export QRMI_RESOURCE_PROVIDER_CONFIG_FILE=/shared/qrmi_config_v2.json

python example.py
```
