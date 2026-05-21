# Alice and Bob Felis - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* Build the [QRMI Rust library](../../../README.md)

## Set environment variables

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#set-environment-variables)

## Generate QIR Input file

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#generate-qir-input-file)

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
