# Tools to generate EstimatorV2/SamplerV2/Executor primitive input

The tools demonstrate the generation of EstimatorV2/SamplerV2/Executor inputs from a quantum circuit example.

## Prerequisites

* Python 3.11 or above

## Install dependencies

```shell-session
pip install -f requirements.txt
```

## Tools

### gen_estimator_input.py

Generates EstimatorV2 input for the circuit introduced in [Getting started doc](https://quantum.cloud.ibm.com/docs/en/guides/get-started-with-estimator).

Usage:

```shell-session
usage: gen_estimator_inputs.py [-h] [--iam_url IAM_URL] backend base_url apikey instance

A tool to generate EstimatorV2 input for testing

positional arguments:
  backend            Backend name
  base_url           API endpoint
  apikey             IAM API key
  instance           Service CRN of your instance - starting with 'crn:v1:'

options:
  -h, --help         show this help message and exit
  --iam_url IAM_URL  IAM endpoint
```

Example:

```bash
python gen_estimator_inputs.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance, starting with 'crn:v1:'>
```

Output:

| Files | Descriptions |
| ---- | ---- |
| estimator_input_{backend_name}_params_only.json | EstimatorV2 input parameters([EstimatorV2 schema](https://quantum.cloud.ibm.com/docs/en/api/qiskit-runtime-rest/tags/jobs)).
| estimator_input_{backend_name}.json | An input for QRMI task runner, which contains 2 properties - `program_id`(=`estimator`) and `parameters`(EstimatorV2 input parameters). |

### gen_sampler_input.py

Generates SamplerV2 input for the circuit introduced in [Getting started doc](https://quantum.cloud.ibm.com/docs/en/guides/get-started-with-sampler).

Usage:

```shell-session
usage: gen_sampler_inputs.py [-h] [--iam_url IAM_URL] backend base_url apikey instance

A tool to generate SamplerV2 input for testing

positional arguments:
  backend            Backend name
  base_url           API endpoint
  apikey             IAM API key
  instance           Service CRN of your instance - starting with 'crn:v1:'

options:
  -h, --help         show this help message and exit
  --iam_url IAM_URL  IAM endpoint
```

Example:

```bash
python gen_sampler_inputs.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance, starting with 'crn:v1:'>
```

Output:

| Files | Descriptions |
| ---- | ---- |
| sampler_input_{backend_name}_params_only.json | SamplerV2 input parameters([SamplerV2 schema](https://quantum.cloud.ibm.com/docs/en/api/qiskit-runtime-rest/tags/jobs)).
| sampler_input_{backend_name}.json | An input for QRMI task runner, which contains 2 properties - `program_id`(=`sampler`) and `parameters`(SamplerV2 input parameters). |

### gen_executor_inputs.py

Generates Executor v2.0 input for the circuit introduced in [Getting started doc](https://quantum.cloud.ibm.com/docs/en/guides/executor-examples).

Usage:

```shell-session
usage: gen_executor_inputs.py [-h] [--iam_url IAM_URL] [--schema_version SCHEMA_VERSION] backend base_url apikey instance

A tool to generate Executor input for testing

positional arguments:
  backend               Backend name
  base_url              API endpoint
  apikey                IAM API key
  instance              Service CRN of your instance - starting with 'crn:v1:'

options:
  -h, --help            show this help message and exit
  --iam_url IAM_URL     IAM endpoint
  --schema_version SCHEMA_VERSION
                        Executor schema version. default: v2.0
```

Example:

```bash
python gen_executor_inputs.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance, starting with 'crn:v1:'>
```

Output:

| Files | Descriptions |
| ---- | ---- |
| executor_input_{backend_name}_{schema_version}_params_only.json | Executor input parameters([Executor v2.0 schema](https://quantum.cloud.ibm.com/docs/en/api/qiskit-runtime-rest/tags/jobs)).
| executor_input_{backend_name}_{schema_version}.json | An input for QRMI task runner, which contains 2 properties - `program_id`(=`executor`) and `parameters`(Executor input parameters). |

