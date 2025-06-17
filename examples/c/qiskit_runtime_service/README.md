# Qiskit Runtime Service QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {resource_name}_QRMI_IBM_QRS_ENDPOINT | Qiskit Runtime Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| {resource_name}_QRMI_IBM_QRS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| {resource_name}_QRMI_IBM_QRS_IAM_APIKEY | IBM Cloud IAM API Key |
| {resource_name}_QRMI_IBM_QRS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Qiskit Runtime Service instance, starting with `crn:v1:`. |
| {resource_name}_QRMI_IBM_QRS_SESSION_MODE | Execution mode to run the session in, `default='dedicated'`, `batch` or `dedicated`. |
| {resource_name}_QRMI_IBM_QRS_SESSION_MAX_TTL | The maximum time (in seconds) for the session to run, subject to plan limits, default: `28800`. |
| {resource_name}_QRMI_IBM_QRS_TIMEOUT_SECONDS | (Optional) Cost of the job as the estimated time it should take to complete (in seconds). Should not exceed the cost of the program, default: `None`. |
| {resource_name}_QRMI_IBM_QRS_SESSION_ID | (Optional) Session ID, can be obtanied by acquire function. If exists, used in the target functions. |

## Create Qiskit Primitive input file as input

Refer [this tool](../../../../commands/task_runner/examples/qiskit) to generate. You can customize quantum circuits by editting the code.

> [!NOTE]
> Use the file with name ending with `_params_only.json`, e.g. `sampler_input_ibm_torino_params_only.json`.

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/qiskit_runtime_service
qiskit_runtime_service <backend_name> <primitive input file> <program id>
```
For example,
```shell-session
export ibm_torino_QRMI_IBM_QRS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
export ibm_torino_QRMI_IBM_QRS_IAM_ENDPOINT=https://iam.cloud.ibm.com
export ibm_torino_QRMI_IBM_QRS_IAM_APIKEY=your_apikey
export ibm_torino_QRMI_IBM_QRS_SERVICE_CRN=your_instance

./build/qiskit_runtime_service ibm_torino sampler_input.json sampler
```
