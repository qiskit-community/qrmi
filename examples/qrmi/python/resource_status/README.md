# Display the current status of the specified resource

This is an example of QRMI status() API.

## Prerequisites

* Python 3.11, 3.12 or 3.13
* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Resource ID | Environment variables | Descriptions |
| ---- | ---- | ---- |
| iqm-server | {resource_id}_QRMI_IQM_ISA_ENDPOINT | IQM Server API endpoint URL |
| | {resource_id}_QRMI_IQM_ISA_TOKEN | IQM Server API token |
| ibm-quantum-system | {resource_id}_QRMI_IBM_QS_ENDPOINT | Quantum System endpoint URL |
| | {resource_id}_QRMI_IBM_QS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| | {resource_id}_QRMI_IBM_QS_IAM_APIKEY | IBM Cloud IAM API Key |
| | {resource_id}_QRMI_IBM_QS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Quantum System instance, starting with `crn:v1:`. |
| ibm-quantum-compute-service | {resource_id}_QRMI_IBM_QCS_ENDPOINT | Quantum Compute Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| | {resource_id}_QRMI_IBM_QCS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| | {resource_id}_QRMI_IBM_QCS_IAM_APIKEY | IBM Cloud IAM API Key |
| | {resource_id}_QRMI_IBM_QCS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Quantum Compute Service instance, starting with `crn:v1:`. |
| pasqal-cloud | {resource_id}_QRMI_PASQAL_CLOUD_PROJECT_ID |  Pasqal Cloud Project ID to access the QPU |
| | {resource_id}_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token (optional when username/password are configured) |
| pasqal-local | {resource_id}_QRMI_WARDEN_URL | URL of the QPU middleware (e.g. `http://localhost:4207`). Falls back to the deprecated `<backend_name>_QRMI_URL` if not set. |
| alice-bob-felis | {resource_id}_QRMI_AB_FELIS_API_KEY | API key to access Alice & Bob Felis |
| | {resource_id}_QRMI_AB_FELIS_BASE_ENDPOINT | API endpoint URL |

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] resource_type resource_id

Display the current status of the specified resource

positional arguments:
  resource_type  Resource type
  resource_id    Resource ID

options:
  -h, --help     show this help message and exit
```
For example,
```shell-session
export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

python example.py iqm-server garnet_mock
```
