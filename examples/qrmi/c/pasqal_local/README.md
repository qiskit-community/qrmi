# Pasqal Local QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)
* [Munge](https://dun.github.io/munge/)


## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| `<backend_name>_QRMI_WARDEN_URL` | URL of the QPU middleware (e.g. `http://localhost:4207`). Falls back to the deprecated `<backend_name>_QRMI_URL` if not set. |
| `QRMI_JOB_UID` | ID of the user executing the job |
| `QRMI_JOB_ID` | ID of the job |

Where `<backend_name>` is the backend name passed as the first argument (e.g. `PASQAL_LOCAL`).



## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit config map
instead of environment variables, via `qrmi_resource_new_from_config()`:

```c
QrmiKeyValue variables[] = {
    {(char *)"QRMI_WARDEN_URL", (char *)"http://localhost:4207"},
    {(char *)"QRMI_JOB_ID",     (char *)"1"},
    {(char *)"QRMI_JOB_UID",    (char *)"1000"},
};
QrmiConfigMap config = { .variables = variables, .length = 3 };

QrmiQuantumResource *qrmi = qrmi_resource_new_from_config(
    "PASQAL_LOCAL", QRMI_RESOURCE_TYPE_PASQAL_LOCAL, &config);
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

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
