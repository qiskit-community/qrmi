# Pasqal CLoud QRMI - Examples in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| <backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID |  Pasqal Cloud Project ID to access the QPU |
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token |

## Create Pulser Sequence file as input

Given a Pulser sequence `sequence`, we can convert it to a JSON string and write it to a file like this:

```python
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)
```

## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/pasqal_cloud --help
QRMI for Pasqal Cloud - Example

Usage: qrmi-example-pasqal-cloud --backend <BACKEND> --input <INPUT>

Options:
  -b, --backend <BACKEND>        backend name
  -i, --input <INPUT>            sequence input file
  -h, --help                     Print help
  -V, --version                  Print version
```
For example,
```shell-session
$ ../target/release/qrmi-example-pasqal-cloud -b FRESNEL -i input.json
```
