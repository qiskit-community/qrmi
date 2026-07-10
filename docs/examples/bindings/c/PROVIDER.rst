QRMI Provider Example in C
==========================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/resource_providers

A unified example that works with any supported provider type
(``qiskit-runtime-service``, ``ibm-quantum-system``, etc.). The resource
type is read from ``qrmi_config.json`` — no code changes needed when
switching between providers.

Prerequisites
-------------

-  C compiler/linker, cmake and make
-  `QRMI Rust library <../../../README.md>`__

Config file
-----------

Create a ``qrmi_config.json`` with an ``is_dynamic: true`` entry:

.. code:: json

   {
       "resources": [
           {
               "name": "ibm_inst1",
               "type": "qiskit-runtime-service",
               "is_dynamic": true,
               "environment": {
                   "QRMI_IBM_QRS_ENDPOINT":     "https://quantum.cloud.ibm.com/api/v1",
                   "QRMI_IBM_QRS_IAM_ENDPOINT": "https://iam.cloud.ibm.com",
                   "QRMI_IBM_QRS_IAM_APIKEY":   "<your_api_key>",
                   "QRMI_IBM_QRS_SERVICE_CRN":  "<your_service_crn>"
               }
           }
       ]
   }

How to build
------------

.. code:: shell-session

   $ mkdir build
   $ cd build
   $ cmake ..
   $ make

How to run
----------

.. code:: shell-session

   # No filter
   ./build/providers /path/to/qrmi_config.json ibm_inst1

   # With filter
   ./build/providers /path/to/qrmi_config.json ibm_inst1 "num_qubits=127&name=ibm_*"
