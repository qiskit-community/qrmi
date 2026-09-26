# OQTOPUS QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {device_id}_QRMI_OQTOPUS_BASE_UR | OQTOPUS Cloud API endpoint |
| {device_id}_QRMI_OQTOPUS_API_TOKEN | OQTOPUS Cloud API token |

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/oqtopus) to generate.

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/oqtopus
oqtopus <device_id> <QASM program file> <job_type('sampling','estimation', 'multi_manual' or 'sse')>
```
For example,
```shell-session
export qulacs_QRMI_OQTOPUS_BASE_URL=https://demo-api.oqtopus.io
export qulacs_QRMI_OQTOPUS_API_TOKEN=your api token

./oqtopus qulacs ../../../../task_runner/oqtopus/bell_state.txt sampling
```
