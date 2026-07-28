# IQM Server QRMI - Examples in Lua

## Prerequisites

* [QRMI C library](../../../../README.md)
* [QRMI Lua Wrapper](../../../../lua/README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below.

| Environment variables | Descriptions |
| ---- | ---- |
| {qc_alias_name}_QRMI_IQM_ISA_ENDPOINT | IQM Server API endpoint |
| {qc_alias_name}_QRMI_IBM_ISA_TOKEN | IQM Server API token |

> [!NOTE]
> Replace the “:” in the QC alias name with “_” when specifying it. For example, `sirius:mock` -> `sirius_mock`.

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/iqm) to generate. You can customize quantum circuits by editting the code.

> [!NOTE]
> Use the file with name ending with `_params_only.json`, e.g. `iqm_json_sirius_params_only.json`.


## How to run this example
```shell-session
lua example.lua <qc_alias> <IQM JSON> <job_type('circuit','run' or 'sweep')>
```
For example,
```shell-session
export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

lua example.lua garnet_mock ../../../task_runner/iqm/iqm_json_garnet\:mock.json circuit
```
