# Python examples for Python Qiskit Primitives with Pasqal Cloud QRMI

## Prerequisites

* Python 3.11 or 3.12
* [Installation of QRMI primitives python package(`qiskit-qrmi-primitives`)](../../README.md)

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
| SLURM_JOB_QPU_RESOURCES | Quantum resource names. Comma-separated values, e.g. `FRESNEL` |
| SLURM_JOB_QPU_TYPES | Quantum resource types. Comma-separated values corresponding to each Quantum resource name specified by `SLURM_JOB_QPU_RESOURCES`.<br><br>Supported types:<ul><li>`pasqal-cloud`</li></ul> |



## How to run

### SamplerV2

Use the Qiskit Pasqal Provider `SamplerV2`.

```shell-session
$ python sampler.py
```

### Target -> Pulser device

Example to show how to get the Pulser device via the QRMI.

```shell-session
$ python target.py
```