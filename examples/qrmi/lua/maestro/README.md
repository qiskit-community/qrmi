# Maestro Local QRMI - Examples in Lua

## Prerequisites

* [QRMI C library(libqrmi.so)](../../../../README.md#standalone-c-library)
* [QRMI Lua Module(qrmi.so)](../../../../lua/README.md)
* A running Maestro Local server, reachable at the Unix domain socket `/run/maestro.sock`

## Setup

```bash
export LUA_CPATH="</path/to/qrmi.so-dir/>?.so;;"
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libqrmi.so-dir
```

Example:
```bash
export LUA_CPATH="/shared/qrmi/lua/build/?.so;;"
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/shared/qrmi/target/release
```

## Environment variables

Maestro Local connects to a fixed local Unix socket, so no endpoint/token environment
variables are required.

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

## How to run this example
```shell-session
lua example.lua <backend name> <QASM input file> [job_type('execute' or 'estimate'), default 'execute'] [qubits, default 5] [observables (pauli strings separated by ';'), default '']
```
For example,
```shell-session
lua example.lua MAESTRO_LOCAL input.qasm execute 2
```

To run an `estimate` job, pass observables (Pauli strings separated by `;`):
```shell-session
lua example.lua MAESTRO_LOCAL input.qasm estimate 2 "ZZ;XX"
```
