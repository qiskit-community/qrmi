# IBM Quantum System QRMI Provider - Examples in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_RESOURCE_PROVIDER_CONFIG_FILE | qrmi config v2 file |


## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/qrmi-example-qiskit-runtime-service --help
QRMI Provider for IBM Quantum System - Example

Usage: qrmi-example-ibm-quantum-system-provider [OPTIONS]

Options:
  -f, --filters <FILTERS>  A filter specification using comma-separated key-value pairs
  -h, --help               Print help
  -V, --version            Print version
```
For example,
```shell-session
export QRMI_RESOURCE_PROVIDER_CONFIG_FILE=/shared/qrmi_config_v2.json

../target/release/qrmi-example-ibm-quantum-system-provider  -f "num_qubits=127&name=ibm_*&max_shots=10000"
```
