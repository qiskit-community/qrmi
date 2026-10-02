.. _oqtopus_c:

OQTOPUS QRMI - Examples in C
===============================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/oqtopus

--------------

Prerequisites
-------------

-  :ref:`QRMI Developer Prerequisites <install_source>`


Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. `This example`_ assumes
that a ``.env`` file is available under the current directory.

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/oqtopus

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


How to build `this example`_
----------------------------

.. code-block:: bash

   mkdir build
   cd build
   cmake ..
   make


How to run `this example`_
--------------------------

.. code-block:: bash

   ./build/oqtopus
   oqtopus <device_id> <QASM program file> <job_type('sampling','estimation', 'multi_manual' or 'sse')>

For example:

.. code-block:: bash

   # .env
   qulacs_QRMI_OQTOPUS_URL=https://demo-api.oqtopus.io
   qulacs_QRMI_OQTOPUS_API_TOKEN=your api token
   PY_BRIDGE_PATH=<path/to/liboqtopus_py_bridge.so>

   ./build/oqtopus qulacs ../../../../task_runner/oqtopus/bell_state.txt sampling
