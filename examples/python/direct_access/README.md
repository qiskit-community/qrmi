# Direct Access QRMI - Examples in Python

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_IBM_DA_ENDPOINT | Direct Access endpoint URL |
| QRMI_IBM_DA_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| QRMI_IBM_DA_IAM_APIKEY | IBM Cloud IAM API Key |
| QRMI_IBM_DA_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Direct Access instance, starting with `crn:v1:`. |
| QRMI_IBM_DA_AWS_ACCESS_KEY_ID | AWS Access Key ID to access S3 bucket |
| QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY | AWS Secret Access Key to access S3 bucket |
| QRMI_IBM_DA_S3_ENDPOINT | S3 endpoint URL |
| QRMI_IBM_DA_S3_BUCKET | S3 bucket name |
| QRMI_IBM_DA_S3_REGION | S3 bucket region name(e.g. `us-east`) |
| QRMI_IBM_DA_TIMEOUT_SECONDS | Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time). System execution time is the amount of time that the system is dedicated to processing your job. |

## Create Qiskit Primitive input file as input

Refer [this tool](../../../../commands/qrun/qiskit_pubs_gen) to generate. You can customize quantum circuits by editting the code.

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] backend input program_id

An example of IBM Direct Access QRMI

positional arguments:
  backend     backend name
  input       primitive input file
  program_id  'estimator' or 'sampler'

options:
  -h, --help  show this help message and exit
```
For example,
```shell-session
$ python example.py your_backend sampler_input.json sampler
```
