.. _provider_rust:

QRMI Provider - Examples in Rust
================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/resource_providers

Prerequisites
-------------

-  Python 3.11 or 3.12
-  Build the :ref:`QRMI Rust library <install_source>`


How to build this example
-------------------------

.. code-block:: shell-session

   cargo clean
   cargo build --release


How to run this example
-----------------------

.. code-block:: shell-session

   QRMI Provider Example

   Usage: qrmi-example-provider [OPTIONS] <CONFIG_FILE> <RESOURCE_NAME>

   Arguments:
     <CONFIG_FILE>    Path to qrmi_config.json
     <RESOURCE_NAME>  Name of the dynamic resource definition (is_dynamic=true)

   Options:
     -f, --filters <FILTERS>  Optional filter string e.g. "num_qubits=127&name=ibm_*"
     -h, --help               Print help
     -V, --version            Print version

For example:

.. code-block:: shell-session

   ../target/release/qrmi-example-provider /etc/slurm/qrmi_config.json ibm_inst1 -f "num_qubits=127&max_shots=10000"
   Filters: num_qubits=127&max_shots=10000

   Available resources (3 found):
   ----------------------------------------
     ibm_fez                        type=qiskit-runtime-service    accessible=true
     ibm_marrakesh                  type=qiskit-runtime-service    accessible=true
     ibm_kingston                   type=qiskit-runtime-service    accessible=true

   Least busy resource:
     ibm_fez
