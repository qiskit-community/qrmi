# Installation for Quantum Resource Management Interface (QRMI)

## Quick Start

We encourage installing QRMI via `pip`:

``` bash
pip install qrmi
```

To use a specific quantum resource, install QRMI with the corresponding
optional dependencies:

``` bash
pip install "qrmi[ibm]"       # Include dependencies for IBM
pip install "qrmi[iqm]"       # Include dependencies for IQM
pip install "qrmi[pasqal]"    # Include dependencies for Pasqal
pip install "qrmi[alice-bob]" # Include dependencies for Alice and Bob
pip install "qrmi[all]"       # Include dependencies for all quantum resources except `alice-bob`
```

Or combine multiple resources:

``` bash
pip install "qrmi[ibm,pasqal]"
```

> [!note]
> `ibm` and `iqm` extras cannot be installed together, as they
> depend on incompatible versions of Qiskit.

> [!note]
> `alice-bob` cannot be installed alongside `ibm` or `iqm`, as
> it depends on Qiskit versions earlier than 2.0.

Pip will handle all dependencies automatically and you will always
install the latest (and most thoroughly tested) version.

## Content

- [Quick Start](#quick-start)
- [Installing from Source](#installing-from-source)
  - [Prerequisites](#prerequisites)
  - [Building Core QRMI Libraries](#building-core-qrmi-libraries)
    - [QRMI Source Code](#qrmi-source-code)
    - [Building from Source](#building-from-source)
    - [Installing Lua Bindings](#installing-lua-bindings)
    - [Building Lua Bindings](#building-lua-bindings)
  - [Building Optional Libraries](#building-optional-libraries)
    - [Building `task_runner`](#building-task_runner)
      - [Running with Python](#running-with-python)
      - [Build with Munge support for Pasqal Local](#build-with-munge-support-for-pasqal-local)
- [Further Resources](#further-resources)
  - [Examples](#examples)
  - [Logging](#logging)
  - [API Documentation](#api-documentation)
    - [How to generate Rust API documents](#how-to-generate-rust-api-documents)
    - [How to generate Python API documents](#how-to-generate-python-api-documents)
    - [How to generate C API documents](#how-to-generate-c-api-documents)
  - [Packaging](#packaging)
  - [Contributing](#contributing)
  - [Linting/Formatting](#lintingformatting)
  - [Rust Formatting](#rust-formatting)
  - [Python Formatting](#python-formatting)
  - [Help and Support](#help-and-support)

## Installing from Source

### Prerequisites

- Compilation requires the following tools:
  - [Rust compiler 1.91 or above](https://www.rust-lang.org/tools/install)
  - A C compiler
    - For example, GCC (gcc) on Linux and Clang (clang-tools-extra) for Rust unknown targets/cross compilations. QRMI is compatible with a compiler conforming to the C11 standard.
    - make/cmake (make/cmake RPM for RHEL compatible OS)
    - Python 3.11, 3.12 or 3.13 (for Python API)
      - Libraries and header files needed for Python development (python3.1x-devel RPM for RHEL compatible OS)
        - `/usr/include/python3.1x`
        - `/usr/lib64/libpython3.1x.so`
    - Standard Lua (PUC-Rio Lua 5.1-5.4)
    - QRMI Standalone C library & header
- Runtime requires the following tools:
  - gcc (libgcc RPM for RHEL compatible OS)
  - Python 3.11, 3.12 or 3.13 (for Python API)
    - Libraries and header files needed for Python development (python3.1x-devel RPM for RHEL compatible OS)
- Doxygen (for generating C API document):
  - `dnf install doxygen` for Linux(RHEL/CentOS/Rocky Linux etc.)
  - `apt install doxygen` for Linux(Ubuntu etc.)
  - `brew install doxygen` for MacOS

### Building Core QRMI Libraries

Core QRMI is a set of libraries to control the state of quantum
resources. It is written in Rust with C, Python and Lua APIs exposed for ease of integration into any compute infrastructure.

#### QRMI Source Code

QRMI's source code can be cloned from the GitHub repository using the
following command:

##### HTTPS

``` bash
git clone https://github.com/qiskit-community/qrmi.git
```

##### SSH

``` bash
git clone git@github.com:qiskit-community/qrmi.git
```

Alternatively, the latest prebuilt binaries for Linux (glibc 2.28
compatible) on x86_64, ppc64le, and aarch64 platforms are available for download from the repository's [Releases
tab](https://github.com/qiskit-community/qrmi/releases/latest).

#### Building from Source

This section will guide you through building QRMI for C, Python and Lua.

##### Rust/C

``` bash
. ~/.cargo/env
cargo clean
cargo build --locked --release
```

##### Python

1. Setup a Python virtual environment

``` bash
. ~/.cargo/env
cargo clean
python3.12 -m venv ~/py312_qrmi_venv
source ~/py312_qrmi_venv/bin/activate
pip install --upgrade pip
pip install -r requirements-dev.txt
```

1. Create stub file for Python code

``` bash
. ~/.cargo/env
cargo run --bin stubgen --features=pyo3
```

1. Create a wheel for distribution

``` bash
source ~/py312_qrmi_venv/bin/activate
CARGO_TARGET_DIR=./target/release/maturin maturin build --release
```

For example,

``` bash
CARGO_TARGET_DIR=./target/release/maturin maturin build --release

🍹 Building a mixed python/rust project
🔗 Found pyo3 bindings with abi3 support
🐍 Found CPython 3.12 at /root/py312_qrmi_venv/bin/python
📡 Using build options features from pyproject.toml
   ...
   Compiling qrmi v0.7.1 (/shared/qrmi)
   Finished `release` profile [optimized] target(s) in 1m 10s
📦 Including files matching "python/qrmi/py.typed"
📦 Including files matching "python/qrmi/*.pyi"
📦 Built wheel for abi3 Python ≥ 3.12 to /shared/qrmi/target/release/maturin/wheels/qrmi-0.7.1-cp312-abi3-manylinux_2_34_aarch64.whl
```

Wheel is created under the `./target/release/maturin/wheels` directory.
You can distribute and install on your hosts using
`pip install <wheel>`.

``` bash
source ~/py312_qrmi_venv/bin/activate
pip install /shared/qrmi/target/release/maturin/wheels/qrmi-0.7.1-cp312-abi3-manylinux_2_34_aarch64.whl
```

#### Installing Lua Bindings

For RHEL / Clone OS installs (Rocky Linux 9, AlmaLinux 8), you need to
enable the additional repository (CRB or PowerTools) to install the
development packages (`-devel`).

##### Rocky Linux 9

``` bash
sudo dnf config-manager --set-enabled crb
sudo dnf install lua lua-devel
```

##### AlmaLinux 8

``` bash
sudo dnf config-manager --set-enabled powertools
sudo dnf install lua lua-devel
```

##### Debian / Ubuntu

On Debian and Ubuntu, development packages use the `-dev` suffix instead
of `-devel`. You can specify the Lua version (e.g., `5.4`) during
installation.

``` bash
sudo apt update
sudo apt install lua5.4 liblua5.4-dev
```

> [!note]
> You can replace `5.4` with other versions like `5.3` or `5.1`
> depending on your requirements.

#### Building Lua Bindings

Once installed, the Lua binding can be built using either gcc or cmake:

##### gcc

Assuming the `qrmi.h` and `libqrmi.so` live in the same directory
(`<QRMI_ROOT>`, e.g. `/path/to/qrmi`):

``` bash
gcc -shared -fPIC -O2 $(pkg-config --cflags lua5.4) \
   -I/path/to/qrmi \
   -o qrmi.so lua_qrmi.c \
   -L/path/to/qrmi -lqrmi \
   $(pkg-config --libs lua5.4) \
   -Wl,-rpath,/path/to/qrmi
```

- `-I/path/to/qrmi` --- lets the compiler find `qrmi.h`
- `-L/path/to/qrmi -lqrmi` --- links against `libqrmi.so`
- `-Wl,-rpath,/path/to/qrmi` --- bakes that directory into `qrmi.so`'s RUNPATH, so `LD_LIBRARY_PATH` doesn't need to be set at runtime (verify with `readelf -d qrmi.so`)

##### cmake

A `CMakeLists.txt` is included. It always links against the
`libqrmi.so`.

Expected layout (header and library in the same directory):

``` shell-session
<QRMI_ROOT>/qrmi.h
<QRMI_ROOT>/libqrmi.so
```

Create a build directory inside the Lua bindings directory and run
cmake:

``` bash
cd lua/
mkdir build && cd build
cmake -DQRMI_ROOT=/path/to/qrmi/install ..
cmake --build .
```

To specify paths individually:

``` bash
cmake -DQRMI_INCLUDE_DIR=/path/to/include -DQRMI_LIBRARY=/path/to/libqrmi.so ..
```

The directory containing `libqrmi.so` is automatically baked into the
built artifact's RUNPATH, so there's no need to set `LD_LIBRARY_PATH` at runtime (verifiable with `readelf -d qrmi.so`).

### Building Optional Libraries

There are optional packages available to install within the QRMI
repository.

#### Building `task_runner`

`task_runner` is a command line tool to execute quantum payloads against quantum hardware. Under the hood, it uses the QRMI library.

##### Running with Python

`task_runner` for Python is already included in the QRMI Python package. Users can use the `task_runner` command after installing qrmi. For detailed instructions on how to use it, please refer to the
[`task_runner` README](python/qrmi/tools/task_runner/README.md).

##### Build with Munge support for Pasqal Local

By default, QRMI is built without Munge support. If you need to use the Pasqal Local client which relies on Munge for authentication, you must enable the `munge` feature during the build process.

1. Build the Rust library:

``` bash
. ~/.cargo/env
cargo build --release --features munge
```

1. Build the Python wheels:

``` bash
source ~/py312_qrmi_venv/bin/activate
CARGO_TARGET_DIR=./target/release/maturin maturin build --release --features munge,pyo3/abi3,qrmi/pyo3
```

## Further Resources

### Examples

A [range of example code](examples) is available within this knowledge base. Each example provides detailed instructions and a link to the directory's GitHub location. You can find links to language-specific examples below:

- [`rust_examples`](examples/qrmi/rust)
- [`python_examples`](examples/qrmi/python)
- [`c_examples`](examples/qrmi/c)
- [`lua_examples`](examples/qrmi/lua)

QRMI is **workload manager agnostic** and supports a range of workload managers via plugins. One example of QRMI usage in a compute
infrastructure project is the Slurm plugin for quantum resources. QRMI is used in these Slurm plugins to control quantum resources during the lifecycle of a Slurm job. You can find full details on implementing the Quantum SPANK plugins for Slurm [here](https://github.com/qiskit-community/spank-plugins).

The Slurm plugin for quantum resources is only one example of QRMI's
workload manager integrations. More information about QRMI's
integrations is available in our
[Quantum-HPC Integration paper](https://arxiv.org/abs/2607.19591).

### Logging

QRMI supports [log crate](https://crates.io/crates/log) for logging. You can find the detailed QRMI runtime logs by specifying the `RUST_LOG` environment variable with log level. Supported levels are `error`, `warn`, `info`, `debug` and `trace`. The default level is `warn`.

If you specify `trace`, you can find underlying HTTP transaction logs.

``` bash
RUST_LOG=trace <YOUR QRMI EXECUTABLE>
```

``` bash
[2025-08-16T03:47:38Z DEBUG reqwest::connect] starting new connection: https://iam.cloud.ibm.com/
[2025-08-16T03:47:38Z DEBUG direct_access_api::middleware::auth] current token ...
```

### API Documentation

#### How to generate Rust API documents

``` bash
. ~/.cargo/env
cargo doc --no-deps --open
```

##### How to generate Python API documents

**Prerequisites**: QRMI Python package is installed in your Python virtual environment (e.g. `~/py312_qrmi_venv`)

``` bash
source ~/py312_qrmi_venv/bin/activate
python -m pydoc -p 8290
Server ready at http://localhost:8290/
Server commands: [b]rowser, [q]uit
server> b
```

Open the following page in your browser.

``` bash
http://localhost:8290/qrmi.html
```

Quit server.

``` bash
server> q
```

##### How to generate C API documents

Generating API document:

``` bash
doxygen Doxyfile
```

HTML document will be created under `./html` directory. Open `html/index.html` in your web browser.

### Packaging

#### How to package libqrmi for RHEL 8 based Linux

``` bash
git clone https://github.com/qiskit-community/qrmi.git
cd qrmi
./run_in_container.sh make libqrmi-
```

The packages below will be created under `./rpmbuild/RPMS/`:

- `libqrmi` --- runtime shared library (libqrmi.so.0), needed at runtime
- `libqrmi-devel` --- header (qrmi.h) + unversioned symlink (libqrmi.so), needed only when compiling applications against the library

> [!note]
> The command above will automatically create the required source and
> vendor tarballs. If you prefer, you could rather download them
> from the latest QRMI release published , renaming them accordingly
> to `./qrmi-<version>.tar.gz` and `./qrmi-<version>-vendor.tar.gz`.

#### How to install the libqrmi RPMs in a RHEL 8 based Linux system

``` bash
dnf install ./libqrmi-*.rpm
```

### Contributing

Whether you are part of the core team or an external contributor,
welcome and thank you for contributing to QRMI implementations!

You can learn more about contributing to the development of QRMI using our [Contribution Guidelines](https://qiskit-community.github.io/qrmi/development/CONTRIBUTING.html).

### Linting/Formatting

Contributors must execute the commands below and fix any issues before submitting a pull request.

#### Rust Formatting

``` bash
. ~/.cargo/env
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cd examples/rust
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

#### Python Formatting

``` bash
source ~/py312_qrmi_venv/bin/activate
cd examples
pylint ./python
black --check ./python
```

### Help and Support

If you require support, our [help and support documentation](https://qiskit-community.github.io/qrmi/getting_started/HELP_AND_SUPPORT.html)
provides up-to-date links to further guidance and our communication channels. Please don't hesitate to get in touch!

## License

[Apache-2.0](https://github.com/qiskit-community/qrmi/blob/main/LICENSE.txt)
