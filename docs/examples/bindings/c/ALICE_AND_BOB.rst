Alice and Bob Felis - Examples in C
===================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/alice_bob_felis

Prerequisites
-------------

-  C compiler/linker, cmake and make
-  Build the `QRMI Rust library <../../../README.md>`__

Set environment variables
-------------------------

See the corresponding section in `the README for the Felis Python
example <../../python/alice_bob_felis/README.md#set-environment-variables>`__

Generate QIR Input file
-----------------------

See the corresponding section in `the README for the Felis Python
example <../../python/alice_bob_felis/README.md#generate-qir-input-file>`__

How to build this example
-------------------------

.. code-block:: shell-session

   $ mkdir build
   $ cd build
   $ cmake ..
   $ make

How to run this example
-----------------------

.. code-block:: shell-session

   ./felis <backend_name> <input file>
