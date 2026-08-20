.. _install:

Installing QRMI
===============

.. rst-class:: lead

    Learn how to install QRMI and explore information about further resources.

--------------

.. contents::
   :local:
   :depth: 2

--------------

Quick Start
-----------

We encourage installing QRMI via ``pip``:

.. code-block:: shell-session

   pip install qrmi

To use a specific quantum resource, install QRMI with the corresponding optional dependencies:

.. code-block:: shell-session 

   pip install "qrmi[ibm]"       # Include dependencies for IBM
   pip install "qrmi[iqm]"       # Include dependencies for IQM
   pip install "qrmi[pasqal]"    # Include dependencies for Pasqal
   pip install "qrmi[alice-bob]" # Include dependencies for Alice and Bob
   pip install "qrmi[all]"       # Include dependencies for all quantum resources except `alice-bob`

Or combine multiple resources:

.. code-block:: shell-session 

   pip install "qrmi[ibm,pasqal]"

.. note::

   ``ibm`` and ``iqm`` extras cannot be installed together, as they depend on incompatible versions of Qiskit.

.. note::

   ``alice-bob`` cannot be installed alongside ``ibm`` or ``iqm``, as it depends on Qiskit versions earlier than 2.0.

Pip will handle all dependencies automatically and you will always install the latest (and most thoroughly tested) version.


.. _install_source:

Installing from Source
----------------------

Prerequisites
~~~~~~~~~~~~~

-  Compilation requires the following tools:

   -  `Rust compiler 1.91 or above <https://www.rust-lang.org/tools/install>`__
   -  A C compiler

      -  For example, GCC (gcc) on Linux and Clang (clang-tools-extra)
         for Rust unknown targets/cross compilations. QRMI is compatible
         with a compiler conforming to the C11 standard.

   -  make/cmake (make/cmake RPM for RHEL compatible OS)
   -  Python 3.11, 3.12 or 3.13 (for Python API)

      -  Libraries and header files needed for Python
         development (python3.1x-devel RPM for RHEL compatible OS)

         -  ``/usr/include/python3.1x``
         -  ``/usr/lib64/libpython3.1x.so``

   - Standard Lua (PUC-Rio Lua 5.1-5.4)
   - :ref:`QRMI Standalone C library & header <building_core_qrmi_libraries>`

-  Runtime requires the following tools:

   -  gcc (libgcc RPM for RHEL compatible OS)
   -  Python 3.11, 3.12 or 3.13 (for Python API)

      -  Libraries and header files needed for Python development
         (python3.1x-devel RPM for RHEL compatible OS)

-  Doxygen (for generating C API document):

   -  ``dnf install doxygen`` for Linux(RHEL/CentOS/Rocky Linux etc.)
   -  ``apt install doxygen`` for Linux(Ubuntu etc.)
   -  ``brew install doxygen`` for MacOS

.. _building_core_qrmi_libraries:

Building Core QRMI Libraries
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Core QRMI is a set of libraries to control the state of quantum
resources. It is written in Rust with C and Python APIs exposed for ease
of integration into any compute infrastructure.

Prebuilt binaries for Linux (glibc 2.28 compatible) on x86_64, ppc64le, and aarch64
platforms are available for download from the repository's `Releases tab`_.

.. _Releases tab: https://github.com/qiskit-community/qrmi/releases/latest

This section will guide you through building QRMI for C, Python and Lua.

.. tabs::

   .. tab:: Rust/C

      .. code-block:: shell-session

         . ~/.cargo/env
         cargo clean
         cargo build --locked --release

   .. tab:: Python

      1. Setup a Python virtual environment

      .. code-block:: shell-session

         . ~/.cargo/env
         cargo clean
         python3.12 -m venv ~/py312_qrmi_venv
         source ~/py312_qrmi_venv/bin/activate
         pip install --upgrade pip
         pip install -r requirements-dev.txt

      2. Create stub file for Python code

      .. code-block:: shell-session

         . ~/.cargo/env
         cargo run --bin stubgen --features=pyo3

      3. Create a wheel for distribution

      .. code-block:: shell-session

         source ~/py312_qrmi_venv/bin/activate
         CARGO_TARGET_DIR=./target/release/maturin maturin build --release

      For example,

      .. code-block:: shell-session

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

      Wheel is created under the ``./target/release/maturin/wheels`` directory.
      You can distribute and install on your hosts using ``pip install <wheel>``.

      .. code-block:: shell-session

         source ~/py312_qrmi_venv/bin/activate
         pip install /shared/qrmi/target/release/maturin/wheels/qrmi-0.7.1-cp312-abi3-manylinux_2_34_aarch64.whl

.. _installing_lua_bindings:

Installing Lua Bindings
^^^^^^^^^^^^^^^^^^^^^^^

For RHEL / Clone OS installs (Rocky Linux 9, AlmaLinux 8), you need to enable the additional
repository (CRB or PowerTools) to install the development packages (``-devel``).

.. tabs::

   .. tab:: Rocky Linux 9

      .. code:: bash

         sudo dnf config-manager --set-enabled crb
         sudo dnf install lua lua-devel

   .. tab:: AlmaLinux 8

      .. code:: bash

         sudo dnf config-manager --set-enabled powertools
         sudo dnf install lua lua-devel

   .. tab:: Debian / Ubuntu

      On Debian and Ubuntu, development packages use the ``-dev`` suffix
      instead of ``-devel``. You can specify the Lua version (e.g., ``5.4``)
      during installation.

         .. code:: bash

            sudo apt update
            sudo apt install lua5.4 liblua5.4-dev

      .. note:: 

         You can replace ``5.4`` with other versions like ``5.3`` or
         ``5.1`` depending on your requirements.

Once installed, the Lua binding can be built using either gcc or cmake:

.. tabs::

   .. tab:: gcc

      Assuming the ``qrmi.h`` and ``libqrmi.so`` live in the same directory
      (``<QRMI_ROOT>``, e.g. ``/path/to/qrmi``):

      .. code:: bash

         gcc -shared -fPIC -O2 $(pkg-config --cflags lua5.4) \
            -I/path/to/qrmi \
            -o qrmi.so lua_qrmi.c \
            -L/path/to/qrmi -lqrmi \
            $(pkg-config --libs lua5.4) \
            -Wl,-rpath,/path/to/qrmi

      -  ``-I/path/to/qrmi`` — lets the compiler find ``qrmi.h``
      -  ``-L/path/to/qrmi -lqrmi`` — links against ``libqrmi.so``
      -  ``-Wl,-rpath,/path/to/qrmi`` — bakes that directory into
         ``qrmi.so``'s RUNPATH, so ``LD_LIBRARY_PATH`` doesn't need to be
         set at runtime (verify with ``readelf -d qrmi.so``)

   .. tab:: cmake

      A ``CMakeLists.txt`` is included. It always links against the
      ``libqrmi.so``.

      Expected layout (header and library in the same directory):

      ::

         <QRMI_ROOT>/qrmi.h
         <QRMI_ROOT>/libqrmi.so

      .. code:: bash

         mkdir build && cd build
         cmake -DQRMI_ROOT=/path/to/qrmi/install ..
         cmake --build .

      To specify paths individually:

      .. code:: bash

         cmake -DQRMI_INCLUDE_DIR=/path/to/include -DQRMI_LIBRARY=/path/to/libqrmi.so ..

      The directory containing ``libqrmi.so`` is automatically baked into the
      built artifact's RUNPATH, so there's no need to set ``LD_LIBRARY_PATH``
      at runtime (verifiable with ``readelf -d qrmi.so``).


Building Optional Libraries
~~~~~~~~~~~~~~~~~~~~~~~~~~~

There are optional packages available to install within the QRMI repository.


Building ``task_runner``
^^^^^^^^^^^^^^^^^^^^^^^^

:ref:`task_runner <task_runner>` is a command line tool to execute quantum payloads
against quantum hardware. Under the hood, it uses the QRMI library.

.. tabs::

   .. tab:: Python

      ``task_runner`` for Python is already included in the QRMI Python
      package. Users can use the  ``task_runner``` command after installing qrmi. For
      detailed instructions on how to use it, please refer to the  :ref:`task_runner README <task_runner>`.


Build with Munge support for Pasqal Local
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

By default, QRMI is built without Munge support. If you need to use the
Pasqal Local client which relies on Munge for authentication, you must
enable the ``munge`` feature during the build process.

#. Build the Rust library:

.. code-block:: shell-session

   . ~/.cargo/env
   cargo build --release --features munge

#. Build the Python wheels:

.. code-block:: shell-session

   source ~/py312_qrmi_venv/bin/activate
   CARGO_TARGET_DIR=./target/release/maturin maturin build --release --features pyo3/extension-module,munge,pyo3/abi3,qrmi/pyo3


Further Resources
-----------------

Examples
~~~~~~~~

A :ref:`range of example code <examples_index>` is available within this knowledge base. Each example provides detailed instructions and
a link to the directory's GitHub location. You can find links to language-specific examples below:

-  :ref:`rust_examples`
-  :ref:`python_examples`
-  :ref:`c_examples`
-  :ref:`lua_examples`

QRMI is **workload manager agnostic** and supports a range of workload managers via plugins. One example of QRMI usage in a compute 
infrastructure project is the Slurm plugin for quantum resources. QRMI is used in these Slurm plugins to control quantum resources 
during the lifecycle of a Slurm job. You can find full details on implementing the Quantum SPANK plugins for Slurm `here`_.

.. _here: https://github.com/qiskit-community/spank-plugins


The Slurm plugin for quantum resources is only one example of QRMI's workload manager integrations. More information about QRMI's 
integrations is available in our :ref:`Quantum-HPC Integration paper <qrmi_integrations>`.


Logging
~~~~~~~

QRMI supports `log crate`_ for logging.
You can find the detailed QRMI runtime logs by specifying the ``RUST_LOG``
environment variable with log level. Supported levels are ``error``,
``warn``, ``info``, ``debug`` and ``trace``. The default level is ``warn``.

.. _log crate: https://crates.io/crates/log

If you specify ``trace``, you can find underlying HTTP transaction logs.

.. code-block:: shell-session

   RUST_LOG=trace <YOUR QRMI EXECUTABLE>

.. code-block:: shell-session

   [2025-08-16T03:47:38Z DEBUG reqwest::connect] starting new connection: https://iam.cloud.ibm.com/
   [2025-08-16T03:47:38Z DEBUG direct_access_api::middleware::auth] current token ...


API Documentation
~~~~~~~~~~~~~~~~~

Links to the Python, Rust, C and Lua API documentation is available here:

-  :ref:`rust_api`
-  :ref:`python_api`
-  :ref:`c_api`
-  :ref:`lua_api`

The Python, Rust and C API documentation can be built locally using our :ref:`API documentation guide <api_docs>`.


Contributing
~~~~~~~~~~~~

Whether you are part of the core team or an external contributor,
welcome and thank you for contributing to QRMI implementations!

You can learn more about contributing to the development of QRMI using our :ref:`contribution guidance <contributing>`.


Linting/Formatting
~~~~~~~~~~~~~~~~~~

For guidance on how to solve linting/formatting issues, and executing the necessary checks
before submitting a PR, follow our :ref:`style and linting documentation <contributing_style>`.


Help and Support
~~~~~~~~~~~~~~~~

If you require support, our :ref:`help and support documentation <help_and_support>` provides up-to-date links to
further guidance and our communication channels. Please don't hesitate to get in touch!


License
-------

`Apache-2.0 <https://github.com/qiskit-community/qrmi/blob/main/LICENSE.txt>`__
