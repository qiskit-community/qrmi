# Qiskit Runtime Service QRMI - Examples in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_RESOURCE_ID | Quantum backend name(e.g. `ibm_torino`) |
| QRMI_IBM_QRS_ENDPOINT | Qiskit Runtime Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| QRMI_IBM_QRS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| QRMI_IBM_QRS_IAM_APIKEY | IBM Cloud IAM API Key |
| QRMI_IBM_QRS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Qiskit Runtime Service instance, starting with `crn:v1:`. |
| QRMI_IBM_QRS_SESSION_MODE | Execution mode to run the session in, `default='dedicated'`, `batch` or `dedicated`. |
| QRMI_IBM_QRS_SESSION_MAX_TTL | The maximum time (in seconds) for the session to run, subject to plan limits, default: `28800`. |
| QRMI_IBM_QRS_TIMEOUT_SECONDS | (Optional) Cost of the job as the estimated time it should take to complete (in seconds). Should not exceed the cost of the program, default: `None`. |
| QRMI_IBM_QRS_SESSION_ID | (Optional) Session ID, can be obtanied by acquire function. If exists, used in the target functions. |

## Create Qiskit Primitive input file as input

Refer [this tool](../../../../commands/qrun/qiskit_pubs_gen) to generate. You can customize quantum circuits by editting the code.

## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/qiskit_runtime_service --help
QRMI for Qiskit Runtime Service - Example

Usage: qrmi-example-qiskit_runtime_service --input <INPUT> --program-id <PROGRAM_ID>

Options:
  -i, --input <INPUT>            primitive input file
  -p, --program-id <PROGRAM_ID>  program id
  -h, --help                     Print help
  -V, --version                  Print version
```
For example,
```shell-session
$ ../target/release/qrmi-example-qiskit_runtime_service -i sampler_input.json -p sampler
```
