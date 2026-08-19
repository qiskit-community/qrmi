.. _alice_and_bob_felis_lua:

Alice and Bob Felis - Examples in Lua
=====================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/alice_bob_felis


Prerequisites
-------------

-  :ref:`QRMI C (libqrmilibrary.so) <building_core_qrmi_libraries>`
-  :ref:`QRMI Lua Module (qrmi.so) <installing_lua_bindings>`


Setup
-----

.. code:: shell-session

   export LUA_CPATH="</path/to/qrmi.so-dir/>?.so;;"
   export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libqrmi.so-dir

Example:

.. code:: shell-session

   export LUA_CPATH="/shared/qrmi/lua/build/?.so;;"
   export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/shared/qrmi/target/release


Set environment variables
-------------------------

See the corresponding section in :ref:`the Felis Python example <alice_and_bob_felis_python_env>`.


Generate QIR Input file
-----------------------

See the corresponding section in :ref:`the Felis Python example <alice_and_bob_felis_python_qri>`.


How to run `this example`_
--------------------------

.. _this example: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/alice_bob_felis

Run `example.lua`_:

.. _example.lua: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/alice_bob_felis/example.lua

.. code:: shell-session

   lua example.lua <backend name> <qir input file>
