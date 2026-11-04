.. _task_runner_oqtopus:

Tools to Generate OQTOPUS program input from Qiskit QuantumCircuit
==================================================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/oqtopus

--------------

The tools demonstrate the generation of OQTOPUS program input from a quantum
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

Genetates OQTOPUS program input from Bell-state QuantumCircuit.

Usage:

.. code-block:: bash

   usage: gen_sampling_inputs.py [-h] output


   positional arguments:
     output      output file

   options:
     -h, --help  show this help message and exit

Example:

.. code-block:: bash

   python gen_sampling_inputs.py bell_state.txt
