# Description

Quantum vendors provide a public API for customers to access their QPU devices. Typically, each quantum vendor defines its own API.
QRMI aims to uniform access to these APIs through the interfaces and data types defined in this crate.

# Bindings

QRMI provides bindings to other languages, but they are not provided in this crate.

# New releases

When changes are made to this package, we communicate the changes to consumers by [bumping the crate version](https://semver.org/) before publishing a new QRMI release.

For example, command to bump the crate patch version
```
cargo install cargo-edit --locked
cargo set-version -p qrmi-core-api --bump patch
```