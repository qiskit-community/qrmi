Qiskit Primitives with Pasqal Cloud QRMI - Python Example
=========================================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/qiskit_primitives/pasqal

--------------

Prerequisites
-------------

-  Python 3.11 or 3.12
-  :ref:`Installation of QRMI primitives Python package (qiskit-qrmi-primitives) <qiskit_examples>`

Install dependencies
--------------------

Assuming your python virtual environment is located at
``~/py311venv_qrmi_primitives/bin/activate``,

.. code-block:: shell-session

   $ source ~/py311venv_qrmi_primitives/bin/activate
   $ pip install -r requirements.txt

Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. `This example`_ assumes
that a ``.env`` file is available under the current directory.

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qiskit_primitives/pasqal

Common
~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by the SPANK plugin.

+----------------------------+-----------------------------------------+
|   Environment variables    |              Descriptions               |
+============================+=========================================+
| ``QRMI_JOB_QPU_RESOURCES`` | Quantum resource names. Comma-separated |
|                            | values, e.g. ``FRESNEL``                |
+----------------------------+-----------------------------------------+
| ``QRMI_JOB_QPU_TYPES``     | Quantum resource types. Comma-separated |
|                            | values corresponding to each Quantum    |
|                            | resource name specified by              |
|                            | ``QRMI_JOB_QPU_RESOURCES``.             |
|                            | Supported types: ``pasqal-cloud``       |
+----------------------------+-----------------------------------------+

How to run `this example`_
--------------------------

SamplerV2
~~~~~~~~~

Use the Qiskit Pasqal Provider ``SamplerV2``.

`This example`_ wraps a QRMI backend with the Qiskit Pasqal Provider
``SamplerV2``.

Execution returns a job object and Qiskit-style result object:

.. code-block:: python
   :linenos:

   job = sampler.run([qc], shots=100)
   result = job.result()
   print(result[0].data.counts)

QRMI run options can be passed through backend options: -
``poll_interval_seconds`` - ``timeout_seconds`` - ``delete_job``

Example:

.. code-block:: python
   :linenos:

   backend = QRMIPasqalBackend(
       qrmi=qrmi,
       options={"run_options": {"poll_interval_seconds": 1.0}},
   )
   sampler = SamplerV2(backend)

For emulator resources where device specs are not exposed, QRMI falls
back to Pulser ``DigitalAnalogDevice``.

Run `sampler.py`_:

.. _sampler.py: https://github.com/qiskit-community/qrmi/blob/main/examples/qiskit_primitives/pasqal/sampler.py

.. code-block:: shell-session

   $ python sampler.py
