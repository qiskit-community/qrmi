# Parsing QRMI config file in Rust

## Prerequisites

* Python 3.11 or 3.12
* [QRMI Rust library](../../../README.md)

## How to build this example

```shell-session
$ cargo clean
$ cargo build --release
```

## How to run this example
```shell-session
$ ../target/release/qrmi-example-config --help
Parsing qrmi_config.json file

Usage: qrmi-example-config --file <FILE>

Options:
  -f, --file <FILE>  qrmi_config.json file
  -h, --help         Print help
  -V, --version      Print version
```
For example,
```shell-session
../target/release/qrmi-example-config -f /etc/slurm/qrmi_config.json
```
