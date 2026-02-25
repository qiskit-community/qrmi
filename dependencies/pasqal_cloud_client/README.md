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

A `ClientBuilder` can be used to create a `Client`.

```rust
use pasqal_cloud_api::ClientBuilder;

let mut builder = ClientBuilder::new("<API token>".to_string(), "<project id>".to_string());
let client = builder.build()?;
```

### Requesting an access token

If username/password auth is used, `request_access_token` can be used to obtain a token.

```rust
use pasqal_cloud_api::{Client, DEFAULT_AUTH_ENDPOINT};

let token = Client::request_access_token(
    DEFAULT_AUTH_ENDPOINT,
    "<username>",
    "<password>",
).await?;
```

### Checking JWT expiry

`jwt_expiry_unix_seconds` returns the JWT `exp` claim (if present), so callers can refresh expired tokens.

```rust
use pasqal_cloud_api::Client;

let exp = Client::jwt_expiry_unix_seconds("<jwt token>")?;
```

### Invoking Rust API

You are ready to invoke Rust API by using created Client instance.

```rust
let batch = client.create_batch(sequence, job_runs, device_type).await?;
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
