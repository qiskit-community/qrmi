Parsing QRMI config file in Rust
================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/qrmi_config

Prerequisites
-------------

-  Python 3.11 or 3.12
-  `QRMI Rust library <../../../README.md>`__

How to build this example
-------------------------

.. code:: shell-session

   $ cargo clean
   $ cargo build --release

How to run this example
-----------------------

.. code:: shell-session

   $ ../target/release/qrmi-example-config --help
   Parsing qrmi_config.json file

   Usage: qrmi-example-config --file <FILE>

   Options:
     -f, --file <FILE>  qrmi_config.json file
     -h, --help         Print help
     -V, --version      Print version

For example,

.. code:: shell-session

   ../target/release/qrmi-example-config -f /etc/slurm/qrmi_config.json
