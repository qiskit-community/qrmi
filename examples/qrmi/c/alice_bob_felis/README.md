# Alice and Bob Felis - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* Build the [QRMI Rust library](../../../README.md)

## Set environment variables

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#set-environment-variables)

## Generate QIR Input file

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#generate-qir-input-file)

## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit config map
instead of environment variables, via `qrmi_resource_new_from_config()`:

```c
QrmiKeyValue variables[] = {
    {(char *)"QRMI_AB_FELIS_API_KEY", (char *)"<your felis api key>"},
    {(char *)"QRMI_AB_FELIS_BASE_ENDPOINT", (char *)"https://api.alice-bob.com/"},
};
QrmiConfigMap config = { .variables = variables, .length = 2 };

QrmiQuantumResource *qrmi = qrmi_resource_new_from_config(
    "ab_emu_1q_lescanne_2020", QRMI_RESOURCE_TYPE_ALICE_BOB_FELIS, &config);
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example

```shell-session
./felis <backend_name> <input file>
```
