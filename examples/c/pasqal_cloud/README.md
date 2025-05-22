# Pasqal Cloud QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.


| Environment variables | Descriptions |
| ---- | ---- |

| <backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID` |  Pasqal Cloud Project ID to access the QPU
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token

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
$ ./build/pasqal-cloud
pasqal-cloud <backend_name> <input file>
```
For example,
```shell-session
$ ./build/pasqal-cloud FRESNEL input.json
```
