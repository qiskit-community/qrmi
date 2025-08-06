# Direct Access QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {resource_name}_QRMI_IBM_DA_ENDPOINT | Direct Access endpoint URL |
| {resource_name}_QRMI_IBM_DA_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| {resource_name}_QRMI_IBM_DA_IAM_APIKEY | IBM Cloud IAM API Key |
| {resource_name}_QRMI_IBM_DA_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Direct Access instance, starting with `crn:v1:`. |
| {resource_name}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID | AWS Access Key ID to access S3 bucket |
| {resource_name}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY | AWS Secret Access Key to access S3 bucket |
| {resource_name}_QRMI_IBM_DA_S3_ENDPOINT | S3 endpoint URL |
| {resource_name}_QRMI_IBM_DA_S3_BUCKET | S3 bucket name |
| {resource_name}_QRMI_IBM_DA_S3_REGION | S3 bucket region name(e.g. `us-east`) |
| {resource_name}_QRMI_JOB_TIMEOUT_SECONDS | Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time). System execution time is the amount of time that the system is dedicated to processing your job. |


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
$ ./build/direct_access
direct_access <backend_name> <primitive input file> <program id>
```
For example,
```shell-session
export test_eagle_QRMI_IBM_DA_ENDPOINT=http://localhost:8080
export test_eagle_QRMI_IBM_DA_IAM_ENDPOINT=https://iam.cloud.ibm.com
export test_eagle_QRMI_IBM_DA_IAM_APIKEY=your_apikey
export test_eagle_QRMI_IBM_DA_SERVICE_CRN=your_instance
export test_eagle_QRMI_IBM_DA_AWS_ACCESS_KEY_ID=your_aws_access_key_id
export test_eagle_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY=your_aws_secret_access_key
export test_eagle_QRMI_IBM_DA_S3_ENDPOINT=https://s3.us-east.cloud-object-storage.appdomain.cloud
export test_eagle_QRMI_IBM_DA_S3_BUCKET=test
export test_eagle_QRMI_IBM_DA_S3_REGION=us-east
export test_eagle_QRMI_JOB_TIMEOUT_SECONDS=86400

./build/direct_access test_eagle sampler_input.json sampler
```
