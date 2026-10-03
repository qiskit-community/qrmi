.. _task_runner_oqtopus:

Tools to Generate OQTOPUS job spec JSON input from Qiskit QuantumCircuit
========================================================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/oqtopus

--------------

The tools demonstrate the generation of OQTOPUS job spec JSON input from a quantum
circuit example.


Prerequisites
-------------

-  Python 3.11 or above


Install dependencies
--------------------

.. code-block:: bash

   pip install -f requirements.txt


Tools
-----

`gen_sampling_inputs.py`_
~~~~~~~~~~~~~~~~~~~~~~~

.. _gen_sampling_inputs.py: https://github.com/qiskit-community/qrmi/blob/main/examples/task_runner/oqtopus/gen_sampling_inputs.py

Genetates OQTOPUS job spec JSON input from Bell-state QuantumCircuit.

Usage:

.. code-block:: bash

   usage: gen_sampling_inputs.py [-h] device_id

   A tool to generate OQTOPUS job spec JSON input from Bell-state QuantumCircuit

   positional arguments:
     device_id   device ID

   options:
     -h, --help  show this help message and exit

Example:

.. code-block:: bash

   python gen_sampling_inputs.py qulacs
