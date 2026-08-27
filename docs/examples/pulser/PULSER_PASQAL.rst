.. _pulser_pasqal:

Pulser Connection with Pasqal Cloud QRMI - Python Example
=========================================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/pulser/pasqal

--------------

Prerequisites
-------------

-  Python 3.11 or 3.12
-  :ref:`Installation of QRMI primitives Python package (qiskit-qrmi-primitives) <qiskit_examples>`


Install dependencies
--------------------

Assuming your Python virtual environment is located at
``~/py311venv_qrmi_primitives/bin/activate``:

.. code-block:: shell-session

   source ~/py311venv_qrmi_primitives/bin/activate
   pip install -r requirements.txt


Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. `This example`_ assumes
that a ``.env`` file is available under the current directory.

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/pulser/pasqal


Common
~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by the SPANK plugin.

+----------------------------+-----------------------------------+
|   Environment variables    |           Descriptions            |
+============================+===================================+
| ``QRMI_JOB_QPU_RESOURCES`` | Quantum resource names.           |
|                            | Comma-separated values,           |
|                            | e.g. ``FRESNEL``                  |
+----------------------------+-----------------------------------+
| ``QRMI_JOB_QPU_TYPES``     | Quantum resource types.           |
|                            | Comma-separated values            |
|                            | corresponding to each Quantum     |
|                            | resource name specified by        |
|                            | ``QRMI_JOB_QPU_RESOURCES``.       |
|                            | Supported types: ``pasqal-cloud`` |
+----------------------------+-----------------------------------+


How to run `this example`_
--------------------------

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/pulser/pasqal

SamplerV2
~~~~~~~~~

Use Pulser's ``QPUBackend`` with ``PulserQRMIConnection``.

Run `pulser_qrmi.py`_:

.. _pulser_qrmi.py: https://github.com/qiskit-community/qrmi/blob/main/examples/pulser/pasqal/pulser_qrmi.py

.. code-block:: shell-session

   python pulser_qrmi.py
