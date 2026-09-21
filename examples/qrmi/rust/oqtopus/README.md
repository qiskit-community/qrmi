# OQTOPUS QRMI - Examples in Rust

## Prerequisites

* [QRMI Rust library](../../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {device_id}_QRMI_OQTOPUS_BASE_URL | OQTOPUS Cloud API endpoint URL |
| {device_id}_QRMI_OQTOPUS_API_TOKEN | OQTOPUS Cloud API token |

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/oqtopus) to generate. You can customize quantum circuits by editing the code.

> [!NOTE]
> Use the file with name ending `_params_only.json`, e.g. `oqtopus_params_only.json`.


## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/qrmi-example-oqtopus -h
QRMI for OQTOPUS - Example

Usage: qrmi-example-oqtopus [OPTIONS] --device-id <DEVICE_ID>

Options:
  -d, --device-id <DEVICE_ID>        Device ID
  -h, --help                         Print help
  -V, --version                      Print version
```

For example,
```shell-session
export qulacs_QRMI_OQTOPUS_BASE_URL=https://demo-api.oqtopus.io
export qulacs_QRMI_OQTOPUS_API_TOKEN=your api token

../target/release/qrmi-example-oqtopus -d qulacs
```
