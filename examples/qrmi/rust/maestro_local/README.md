# Maestro Local QRMI - Examples in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../../README.md)
* A running [Maestro Local server](https://github.com/QoroQuantum/maestro-local-server) ([installation guide](https://github.com/QoroQuantum/maestro-local-server/blob/main/docs/installation.md)), reachable at the Unix domain socket `/run/maestro.sock`

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

## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/qrmi-example-maestro-local --help
QRMI for Maestro Local - Example

Usage: qrmi-example-maestro-local [OPTIONS] --backend <BACKEND> --input <INPUT>

Options:
  -b, --backend <BACKEND>
          Backend name (device identifier)
  -i, --input <INPUT>
          QASM input file
  -j, --job-type <JOB_TYPE>
          Job type ('execute' or 'estimate') [default: execute]
  -q, --qubits <QUBITS>
          Number of qubits [default: 5]
  -s, --simulator-type <SIMULATOR_TYPE>
          Simulator type (0 = aer, 1 = qcsim) [default: 0]
  -m, --simulation-method <SIMULATION_METHOD>
          Simulation method (0 = statevector, 1 = matrix_product_state) [default: 0]
  -o, --observables <OBSERVABLES>
          Observables (pauli strings separated by ";"), required for 'estimate' job type [default: ""]
  -c, --config <CONFIG>
          Task configuration, in JSON format [default: {}]
  -h, --help
          Print help
  -V, --version
          Print version
```

For example,
```shell-session
$ ../target/release/qrmi-example-maestro-local -b MAESTRO_LOCAL -i input.qasm -q 2
```

To run an `estimate` job, pass observables (Pauli strings separated by `;`):
```shell-session
$ ../target/release/qrmi-example-maestro-local -b MAESTRO_LOCAL -i input.qasm -q 2 -j estimate -o "ZZ;XX"
```
