Installation for Quantum Resource Management Interface (QRMI)
=============================================================

.. contents::
   :local:
   :depth: 2

Prerequisites
-------------

-  Compilation requires the following tools:

   -  `Rust compiler 1.91 or
      above <https://www.rust-lang.org/tools/install>`__
   -  A C compiler

      -  For example, GCC (gcc) on Linux and Clang (clang-tools-extra)
         for Rust unknown targets/cross compilations. QRMI is compatible
         with a compiler conforming to the C11 standard.

   -  make/cmake (make/cmake RPM for RHEL compatible OS)
   -  Python 3.11, 3.12 or 3.13 (For Python API)

      -  Libraries and header files needed for Python
         development(python3.1x-devel RPM for RHEL compatible OS)

         -  ``/usr/include/python3.1x``
         -  ``/usr/lib64/libpython3.1x.so``

-  Runtime requires the following tools:

   -  gcc (libgcc RPM for RHEL compatible OS)
   -  Python 3.11, 3.12 or 3.13 (for Python API)

      -  Libraries and header files needed for Python development
         (python3.1x-devel RPM for RHEL compatible OS)

-  Doxygen (for generating C API document)

   -  ``dnf install doxygen`` for Linux(RHEL/CentOS/Rocky Linux etc)
   -  ``apt install doxygen`` for Linux(Ubuntu etc.)
   -  ``brew install doxygen`` for MacOS

Building Core QRMI Libraries
----------------------------

Core QRMI is a set of libraries to control the state of quantum
resources. It is written in Rust with C and Python APIs exposed for ease
of integration into any compute infrastructure.

This section shows how to build QRMI for C and Python.

How to Build Rust/C API Library
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: shell-session

   . ~/.cargo/env
   cargo clean
   cargo build --locked --release

How to Build & Install QRMI Python Package
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Setup Python virtual environment
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code:: shell-session

   . ~/.cargo/env
   cargo clean
   python3.12 -m venv ~/py312_qrmi_venv
   source ~/py312_qrmi_venv/bin/activate
   pip install --upgrade pip
   pip install -r requirements-dev.txt

Create stub file for python code
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code:: shell-session

   . ~/.cargo/env
   cargo run --bin stubgen --features=pyo3

Create a wheel for distribution
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code:: shell-session

   source ~/py312_qrmi_venv/bin/activate
   CARGO_TARGET_DIR=./target/release/maturin maturin build --release

For example,

.. code:: shell-session

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

Wheel is created under ``./target/release/maturin/wheels`` directory.
You can distribute and install on your hosts by ``pip install <wheel>``.

.. code:: shell-session

   source ~/py312_qrmi_venv/bin/activate
   pip install /shared/qrmi/target/release/maturin/wheels/wheels/qrmi-0.7.1-cp312-abi3-manylinux_2_34_aarch64.whl

Building optional libraries
---------------------------

Optional packages that are available in QRMI repository.

``task_runner`` is command line command to execute quantum payload
againts quantum hardware. Under the hood it is using QRMI library.

How to build task_runner for Rust version
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

   [!WARNING] Rust version of task_runner is now obsoleted. Use Python
   version.

.. code:: shell-session

   . ~/.cargo/env
   cargo build --bin task_runner --release --features=build-binary

How to run task_runner for Python version
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``task_runner`` for Python version is already included in QRMI Python
package. User can use task_runner command after installing qrmi. For
detailed instructions on how to use it, please refer to this
`README <./python/qrmi/tools/task_runner/README.md>`__.

How to build with Munge support for Pasqal Local
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

By default, QRMI is built without Munge support. If you need to use the
Pasqal Local client which relies on Munge for authentication, you must
enable the ``munge`` feature during the build process.

Build the rust library:

.. code:: shell-session

   . ~/.cargo/env
   cargo build --release --features munge

Build the python wheels:

.. code:: shell-session

   source ~/py312_qrmi_venv/bin/activate
   CARGO_TARGET_DIR=./target/release/maturin maturin build --release --features pyo3/extension-module,munge,pyo3/abi3,qrmi/pyo3

Other
-----

Examples
~~~~~~~~

-  `Examples in Rust <./examples/qrmi/rust>`__
-  `Examples in Python <./examples/qrmi/python>`__
-  `Examples in C <./examples/qrmi/c>`__

Logging
~~~~~~~

QRMI supports `log crate <https://crates.io/crates/log>`__ for logging.
You can find the detailed QRMI runtime logs by specifying ``RUST_LOG``
environment variable with log level. Supported levels are ``error``,
``warn``, ``info``, ``debug`` and ``trace``. Default level is ``warn``.

If you specify ``trace``, you can find underlying HTTP transaction logs.

.. code:: shell-session

   RUST_LOG=trace <your QRMI executable>

.. code:: shell-session

   [2025-08-16T03:47:38Z DEBUG reqwest::connect] starting new connection: https://iam.cloud.ibm.com/
   [2025-08-16T03:47:38Z DEBUG direct_access_api::middleware::auth] current token ...

API Docs
~~~~~~~~

How to generate Rust API document
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code:: shell-session

   . ~/.cargo/env
   cargo doc --no-deps --open

How to generate Python API document
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Prerequisites: QRMI Python package is installed in your python virtual
environment(e.g. ``~/py312_qrmi_venv``)

.. code:: shell-session

   source ~/py312_qrmi_venv/bin/activate
   python -m pydoc -p 8290
   Server ready at http://localhost:8290/
   Server commands: [b]rowser, [q]uit
   server> b

Open the following page in your browser.

.. code:: shell-session

   http://localhost:8290/qrmi.html 

Quit server.

.. code:: shell-session

   server> q

How to generate C API document
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Generating API document
'''''''''''''''''''''''

.. code:: shell-session

   doxygen Doxyfile

HTML document will be created under ``./html`` directory. Open
``html/index.html`` in your web browser.

Contributing
~~~~~~~~~~~~

Regardless if you are part of the core team or an external contributor,
welcome and thank you for contributing to QRMI implementations!

Solving linting/format issues
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Contributor must execute the commands below and fix any issues before
submitting Pull Request.

Rust code
^^^^^^^^^

.. code:: shell-session

   $ . ~/.cargo/env
   $ cargo fmt --all -- --check
   $ cargo clippy --all-targets -- -D warnings
   $ cd examples/rust
   $ cargo fmt --all -- --check
   $ cargo clippy --all-targets -- -D warnings

Python code
^^^^^^^^^^^

.. code:: shell-session

   $ source ~/py312_qrmi_venv/bin/activate
   $ cd examples
   $ pylint ./python
   $ black --check ./python

License
-------

`Apache-2.0 <https://github.com/qiskit-community/spank-plugins/blob/main/qrmi/LICENSE.txt>`__
