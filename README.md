# Quantum Resource Management Interface(QRMI)

## Supported OS

* Linux
  * AlmaLinux 9
  * Amazon Linux 2023
  * CentOS Stream 9
  * CentOS Stream 10
  * RedHat Enterprise Linux 8
  * RedHat Enterprise Linux 9
  * RedHat Enterprise Linux 10
  * Rocky Linux 8
  * Rocky Linux 9
  * SuSE 15
  * Ubuntu 22.04
  * Ubuntu 24.04

* macOS
  * Sequoia 15.1 or above

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12


## How to build Rust/C library
```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## How to build & install QRMI Python package

### Setup Python virtual environment
```shell-session
. ~/.cargo/env
cargo clean
python3.11 -m venv ~/py311_qrmi_venv
source ~/py311_qrmi_venv/bin/activate
pip install --upgrade pip
pip install -r requirements-dev.txt
```

### Build Python package and install to your Python virtual environment
```shell-session
source ~/py311_qrmi_venv/bin/activate
maturin develop --release
```

Once you successfully build and install, `qrmi` package is ready to use.
```shell-session
$ pip list
qrmi                   0.1.0       /Users/devuser/git/spank-plugins/qrmi

$ pip show qrmi
Name: qrmi
Version: 0.1.0
Summary: 
Home-page: 
Author: 
Author-email: 
License: 
Location: /Users/devuser/py311_qrmi_venv/lib/python3.11/site-packages
Editable project location: /Users/devuser/git/spank-plugins/qrmi
Requires: 
Required-by: 
```

## Examples

* [Examples in Rust](./examples/rust)
* [Examples in Python](./examples/python)
* [Examples in C](./examples/c)

## How to generate Rust API document

```shell-session
. ~/.cargo/env
cargo doc --no-deps --open
```

## Contributing

Regardless if you are part of the core team or an external contributor, welcome and thank you for contributing to QRMI implementations!

### Solving linting/format issues

Contributor must execute the commands below and fix any issues before submitting Pull Request.

#### Rust code
```shell-session
$ . ~/.cargo/env
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
$ cd examples/rust
$ cargo fmt --all -- --check
$ cargo clippy --all-targets -- -D warnings
```

#### Python code
```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ cd examples
$ pylint ./python
$ black --check ./python
```
