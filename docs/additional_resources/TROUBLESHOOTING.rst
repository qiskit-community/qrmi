.. _troubleshooting:

Troubleshooting
===============

.. rst-class:: lead

    Guidance on how to troubleshoot common issues when using QRMI.

.. contents::
   :local:
   :depth: 2

Job Execution Errors
--------------------

``error: spank_qrmi_c, failed to acquire resource: ibm_brisbane``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cause:** This error occurs when accessing IBM Quantum backends using
an Open Plan account on IBM Quantum Platform.

**Check:**

#. Setup virtual environment:

.. code-block:: bash

   python3.11 -m venv ~/{your_pyenv}
   source ~/{your_pyenv}/bin/activate
   pip install qiskit-ibm-runtime

#. Create ``test.py``:

Replace ``SERVICE_CRN`` and ``API_KEY`` values with your credentials and with your
backend name.

.. code-block:: python
   :caption: test.py
   :linenos:

   """A testcase to check if Qiskit Session can be created with the given credentials"""
   from qiskit_ibm_runtime import QiskitRuntimeService, Session

   SERVICE_CRN="YOUR_SERVICE_CRN"
   API_KEY="YOUR_APIKEY" # pragma: allowlist secret

   service = QiskitRuntimeService(
       channel="ibm_cloud",
       instance=SERVICE_CRN,
       token=API_KEY,
   )

   backend = service.backend("<your backend name>")
   with Session(backend=backend, max_time=1) as session:
       print("Succeeded in obtaining a Qiskit Session")

#. Run the test case:

.. code-block:: bash

   python test.py

This will fail due to the error:

.. code-block:: bash

   You are not authorized to run a session when using the open plan.

**Solution:**

-  Use a Premium Plan account, or
-  Use `Batch`_ execution mode

.. _Batch: https://quantum.cloud.ibm.com/docs/en/guides/execution-modes#batch-mode

   -  Add ``QRMI_IBM_QRS_SESSION_MODE`` environment variable with
      “batch” in your ``qrmi_config.json``

.. code-block:: json

       {
         "name": "ibm_brisbane",
         "type": "qiskit-runtime-service",
         "environment": {
             "...": "...", 
             "QRMI_IBM_QRS_SESSION_MODE": "batch"
         }
      }
