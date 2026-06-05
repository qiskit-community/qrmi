# QRMI Provider for Qiskit Runtime Service - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_RESOURCE_PROVIDER_CONFIG_FILE | qrmi config v2 file |

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
./build/qiskit_runtime_service_provider
```
For example,
```shell-session
export QRMI_RESOURCE_PROVIDER_CONFIG_FILE=/shared/qrmi_config.json

./build/qiskit_runtime_service_provider
```
