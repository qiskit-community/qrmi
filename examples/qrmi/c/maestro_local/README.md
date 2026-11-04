# Maestro Local QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)
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

Where `<backend_name>` is the backend name passed as the first argument (e.g. `MAESTRO_LOCAL`).

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
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/maestro_local
maestro_local <backend name> <QASM input file> [job_type('execute' or 'estimate'), default 'execute'] [qubits, default 5] [observables (pauli strings separated by ';'), default '']
```
For example,
```shell-session
$ ./build/maestro_local MAESTRO_LOCAL input.qasm execute 2
```

To run an `estimate` job, pass observables (Pauli strings separated by `;`):
```shell-session
$ ./build/maestro_local MAESTRO_LOCAL input.qasm estimate 2 "ZZ;XX"
```
