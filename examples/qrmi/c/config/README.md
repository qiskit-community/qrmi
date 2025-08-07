# Parsing QRMI config file in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../../README.md)

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/
qrmi_config <qrmi_config.json file> <resource name>
```
For example,
```shell-session
./build/qrmi_config /etc/slurm/qrmi_config.json ibm_fez
```
