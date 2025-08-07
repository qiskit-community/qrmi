# Python examples for Sampler/Estimator primitives with IBM Direct Access QRMI

## Prerequisites

* Python 3.11 or 3.12
* [Installation of QRMI primitives python package(`qiskit-qrmi-primitives`)](../../../../README.md)

## Install dependencies

Assuming your python virtual environment is located at `~/py311venv_qrmi_primitives/bin/activate`,

```shell-session
$ source ~/py311venv_qrmi_primitives/bin/activate
$ pip install -r requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

### Common

When run as a job in a Slurm cluster, these environment variables are set by the SPANK plugin.

| Environment variables | Descriptions |
| ---- | ---- |
| SLURM_JOB_QPU_RESOURCES | Quantum resource names. Comma-separated values, e.g. `ibm_torino,ibm_brisbane` |
| SLURM_JOB_QPU_TYPES | Quantum resource types. Comma-separated values corresponding to each Quantum resource name specified by `SLURM_JOB_QPU_RESOURCES`.<br><br>Supported types:<ul><li>`direct-access`</li><li>`qiskit-runtime-service`</li></ul> |

### IBM Direct Access specific

When run as a job in a Slurm cluster, these environment variables are set by users or administrator.

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
| {resource_name}_QRMI_IBM_DA_TIMEOUT_SECONDS | Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time). System execution time is the amount of time that the system is dedicated to processing your job. |

#### Example
```shell-session
export SLURM_JOB_QPU_RESOURCES=test_eagle
export SLURM_JOB_QPU_TYPES=direct-access
export test_eagle_QRMI_IBM_DA_ENDPOINT=http://localhost:8080
export test_eagle_QRMI_IBM_DA_IAM_ENDPOINT=https://iam.cloud.ibm.com
export test_eagle_QRMI_IBM_DA_IAM_APIKEY=your_apikey
export test_eagle_QRMI_IBM_DA_SERVICE_CRN=your_instance
export test_eagle_QRMI_IBM_DA_AWS_ACCESS_KEY_ID=your_aws_access_key_id
export test_eagle_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY=your_aws_secret_access_key
export test_eagle_QRMI_IBM_DA_S3_ENDPOINT=https://s3.us-east.cloud-object-storage.appdomain.cloud
export test_eagle_QRMI_IBM_DA_S3_BUCKET=test
export test_eagle_QRMI_IBM_DA_S3_REGION=us-east
export test_eagle_QRMI_IBM_DA_TIMEOUT_SECONDS=86400
```

### IBM Qiskit Runtime Service specific

When run as a job in a Slurm cluster, these environment variables are set by users or administrator.

| Environment variables | Descriptions |
| ---- | ---- |
| {resource_name}_QRMI_IBM_QRS_ENDPOINT | Qiskit Runtime Service endpoint URL(e.g. `https://quantum.cloud.ibm.com/api`) |
| {resource_name}_QRMI_IBM_QRS_IAM_ENDPOINT | IBM Cloud IAM endpoint URL(e.g. `https://iam.cloud.ibm.com`) |
| {resource_name}_QRMI_IBM_QRS_IAM_APIKEY | IBM Cloud IAM API Key |
| {resource_name}_QRMI_IBM_QRS_SERVICE_CRN | Cloud Resource Name(CRN) of the provisioned Direct Access instance, starting with `crn:v1:`. |
| {resource_name}_QRMI_IBM_QRS_TIMEOUT_SECONDS | Time (in seconds) after which job should time out and get cancelled. It is based on system execution time (not wall clock time).
| {resource_name}_QRMI_IBM_QRS_SESSION_MODE | Session mode, default='dedicated', batch or dedicated. |
| {resource_name}_QRMI_IBM_QRS_SESSION_ID | Session ID, set by acquire function. Optional for acquire function, however, required other functions. |
System execution time is the amount of time that the system is dedicated to processing your job. |

#### Example
```shell-session
export SLURM_JOB_QPU_RESOURCES=ibm_torino,ibm_marrakesh
export SLURM_JOB_QPU_TYPES=qiskit-runtime-service,qiskit-runtime-service
export ibm_torino_QRMI_IBM_QRS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
export ibm_torino_QRMI_IBM_QRS_IAM_ENDPOINT=https://iam.cloud.ibm.com
export ibm_torino_QRMI_IBM_QRS_IAM_APIKEY=your_apikey
export ibm_torino_QRMI_IBM_QRS_SERVICE_CRN=your_instance
export ibm_marrakesh_QRMI_IBM_QRS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
export ibm_marrakesh_QRMI_IBM_QRS_IAM_ENDPOINT=https://iam.cloud.ibm.com
export ibm_marrakesh_QRMI_IBM_QRS_IAM_APIKEY=your_apikey
export ibm_marrakesh_QRMI_IBM_QRS_SERVICE_CRN=your_instance
```

## How to run

### SamplerV2

Code is based on "Get started with Sampler" tutorial (https://docs.quantum.ibm.com/guides/get-started-with-primitives#get-started-with-sampler).

```shell-session
$ python sampler.py
```

### EstimatorV2

Code is based on "Get started with Estimator" tutorial (https://docs.quantum.ibm.com/guides/get-started-with-primitives#get-started-with-estimator).

```shell-session
$ python estimator.py
```

### SQD tutorial

[01_chemistry_hamiltonian.ipynb](./01_chemistry_hamiltonian.ipynb) is QRMI primitive port of [Improving energy estimation of a chemistry Hamiltonian with SQD](https://github.com/Qiskit/qiskit-addon-sqd/blob/main/docs/tutorials/01_chemistry_hamiltonian.ipynb). Start jupyter notebook and run all cells from beginning.
