# Installation for Quantum Resource Management Interface(QRMI)

> **Supported OS**: AlmaLinux 9, Amazon Linux 2023, CentOS Stream 9, CentOS Stream 10, RedHat Enterprise Linux 8, RedHat Enterprise Linux 9, RedHat Enterprise Linux 10, Rocky Linux 8, Rocky Linux 9, SuSE 15, Ubuntu 22.04, Ubuntu 24.04, MacOS Sequoia 15.1 or above

## Prerequisites

* Compilation requires the following tools:
  * Rust compiler 1.86 or above [Link](https://www.rust-lang.org/tools/install)
  * A C compiler: for example, GCC(gcc) on Linux and Clang(clang-tools-extra) for Rust unknown targets/cross compilations. QRMI is compatible with a compiler conforming to the C11 standard.
  * make/cmake (make/cmake RPM for RHEL compatible OS)
  * openssl (openssl-devel RPM for RHEL compatible OS)
  * zlib (zlib-devel RPM for RHEL compatible OS)
  * Python 3.11, 3.12 or 3.13 (For Python API)

* Runtime requires the following tools:
  * gcc (libgcc RPM for RHEL compatible OS)
  * openssl (openssl-libs RPM for RHEL compatible OS)
  * zlib (zlib RPM for RHEL compatible OS)
  * Python 3.11, 3.12 or 3.13 (For Python API)

* Doxygen (for generating C API document)
  * ```dnf install doxygen``` for Linux(RHEL/CentOS/Rocky Linux etc)
  * ```apt install doxygen``` for Linux(Ubuntu etc.)
  * ```brew install doxygen```for MacOS


## 📋 Content

- [🔶 Building core QRMI library](#building-core-qrmi-libraries)
  - [🦀/©️ Rust and C](#how-to-build-rustc-api-library)
  - [🐍 Python](#how-to-build--install-qrmi-python-package)
- [🔸 Building optional libraries](#building-optional-libraries)
  - [🏃 Building task runner CLI](#how-to-build-task_runner-for-rust-version)
  - [🏃 Building task runner for Python](#how-to-run-task_runner-for-python-version)
- [🔹 Other](#other)
  - [📋 Examples](#examples)
  - [📃 Logging](#logging)
  - [📄 Generate API docs](#api-docs)
    - [🦀 Rust API docs](#how-to-generate-rust-api-document)
    - [🐍 Pythoni API docs](#how-to-generate-python-api-document)
    - [©️ C API docs](#how-to-generate-c-api-document)
  - [Contributing](#contributing)


## Building core QRMI libraries

Core QRMI is a set of libraries to control state of quantum resources. Written in Rust with C and Python API exposed for ease of integration to any compute infrastructure. 

This section shows how to build QRMI for C and Python.

### How to build Rust/C API library
```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

### How to build & install QRMI Python package

#### Setup Python virtual environment
```shell-session
. ~/.cargo/env
cargo clean
python3.12 -m venv ~/py312_qrmi_venv
source ~/py312_qrmi_venv/bin/activate
pip install --upgrade pip
pip install -r requirements-dev.txt
```


#### Create stub file for python code
```shell-session
. ~/.cargo/env
cargo run --bin stubgen --features=pyo3
```


#### Create a wheel for distribution

```shell-session
source ~/py312_qrmi_venv/bin/activate
CARGO_TARGET_DIR=./target/release/maturin maturin build --release
```

For example,
```shell-session
CARGO_TARGET_DIR=./target/release/maturin maturin build --release

🍹 Building a mixed python/rust project
🔗 Found pyo3 bindings with abi3 support
🐍 Found CPython 3.12 at /root/py312_qrmi_venv/bin/python
📡 Using build options features from pyproject.toml
   ...
   Compiling qrmi v0.7.1 (/shared/qrmi)
    Finished `release` profile [optimized] target(s) in 1m 10s
🖨  Copied external shared libraries to package qrmi.libs directory:
    /usr/lib64/libcrypto.so.3.2.2
    /usr/lib64/libssl.so.3.2.2
📦 Including files matching "python/qrmi/py.typed"
📦 Including files matching "python/qrmi/*.pyi"
📦 Built wheel for abi3 Python ≥ 3.12 to /shared/qrmi/target/release/maturin/wheels/qrmi-0.7.1-cp312-abi3-manylinux_2_34_aarch64.whl
```

Wheel is created under `./target/release/maturin/wheels` directory. You can distribute and install on your hosts by `pip install <wheel>`.

```shell-session
source ~/py312_qrmi_venv/bin/activate
pip install /shared/qrmi/target/release/maturin/wheels/wheels/qrmi-0.7.1-cp312-abi3-manylinux_2_34_aarch64.whl
```


## Building optional libraries

Optional packages that are available in QRMI repository.

`task_runner` is command line command to execute quantum payload againts quantum hardware. Under the hood it is using QRMI library.


### How to build task_runner for Rust version

> [!WARNING]
> Rust version of task_runner is now obsoleted. Use Python version.

```shell-session
. ~/.cargo/env
cargo build -p task_runner 
```


### How to run task_runner for Python version
`task_runner` for Python version is already included in qrmi package. User can use task_runner command after installing qrmi. 
For detailed instructions on how to use it, please refer to this [README](./bin/task_runner/README.md).


## Other


### Examples

* [Examples in Rust](./examples/qrmi/rust)
* [Examples in Python](./examples/qrmi/python)
* [Examples in C](./examples/qrmi/c)


### Logging

QRMI supports [log crate](https://crates.io/crates/log) for logging. You can find the detailed QRMI runtime logs by specifying `RUST_LOG` environment variable with log level. Supported levels are `error`, `warn`, `info`, `debug` and `trace`. Default level is `warn`. 

If you specify `trace`, you can find underlying HTTP transaction logs.


```shell-session
RUST_LOG=trace <your QRMI executable>
```

```shell-session
[2025-08-16T03:47:38Z DEBUG reqwest::connect] starting new connection: https://iam.cloud.ibm.com/
[2025-08-16T03:47:38Z DEBUG direct_access_api::middleware::auth] current token ...
```


### API Docs


#### How to generate Rust API document

```shell-session
. ~/.cargo/env
cargo doc --no-deps --open
```


#### How to generate Python API document

Prerequisites: QRMI Python package is installed in your python virtual environment(e.g. `~/py312_qrmi_venv`)

```shell-session
source ~/py312_qrmi_venv/bin/activate
python -m pydoc -p 8290
Server ready at http://localhost:8290/
Server commands: [b]rowser, [q]uit
server> b
```

Open the following page in your browser.
```shell-session
http://localhost:8290/qrmi.html 
```

Quit server.
```shell-session
server> q
```


#### How to generate C API document

##### Generating API document
```shell-session
doxygen Doxyfile
```

HTML document will be created under `./html` directory. Open `html/index.html` in your web browser. 


### Contributing

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
