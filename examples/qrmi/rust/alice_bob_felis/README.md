# Alice Bob Felis QRMI - Examples in Rust

## Prerequisites

* C compiler/linker, cmake and make
* Build the [QRMI Rust library](../../../README.md)

## Set environment variables

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#set-environment-variables)

## Generate QIR Input file

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#generate-qir-input-file)

## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit config map
instead of environment variables, via `AliceBobFelis::from_config()`:

```rust
use qrmi::alice_bob::AliceBobFelis;
use std::collections::HashMap;

let config = HashMap::from([
    ("QRMI_AB_FELIS_API_KEY".to_string(), "<your felis api key>".to_string()),
    ("QRMI_AB_FELIS_BASE_ENDPOINT".to_string(), "https://api.alice-bob.com/".to_string()),
]);
let qrmi = AliceBobFelis::from_config("ab_emu_1q_lescanne_2020", config)?;
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

## How to build this example

```shell-session
cargo clean
CARGO_TARGET_DIR=./target cargo build --release
```

## How to run this example

```shell-session
qrmi-example-alice-bob-felis --backend <BACKEND> --input <INPUT>
```

For example,

```shell-session
 ./target/debug/qrmi-example-alice-bob-felis --backend 'ab_emu_1q_lescanne_2020' --input ./generated_circuit.ll
```
