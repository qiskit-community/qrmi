# Pasqal Local QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)
* [Munge](https://dun.github.io/munge/)


## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| `<backend_name>_QRMI_URL` | URL of the QPU middleware (e.g. `http://localhost:4207`) |
| `QRMI_JOB_UID` | ID of the user executing the job |
| `QRMI_JOB_ID` | ID of the job |

Where `<backend_name>` is the backend name passed as the first argument (e.g. `PASQAL_LOCAL`).



## Create Pulser Sequence file as input

Given a Pulser sequence `sequence`, we can convert it to a JSON string and write it to a file like this:

```python
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)
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
$ ./build/pasqal_local
pasqal_local <backend name> <input file>
```
For example,
```shell-session
$ ./build/pasqal_local PASQAL_LOCAL input.json
```
