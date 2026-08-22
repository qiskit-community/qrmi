# QRMIService - Example in Rust

Unlike the other examples under [`examples/qrmi/rust`](../), which each construct a single, specific `QuantumResource` implementation directly (e.g.
`IBMQuantumSystem::new("ibm_torino")`), this example uses `QRMIService`, which discovers *all* of the QPU resources assigned to the current job from
the environment -- the same environment variables a Slurm QRMI plugin would set -- and exposes the ones that are currently accessible.

This is the Rust counterpart to the `qrmi.QRMIService` Python class (see [`python/qrmi/primitives`](../../../../python/qrmi/primitives)); both share
the same underlying implementation (`qrmi::QRMIService` in the core Rust library), just with a Rust vs. Python API on top.

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../../README.md)

## Set environment variables

`QRMIService` reads `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES` (falling back to the legacy `SLURM_JOB_QPU_RESOURCES` / `SLURM_JOB_QPU_TYPES`), each a
delimiter-separated list (delimiter: `,` by default, overridable via `QRMI_LIST_DELIMITER`). The two lists must be the same length and pair up positionally:

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_JOB_QPU_RESOURCES | Comma-separated list of resource identifiers, e.g. `ibm_torino,my_pasqal_qpu` |
| QRMI_JOB_QPU_TYPES | Comma-separated list of resource types, positionally paired with the above, e.g. `qiskit-runtime-service,pasqal-cloud` |
| QRMI_LIST_DELIMITER | Delimiter used for both lists above. Optional, defaults to `,` |

Supported values for `QRMI_JOB_QPU_TYPES` entries: `ibm-quantum-system`, `ibm-quantum-compute-service`, `qiskit-runtime-service`, `pasqal-cloud`, `pasqal-local`, `alice-bob-felis`, `iqm-server`.

Each resource named in `QRMI_JOB_QPU_RESOURCES` also needs its own vendor-specific environment variables set -- see the other examples in this
directory (e.g. [`../ibm_quantum_compute_service`](../ibm_quantum_compute_service) or [`../pasqal_cloud`](../pasqal_cloud)) for what those are per vendor. This
example assumes a `.env` file with all of the above is available in the current directory.

## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example

```shell-session
$ ../target/release/qrmi-example-service --help
QRMIService - Example

Usage: qrmi-example-service [OPTIONS]

Options:
  -r, --resource <RESOURCE>  Resource identifier to acquire, e.g. a backend name. If omitted, this just lists every accessible resource assigned to the job
  -h, --help                 Print help
  -V, --version              Print version
```

For example, with a `.env` file set up as described above:

```shell-session
# List every accessible resource assigned to this job.
../target/release/qrmi-example-service
```
```text
Accessible resources (2 found):
----------------------------------------
  ibm_torino                     type=qiskit-runtime-service
  my_pasqal_qpu                  type=pasqal-cloud
```

```shell-session
# Acquire one specific resource by id, print its metadata, then release it.
../target/release/qrmi-example-service --resource ibm_torino
```
