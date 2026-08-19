.. _iqm_server_lua:

IQM Server QRMI - Examples in Lua
=================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/iqm


Prerequisites
-------------

-  :ref:`QRMI C library (libqrmi.so) <building_core_qrmi_libraries>`
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

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below.

========================================= =======================
Environment variables                     Descriptions
========================================= =======================
``{qc_alias_name}_QRMI_IQM_ISA_ENDPOINT`` IQM Server API endpoint
``{qc_alias_name}_QRMI_IBM_ISA_TOKEN``    IQM Server API token
========================================= =======================

.. note::

   Replace the ":" in the ``{qc_alias_name}`` with "\_" when
   specifying it. For example, ``sirius:mock`` -> ``sirius_mock``.


Create IQM JSON input file as input
-----------------------------------

Refer to :ref:`this tool <task_runner_iqm>` to generate. You can
customise quantum circuits by editting the code.

.. note::

   Use the file with name ending ``_params_only.json``,
   e.g. ``iqm_json_sirius_params_only.json``.

How to run `this example`_
--------------------------

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/iqm

Run `example.lua`_:

.. _example.lua: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/iqm/example.lua

.. code:: shell-session

   lua example.lua <qc_alias> <IQM JSON> <job_type('circuit','run' or 'sweep')>

For example:

.. code:: shell-session

   export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
   export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

   lua example.lua garnet_mock ../../../task_runner/iqm/iqm_json_garnet\:mock.json circuit
