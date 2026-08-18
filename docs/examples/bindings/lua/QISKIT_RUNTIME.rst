.. _qiskit_runtime_lua:


Qiskit Runtime Service QRMI - Examples in Lua
=============================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/ibm


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

+--------------------------------------------------+--------------------------------------------------+
|              Environment variables               |                   Descriptions                   |
+==================================================+==================================================+
| ``{resource_name}_QRMI_IBM_QRS_ENDPOINT``        | Qiskit Runtime Service endpoint                  |
|                                                  | URL (e.g. ``https://quantum.cloud.ibm.com/api``) |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_IAM_ENDPOINT``    | IBM Cloud IAM endpoint                           |
|                                                  | URL (e.g. ``https://iam.cloud.ibm.com``)         |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_IAM_APIKEY``      | IBM Cloud IAM API Key                            |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_SERVICE_CRN``     | Cloud Resource Name (CRN) of the                 |
|                                                  | provisioned Qiskit Runtime                       |
|                                                  | Service instance, starting with                  |
|                                                  | ``crn:v1:``.                                     |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_SESSION_MODE``    | Execution mode to run the session                |
|                                                  | in, ``default='dedicated'``,                     |
|                                                  | ``batch`` or ``dedicated``.                      |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_SESSION_MAX_TTL`` | The maximum time (in seconds) for                |
|                                                  | the session to run, subject to                   |
|                                                  | plan limits, default: ``28800``.                 |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_TIMEOUT_SECONDS`` | (Optional) Cost of the job as the                |
|                                                  | estimated time it should take to                 |
|                                                  | complete (in seconds). Should not                |
|                                                  | exceed the cost of the program,                  |
|                                                  | default: ``None``.                               |
+--------------------------------------------------+--------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QRS_SESSION_ID``      | (Optional) Session ID, can be                    |
|                                                  | obtanied by acquire function. If                 |
|                                                  | exists, used in the target                       |
|                                                  | functions.                                       |
+--------------------------------------------------+--------------------------------------------------+


Create Qiskit Primitive input file as input
-------------------------------------------

Refer to :ref:`this tool <task_runner_qiskit>` to
generate. You can customize quantum circuits by editing the code.

.. note::

   Use the file with name ending ``_params_only.json``,
   e.g. ``sampler_input_ibm_torino_params_only.json``.


How to build `this example`_
----------------------------

.. _this  example: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/ibm

.. code:: shell-session

   $ mkdir build
   $ cd build
   $ cmake ..
   $ make


How to run `this example`_
--------------------------

Run `example.lua`_:

.. _example.lua: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/ibm/example.lua

.. code:: shell-session

   lua example.lua <backend_name> <resource_type> <program type> <input filename>

For example:

.. code:: shell-session

   export ibm_torino_QRMI_IBM_QRS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
   export ibm_torino_QRMI_IBM_QRS_IAM_ENDPOINT=https://iam.cloud.ibm.com
   export ibm_torino_QRMI_IBM_QRS_IAM_APIKEY=your_apikey
   export ibm_torino_QRMI_IBM_QRS_SERVICE_CRN=your_instance

   lua example.lua ibm_torino qiskit-runtime-service sampler ../../examples/task_runner/qiskit/sampler_input_ibm_torino_params_only.json
