Parsing QRMI Config File in C
=============================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/config

Prerequisites
-------------

-  C compiler/linker, cmake and make
-  `QRMI Rust library <../../../../README.md>`__

How to build this example
-------------------------

.. code:: shell-session

   $ mkdir build
   $ cd build
   $ cmake ..
   $ make

How to run this example
-----------------------

.. code:: shell-session

   $ ./build/
   qrmi_config <qrmi_config.json file> <resource name>

For example,

.. code:: shell-session

   ./build/qrmi_config /etc/slurm/qrmi_config.json ibm_fez
