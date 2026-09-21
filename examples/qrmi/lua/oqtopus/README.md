# OQTOPUS QRMI - Examples in Lua

## Prerequisites

* [QRMI C library(libqrmi.so)](../../../../README.md#standalone-c-library)
* [QRMI Lua Module(qrmi.so)](../../../../lua/README.md)

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


## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below.

| Environment variables | Descriptions |
| ---- | ---- |
| {device_id}_QRMI_OQTOPUS_BASE_URL | OQTOPUS Cloud API endpoint |
| {device_id}_QRMI_OQTOPUS_API_TOKEN | OQTOPUS Cloud API token |

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/oqtopus) to generate. You can customize quantum circuits by editting the code.

> [!NOTE]
> Use the file with name ending with `_params_only.json`, e.g. `oqtopus_params_only.json`.


## How to run this example
```shell-session
lua example.lua <device_id>
```
For example,
```shell-session
export qulacs_QRMI_OQTOPUS_BASE_URL=https://demo-api.oqtopus.io
export qulacs_QRMI_OQTOPUS_API_TOKEN=your api token

lua example.lua qulacs
```
