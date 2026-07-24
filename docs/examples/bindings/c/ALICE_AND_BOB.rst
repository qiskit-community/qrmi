.. _alice_and_bob_felis_c:

Alice and Bob Felis - Examples in C
===================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/alice_bob_felis


Prerequisites
-------------

-  C compiler/linker, cmake and make
-  Build the :ref:`QRMI Rust library <install_source>`


Set environment variables
-------------------------

See the corresponding section in the :ref:`README for the Felis Python
example <alice_and_bob_felis_python_env>`.


Generate QIR Input file
-----------------------

See the corresponding section in the :ref:`README for the Felis Python
example <alice_and_bob_felis_python_qri>`.


How to build this example
-------------------------

.. code-block:: shell-session

   mkdir build
   cd build
   cmake ..
   make


How to run this example
-----------------------

.. code-block:: shell-session

   ./felis <backend_name> <input file>
