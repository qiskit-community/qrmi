# IQM Server QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| {qc_alias_name}_QRMI_IQM_ISA_ENDPOINT | IQM Server API endpoint |
| {qc_alias_name}_QRMI_IBM_ISA_TOKEN | IQM Server API token |

> [!NOTE]
> Replace the ":" in the QC alias name with "_" when specifying it. For example, `sirius:mock` -> `sirius_mock`.

## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit config map
instead of environment variables, via `qrmi_resource_new_from_config()`:

```c
QrmiKeyValue variables[] = {
    {(char *)"QRMI_IQM_ISA_ENDPOINT", (char *)"https://resonance.meetiqm.com"},
    {(char *)"QRMI_IQM_ISA_TOKEN", (char *)"your api token"},
};
QrmiConfigMap config = { .variables = variables, .length = 2 };

QrmiQuantumResource *qrmi = qrmi_resource_new_from_config(
    "garnet_mock", QRMI_RESOURCE_TYPE_IQM_SERVER, &config);
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

## Create IQM JSON input file as input

Refer [this tool](../../../task_runner/iqm) to generate. You can customize quantum circuits by editing the code.

> [!NOTE]
> Use the file with name ending `_params_only.json`, e.g. `iqm_json_sirius_params_only.json`.

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/iqm_server
iqm_server <qc_alias> <IQM JSON> <job_type('circuit','run' or 'sweep')
```
For example,
```shell-session
export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

./iqm_server garnet_mock /shared/qrmi/examples/task_runner/iqm/iqm_json_garnet\:mock_params_only.json circuit
```
