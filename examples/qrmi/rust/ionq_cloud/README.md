# QRMI IonQ Cloud Rust Example

This example uses the QRMI `QuantumResource` interface to run a simple circuit on IonQ Cloud
via `qrmi::ionq::IonQCloud`.

## Prerequisites

- Rust toolchain as required by the repo (see `rust-toolchain.toml`)
- An IonQ API key exported as:

```bash
export QRMI_IONQ_CLOUD_API_KEY="YOUR_IONQ_API_KEY"
```

> Note: Don't commit your API key. Prefer `.env` or a secrets manager.

## Run (real IonQ Cloud)

From the repo root:

```bash
RUST_LOG=info,qrmi=debug,ionq_cloud_api=debug \
cargo run --manifest-path examples/qrmi/rust/ionq_cloud/Cargo.toml -- \
  --backend simulator \
  --shots 100 \
  --format qasm2
```

Supported backend names (as implemented in `IonQCloud::new`) typically include:

* `simulator`
* `qpu.harmony`
* `qpu.aria-1`
* `qpu.aria-2`
* `qpu.forte-1`
* `qpu.forte-enterprise-1`
* `qpu.forte-enterprise-2`

## Run (offline mock)

If you want to validate QRMI wiring without network credentials:

```bash
RUST_LOG=debug \
cargo run --manifest-path examples/qrmi/rust/ionq_cloud/Cargo.toml -- \
  --mock \
  --backend simulator \
  --shots 100 \
  --format qasm3
```

## Build + Test

Build the example:

```bash
cargo build --manifest-path examples/qrmi/rust/ionq_cloud/Cargo.toml
```

Run the repo test suite from the root:

```bash
cargo test
```

(Optional) lint/format:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```
