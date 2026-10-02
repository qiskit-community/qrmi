.. _oqtopus_lua:

OQTOPUS QRMI - Examples in Lua
=================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/oqtopus

--------------

Prerequisites
-------------

-  :ref:`QRMI C library (libqrmi.so) <building_core_qrmi_libraries>`
-  :ref:`QRMI Lua Module (qrmi.so) <installing_lua_bindings>`

Setup
-----

.. code:: bash

   export LUA_CPATH="</path/to/qrmi.so-dir/>?.so;;"
   export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libqrmi.so-dir

Example:

.. code:: bash

   export LUA_CPATH="/shared/qrmi/lua/build/?.so;;"
   export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/shared/qrmi/target/release

Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below.

=============================================== ==============================================================
Environment variables                           Descriptions
=============================================== ==============================================================
{device_id}_QRMI_OQTOPUS_URL                    OQTOPUS Cloud API endpoint
{device_id}_QRMI_OQTOPUS_API_TOKEN              OQTOPUS Cloud API token
{device_id}_QRMI_OQTOPUS_PROXY                  Proxy(Optional)
{device_id}_QRMI_OQTOPUS_TIMEOUT                HTTP request timeout seconds. (e.g. 30.0)
{device_id}_QRMI_OQTOPUS_RETRY_BACKOFF_SECONDS  Exponential backoff base seconds.(e.g. 0.2)
{device_id}_QRMI_OQTOPUS_RETRY_STATUS_CODES     HTTP status codes treated as retryable. (e.g. 429,500,502,503)
{device_id}_QRMI_OQTOPUS_RETRY_METHODS          HTTP methods treated as retryable.(e.g. GET,POST)
=============================================== ==============================================================


Create OQTOPUS program input file as input
------------------------------------------

Refer to :ref:`this tool <task_runner_oqtopus>` to
generate. You can customise quantum circuits by editing the code.

How to run `this example`_
--------------------------

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/oqtopus

Run `example.lua`_:

.. _example.lua: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/oqtopus/example.lua

.. code:: bash

   lua example.lua <device_id> <QASM program file> <job_type('sampling','estimation', 'multi_manual' or 'sse')>

For example:

.. code:: bash

   export qulacs_QRMI_OQTOPUS_URL=https://demo-api.oqtopus.io
   export qulacs_QRMI_OQTOPUS_API_TOKEN=your api token
   export PY_BRIDGE_PATH=<path/to/liboqtopus_py_bridge.so>

   lua example.lua qulacs ../../../task_runner/oqtopus/bell_state.txt sampling
