# Pasqal Local QRMI - Examples in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../../README.md)
* [Munge](https://dun.github.io/munge/)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| `<backend_name>_QRMI_URL` | URL of the QPU middleware (e.g. `http://localhost:4207`) |
| `QRMI_JOB_UID` | ID of the user executing the job |
| `QRMI_JOB_ID` | ID of the job |

Where `<backend_name>` is the backend name passed via `--backend` (e.g. `PASQAL_LOCAL`).


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
$ cargo build --release --features=qrmi/munge
```

## How to run this example
```shell-session
$ ../target/release/qrmi-example-pasqal-local --help
QRMI for Pasqal Local - Example

Usage: qrmi-example-pasqal-local --backend <BACKEND> --input <INPUT>

Options:
  -b, --backend <BACKEND>        backend name (device identifier)
  -i, --input <INPUT>            sequence input file
  -h, --help                     Print help
  -V, --version                  Print version
```
For example,
```shell-session
$ ../target/release/qrmi-example-pasqal-local -b PASQAL_LOCAL -i input.json
```
