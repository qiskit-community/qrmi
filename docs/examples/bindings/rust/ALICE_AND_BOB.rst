Alice Bob Felis QRMI - Examples in Rust
=======================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/alice_bob_felis

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

.. code:: shell-session

   cargo clean
   CARGO_TARGET_DIR=./target cargo build --release

How to run this example
-----------------------

.. code:: shell-session

   qrmi-example-alice-bob-felis --backend <BACKEND> --input <INPUT>

For example,

.. code:: shell-session

    ./target/debug/qrmi-example-alice-bob-felis --backend 'ab_emu_1q_lescanne_2020' --input ./generated_circuit.ll
