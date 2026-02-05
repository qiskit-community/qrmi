# IonQ Cloud API Client for Rust

We refer to IonQ's cloud API [documentation](https://docs.ionq.com/api-reference/v0.4/introduction).

## Supported OS

* Linux
  * AlmaLinux 9
  * Amazon Linux 2023
  * RedHat Enterprise Linux 8
  * RedHat Enterprise Linux 9
  * SuSE 15
  * Ubuntu 22.04
  * Ubuntu 24.04

* macOS
  * Sequoia 15.1 or above

## Prerequisites

* Rust 1.81.0 or above

## How to build
```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## Examples
TODO

## API documents
TODO

## Logging

You can find the detailed runtime logs for Rust client by specifying `RUST_LOG` environment variable with log level.


```bash
RUST_LOG=trace <your command>
```

## Programming Guide

### Building API client instance
TODO

### Invoking C++ API
TODO

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to IonQ Cloud API Client for Rust!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Rust code
```shell-session
$ . ~/.cargo/env
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
```

### Running unit test

Contributor must execute the command below and fix any issues before submitting Pull Request.
```shell-session
$ . ~/.cargo/env
$ cargo test
```

### Checking API document

Contributor can generate API document from source.
```shell-session
$ . ~/.cargo/env
$ cargo doc --no-deps
```
API document will be created under `../target/doc/ionq_cloud_api` directory.
