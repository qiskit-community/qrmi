# IQM Qiskit example

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
| SLURM_JOB_QPU_RESOURCES | Quantum resource names. Comma-separated values, e.g. `garnet,emerald` |
| SLURM_JOB_QPU_TYPES | Quantum resource types. Comma-separated values corresponding to each Quantum resource name specified by `SLURM_JOB_QPU_RESOURCES`.<br><br>Supported types:<ul><li>`ibm-quantum-system`</li><li>`qiskit-runtime-service`</li><li>`iqm-server`</li></ul> |

### IQM Server specific

When run as a job in a Slurm cluster, these environment variables are set by users or administrator.

| Environment variables | Descriptions |
| ---- | ---- |
| {qc_alias_name}_QRMI_IQM_ISA_ENDPOINT | IQM Server API endpoint |
| {qc_alias_name}_QRMI_IQM_ISA_TOKEN | IQM Server API token |

> [!NOTE]
> Replace the “:” in the QC alias name with “_” when specifying it. For example, `sirius:mock` -> `sirius_mock`.

#### Example
```shell-session
export SLURM_JOB_QPU_RESOURCES=garnet_mock
export SLURM_JOB_QPU_TYPES=iqm-server
export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
export garnet_mock_QRMI_IQM_ISA_TOKEN=your token
```

## How to run

```shell-session
$ python iqm_example.py
```
