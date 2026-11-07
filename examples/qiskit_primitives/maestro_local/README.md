# Python examples for Qiskit Primitives with Maestro Local QRMI

## Prerequisites

* Rust 1.91.1 or above
* Python 3.11 or 3.12
* [Installation of QRMI python package](../../../README.md)
* A running [Maestro Local server](https://github.com/QoroQuantum/maestro-local-server) ([installation guide](https://github.com/QoroQuantum/maestro-local-server/blob/main/docs/installation.md)), reachable at the Unix domain socket `/run/maestro.sock`

## Install dependencies

Assuming your python virtual environment is located at `~/py311venv_qrmi_primitives/bin/activate`,

```shell-session
$ source ~/py311venv_qrmi_primitives/bin/activate
$ pip install -r requirements.txt
```

## Environment variables

The examples construct `QuantumResource("MAESTRO_LOCAL", ResourceType.MaestroLocal)`
directly. Slurm jobs can also use `QRMIService` with
`QRMI_JOB_QPU_RESOURCES=MAESTRO_LOCAL` and `QRMI_JOB_QPU_TYPES=maestro-local`.

The socket defaults to `/run/maestro.sock`; `QRMI_MAESTRO_SOCKET` overrides it.
`acquire()` retains the session in the resource. To share an existing session,
set `<backend_name>_QRMI_JOB_ACQUISITION_TOKEN` before constructing the resource.
The examples export their acquired token for that purpose.

## How it works

Each example builds a circuit (and, for the estimator, an observable) using
Qiskit, converts the circuit to QASM with `qiskit.qasm2.dumps()`, and submits
it as a `MaestroLocal` task payload to be executed by the local server.

### SamplerV2-style

[`sampler.py`](./sampler.py) builds a 3-qubit GHZ-state circuit, executes it,
and prints the measurement counts.

```shell-session
$ python sampler.py
```

### EstimatorV2-style

[`estimator.py`](./estimator.py) builds a Bell-state circuit and a pair of
Pauli observables (`ZZ`, `XX`), estimates their expectation values, and
prints the results.

```shell-session
$ python estimator.py
```

## Running as a Slurm job with task_runner

The two scripts above submit and poll the task themselves, which is useful for
interactive/local development. To instead run the same kind of workload as a
Slurm job, generate a JSON input file and hand it to the `task_runner`
executable:

* [examples/task_runner/maestro_local](../../task_runner/maestro_local) builds
  the same kind of circuits with Qiskit and writes them out as `task_runner`
  input files instead of submitting them directly.
* See the [task_runner documentation](../../../python/qrmi/tools/task_runner/README.md#maestro-local)
  for how to set the required environment variables (including the session
  acquisition token) and invoke `task_runner` against a Maestro Local
  resource, from a Slurm job script or locally.
