# Pasqal Local API Client for Rust

Client to interact with Pasqal on prem QPUs.

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

## HTTPS / TLS trust

For `https://` endpoints, this client is built with reqwest's
`rustls-tls-native-roots` feature, so it trusts the **OS trust store** and the
`SSL_CERT_FILE` / `SSL_CERT_DIR` environment variables.

To trust a self-signed / private CA, do one of:

* install it into the OS trust store, e.g. on RHEL/Rocky:
  ```shell-session
  sudo cp rootCA.crt /etc/pki/ca-trust/source/anchors/
  sudo update-ca-trust extract
  ```
* or point `SSL_CERT_FILE` at a bundle that contains it.

The endpoint hostname must match a SAN in the server certificate.

## Examples

TODO

## API documents

Users can generate API document from source.
```shell-session
. ~/.cargo/env
cargo doc --no-deps --open
```
API document will be created under `../target/doc/pasqal_local_api` directory. 


## Logging

You can find the detailed runtime logs for Rust client by specifying `RUST_LOG` environment variable with log level.


```bash
RUST_LOG=trace <your command>
```

## Programming Guide

### Building API client instance

A ClientBuilder can be used to create a Client.
Currently assumed that the user will authenticate in a different way and provide the API token directly.


```rust
let client = ClientBuilder::new("http://localhost:8006")
```

### Invoking C++ API

You are ready to invoke Rust API by using created Client instance.

```cpp
  let job_id = client.create_job(&job).await?;
```

All API client related errors are delivered as Error in Result struct like other Rust functions.

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to Pasqal Local API Client for Rust!

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
API document will be created under `../target/doc/pasqal_local_api` directory.
