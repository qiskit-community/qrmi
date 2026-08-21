# QRMIService - Example in Python

Unlike the other examples under [`examples/qrmi/python`](../), which each
construct a single, specific `QuantumResource` directly (e.g.
`QuantumResource("ibm_torino", ResourceType.IBMQuantumSystem)`), this example
uses `QRMIService`, which discovers *all* of the QPU resources assigned to
the current job from the environment -- the same environment variables a
Slurm QRMI plugin would set -- and exposes the ones that are currently
accessible.

This is the same `QRMIService` used by `qrmi.primitives` (see
[`../../qiskit_primitives`](../../qiskit_primitives)); this example just
uses it directly, without a Qiskit primitive on top.

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

`QRMIService` reads `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES` (falling
back to the legacy `SLURM_JOB_QPU_RESOURCES` / `SLURM_JOB_QPU_TYPES`), each a
delimiter-separated list (delimiter: `,` by default, overridable via
`QRMI_LIST_DELIMITER`). The two lists must be the same length and pair up
positionally:

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_JOB_QPU_RESOURCES | Comma-separated list of resource identifiers, e.g. `ibm_torino,my_pasqal_qpu` |
| QRMI_JOB_QPU_TYPES | Comma-separated list of resource types, positionally paired with the above, e.g. `qiskit-runtime-service,pasqal-cloud` |
| QRMI_LIST_DELIMITER | Delimiter used for both lists above. Optional, defaults to `,` |
| QRMI_PLUGIN_ERROR | If set, `QRMIService()` fails immediately, raising this as a `RuntimeError` (mirrors how a Slurm QRMI plugin reports a resource acquisition failure) |

Supported values for `QRMI_JOB_QPU_TYPES` entries: `ibm-quantum-system`,
`qiskit-runtime-service`, `pasqal-cloud`, `pasqal-local`, `alice-bob-felis`,
`iqm-server`.

Each resource named in `QRMI_JOB_QPU_RESOURCES` also needs its own
vendor-specific environment variables set -- see the other examples in this
directory (e.g. [`../ibm_quantum_compute_service`](../ibm_quantum_compute_service) or
[`../pasqal_cloud`](../pasqal_cloud)) for what those are per vendor. This
example assumes a `.env` file with all of the above is available in the
current directory.

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] [resource]

QRMIService Example

positional arguments:
  resource    Resource identifier to acquire, e.g. a backend name. If
              omitted, this just lists every accessible resource assigned
              to the job.

options:
  -h, --help  show this help message and exit
```

For example, with the environment variables above set:

```shell-session
# List every accessible resource assigned to this job.
python example.py
```
```text
Accessible resources (2 found):
  ibm_torino                     type=ResourceType.IBMQiskitRuntimeService
  my_pasqal_qpu                  type=ResourceType.PasqalCloud
```

```shell-session
# Acquire one specific resource by id, print its metadata and target, then release it.
python example.py ibm_torino
```
