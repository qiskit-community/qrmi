.. _alice_and_bob_felis_rust:

Alice Bob Felis QRMI - Examples in Rust
=======================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/alice_bob_felis


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

   cargo clean
   CARGO_TARGET_DIR=./target cargo build --release


How to run this example
-----------------------

.. code-block:: shell-session

   qrmi-example-alice-bob-felis --backend <BACKEND> --input <INPUT>

For example:

.. code-block:: shell-session

    ./target/debug/qrmi-example-alice-bob-felis --backend 'ab_emu_1q_lescanne_2020' --input ./generated_circuit.ll
