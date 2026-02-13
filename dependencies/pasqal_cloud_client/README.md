# Pasqal Cloud API Client for Rust

Like [this](https://github.com/pasqal-io/pasqal-cloud), in Rust.

We refer to Pasqal's cloud API [documetation](https://docs.pasqal.com/cloud/api/core/#overview).

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

Users can generate API document from source.
```shell-session
. ~/.cargo/env
cargo doc --no-deps --open
```
API document will be created under `../target/doc/pasqal_cloud_api` directory. 


## Logging

You can find the detailed runtime logs for Rust client by specifying `RUST_LOG` environment variable with log level.


```bash
RUST_LOG=trace <your command>
```

## Programming Guide

### Building API client instance

`ClientBuilder` creates an auth-aware `Client`.

You should provide either:
- username + password, or
- API token

Default flow (username + password):

```rust
use pasqal_cloud_api::ClientBuilder;

let mut builder = ClientBuilder::for_project("<project id>".to_string());
builder.with_credentials("<username>".to_string(), "<password>".to_string());
let mut client = builder.build()?;
```

Token flow:

```rust
use pasqal_cloud_api::ClientBuilder;

let mut builder = ClientBuilder::for_project("<project id>".to_string());
builder.with_token("<api token>".to_string());
let mut client = builder.build()?;
```

### Invoking Rust API

You are ready to invoke Rust API by using created Client instance.

```rust
let batch = client.create_batch(sequence, job_runs, device_type).await?;
```

### Optional: custom auth endpoint

Most users do not need to set this. Use only when a non-default auth endpoint is required.

```rust
use pasqal_cloud_api::ClientBuilder;

let mut builder = ClientBuilder::for_project("<project id>".to_string());
builder.with_credentials("<username>".to_string(), "<password>".to_string());
builder.with_auth_endpoint("<auth endpoint>".to_string());
let mut client = builder.build()?;
```

All API client related errors are delivered as Error in Result struct like other Rust functions.

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to Pasqal Cloud API Client for Rust!

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
API document will be created under `../target/doc/pasqal_cloud_api` directory.
