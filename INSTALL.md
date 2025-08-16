# Installation for Quantum Resource Management Interface(QRMI)

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
* Python 3.11, 3.12 or 3.13
* doxygen (for generating C API document)


## How to build Rust/C API library
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
python3.12 -m venv ~/py312_qrmi_venv
source ~/py312_qrmi_venv/bin/activate
pip install --upgrade pip
pip install -r requirements-dev.txt
```

### Create stub file for python code
```shell-session
. ~/.cargo/env
cargo run --bin stubgen --features=pyo3
```

### Build Python module and install to your Python virtual environment
```shell-session
source ~/py312_qrmi_venv/bin/activate
CARGO_TARGET_DIR=./target/release/maturin maturin develop --release
```

Once you successfully build and install, `qrmi` package is ready to use.
```shell-session
$ pip list
qrmi                   0.5.2       /Users/devuser/git/spank-plugins/qrmi

$ pip show qrmi
Name: qrmi
Version: 0.5.2
Summary: Quantum Resource Management Interface(QRMI)
Home-page: 
Author: IBM, Pasqal SAS and UKRI-STFC (Hartree Centre)
Author-email: qiskit@us.ibm.com
License: Apache-2.0
Location: /shared/pyenv/lib64/python3.12/site-packages
Editable project location: /shared/spank-plugins/qrmi
Requires: 
Required-by: qiskit-qrmi-primitives
```


### Create a wheel for distribution

```shell-session
source ~/py312_qrmi_venv/bin/activate
CARGO_TARGET_DIR=./target/release/maturin maturin build --release
```

For example,
```shell-session
CARGO_TARGET_DIR=./target/release/maturin maturin build --release
🔗 Found pyo3 bindings with abi3 support
🐍 Found CPython 3.12 at /shared/pyenv/bin/python
📡 Using build options features from pyproject.toml
   Compiling qrmi v0.5.2 (/shared/spank-plugins/qrmi)
    Finished `release` profile [optimized] target(s) in 12.76s
🖨  Copied external shared libraries to package qrmi.libs directory:
    /usr/lib64/libssl.so.3.2.2
    /usr/lib64/libcrypto.so.3.2.2
📦 Built wheel for abi3 Python ≥ 3.12 to /shared/spank-plugins/qrmi/target/release/maturin/wheels/qrmi-0.5.2-cp312-abi3-manylinux_2_34_aarch64.whl
```

Wheel is created under `./target/release/maturin/wheels` directory. You can distribute and install on your hosts by `pip install <wheel>`.

```shell-session
source ~/py312_qrmi_venv/bin/activate
pip install /shared/spank-plugins/qrmi/target/release/maturin/wheels/qrmi-0.5.2-cp312-abi3-manylinux_2_34_aarch64.whl
```

## How to build task_runner for Rust version
```shell-session
. ~/.cargo/env
cargo build -p task_runner 
```

## How to run task_runner for Python version
`task_runner` for Python version is already included in qrmi package. User can use task_runner command after installing qrmi. 
For detailed instructions on how to use it, please refer to this [README](./bin/task_runner/README.md).

## Logging

QRMI supports [standard Rust logging mechanism](https://docs.rs/log/latest/log/) for logging. You can find the detailed QRMI runtime logs by specifying `RUST_LOG` environment variable with log level. Supported levels are `error`, `warn`, `info`, `debug` and `trace`. Default level is `warn`. 

If you specify `trace`, you can find underlying HTTP transaction logs.


```shell-session
RUST_LOG=trace <your QRMI executable>
```

```shell-session
[2025-08-16T03:47:38Z DEBUG reqwest::connect] starting new connection: https://iam.cloud.ibm.com/
[2025-08-16T03:47:38Z DEBUG direct_access_api::middleware::auth] current token ...
```


## Examples

* [Examples in Rust](./examples/qrmi/rust)
* [Examples in Python](./examples/qrmi/python)
* [Examples in C](./examples/qrmi/c)

## How to generate Rust API document

```shell-session
. ~/.cargo/env
cargo doc --no-deps --open
```

## Note
`get_target` method has been refactored into a library, so we updated the import statement.

Before
```
from target import get_target
```
After
```
from qrmi.primitives.ibm import get_target
```

## How to generate C API document

### Installing doxygen
#### Linux
```shell-session
dnf install doxygen
```

#### MacOS
```shell-session
brew install doxygen
```

### Generating API document
```shell-session
doxygen Doxyfile
```

HTML document will be created under `./html` directory. Open `html/index.html` in your web browser. 

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
$ source ~/py312_qrmi_venv/bin/activate
$ cd examples
$ pylint ./python
$ black --check ./python
```

## License

[Apache-2.0](https://github.com/qiskit-community/spank-plugins/blob/main/qrmi/LICENSE.txt)
