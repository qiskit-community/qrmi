# Maestro Local QRMI - Examples in Python

## Prerequisites

* Rust 1.91.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)
* A running [Maestro Local server](https://github.com/QoroQuantum/maestro-local-server) ([installation guide](https://github.com/QoroQuantum/maestro-local-server/blob/main/docs/installation.md)), reachable at the Unix domain socket `/run/maestro.sock`

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Environment variables

Maestro Local connects to `/run/maestro.sock` by default. Set the optional
`QRMI_MAESTRO_SOCKET` variable to use another socket. The resource retains the
session returned by `acquire()`; `<backend_name>_QRMI_JOB_ACQUISITION_TOKEN` is only
needed to reuse a session in another resource instance or process, and must be
set before constructing that resource.

Maestro is also available through `QRMIService`. See the
[Maestro native request guide](../../../../docs/maestro-native-api.rst) for native
request configuration and simulation details.

Where `<backend_name>` is the backend name passed via `--backend` (e.g. `MAESTRO_LOCAL`).

## Create QASM input file

Provide a plain-text file containing the QASM (or other request payload) to submit, e.g.:

```shell-session
$ cat input.qasm
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];
h q[0];
cx q[0],q[1];
measure q -> c;
```

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] --backend BACKEND [--job-type JOB_TYPE] [--qubits QUBITS]
                   [--simulator-type SIMULATOR_TYPE]
                   [--simulation-method SIMULATION_METHOD]
                   [--observables OBSERVABLES] [--config CONFIG]
                   input

An example of Maestro Local QRMI

positional arguments:
  input                 QASM input file

options:
  -h, --help            show this help message and exit
  --backend BACKEND     Backend name (device identifier)
  --job-type JOB_TYPE   Job type ('execute' or 'estimate')
  --qubits QUBITS       Number of qubits
  --simulator-type SIMULATOR_TYPE
                        Simulator type (0 = aer, 1 = qcsim)
  --simulation-method SIMULATION_METHOD
                        Simulation method (0 = statevector, 1 = matrix_product_state)
  --observables OBSERVABLES
                        Observables (pauli strings separated by ";"), required for
                        "estimate" job type
  --config CONFIG       Task configuration, in JSON format
```

For example,
```shell-session
$ python example.py --backend MAESTRO_LOCAL --qubits 2 input.qasm
```

To run an `estimate` job, pass observables (Pauli strings separated by `;`):
```shell-session
$ python example.py --backend MAESTRO_LOCAL --qubits 2 --job-type estimate --observables "ZZ;XX" input.qasm
```
