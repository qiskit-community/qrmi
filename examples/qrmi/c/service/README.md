# QRMI Service Example in C

Unlike the other examples in [`examples/qrmi/c`](../), which each construct a single, specific resource directly (e.g. `qrmi_resource_new("ibm_torino", QRMI_RESOURCE_TYPE_IBM_QUANTUM_SYSTEM)`), this example uses `qrmi_service_resources()`, which discovers *all* of the QPU resources assigned to the current job from the environment -- the same environment variables a Slurm QRMI plugin would set -- and returns the ones that are currently accessible.

This is the C counterpart of `qrmi.QRMIService` (Python) and `qrmi::QRMIService` (Rust); all three share the same underlying discovery logic, just with a different API on top. Unlike the Python/Rust versions, `qrmi_service_resources()` is a one-shot call: there is no persistent "service" handle to create or free -- each call performs its own discovery and returns a freshly-owned array of `QrmiQuantumResource` handles, in the same style as `qrmi_provider_resources()`. See that function's docs (and [`../resource_providers`](../resource_providers)) if you're not familiar with it.

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../../README.md)

## Set environment variables

`qrmi_service_resources()` reads `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES` (falling back to the legacy `SLURM_JOB_QPU_RESOURCES` / `SLURM_JOB_QPU_TYPES`), each a delimiter-separated list (delimiter: `,` by default, overridable via `QRMI_LIST_DELIMITER`). The two lists must be the same length and pair up positionally:

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_JOB_QPU_RESOURCES | Comma-separated list of resource identifiers, e.g. `ibm_torino,my_pasqal_qpu` |
| QRMI_JOB_QPU_TYPES | Comma-separated list of resource types, positionally paired with the above, e.g. `qiskit-runtime-service,pasqal-cloud` |
| QRMI_LIST_DELIMITER | Delimiter used for both lists above. Optional, defaults to `,` |

Supported values for `QRMI_JOB_QPU_TYPES` entries: `ibm-quantum-system`, `ibm-quantum-compute-service`, `qiskit-runtime-service`, `pasqal-cloud`, `pasqal-local`, `alice-bob-felis`, `iqm-server`.

Each resource named in `QRMI_JOB_QPU_RESOURCES` also needs its own vendor-specific environment variables set -- see the other examples in this directory (e.g. [`../ibm_quantum_compute_service`](../ibm_quantum_compute_service) or [`../pasqal_cloud`](../pasqal_cloud)) for what those are per vendor. This example assumes a `.env` file with all of the above is available in the current directory.

## How to build

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run

```shell-session
# List every accessible resource assigned to this job.
./build/service

# Acquire one specific resource by id, print its metadata and target, then release it.
./build/service ibm_torino
```
