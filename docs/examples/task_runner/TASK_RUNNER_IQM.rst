Tools to Generate IQM JSON from Qiskit QuantumCircuit
=====================================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/iqm

The tools demonstrate the generation of IQM JSON input from a quantum
circuit example.

Prerequisites
-------------

-  Python 3.11 or above

Install dependencies
--------------------

.. code-block:: shell-session

   pip install -f requirements.txt

Tools
-----

gen_iqm_json.py
~~~~~~~~~~~~~~~

Generates IQM JSON input for the circuit introduced in Starter notebook
provided by IQM.

Usage:

.. code-block:: shell-session

   usage: gen_iqm_json.py [-h] qc_alias base_url token

   A tool to generate IQM JSON from sample QuantumCircuit

   positional arguments:
     qc_alias    QC alias(e.g. sirius:mock)
     base_url    IQM Server API endpoint
     token       IQM Server API token

   options:
     -h, --help  show this help message and exit

Example:

.. code-block:: bash

   python gen_iqm_json.py sirius:mock https://resonance.meetiqm.com <your API token>

Output:

+---------------------------------------+---------------------------------------+
| Files                                 | Descriptions                          |
+=======================================+=======================================+
| iqm_json\_{qc_alias}_params_only.json | IQM JSON input.                       |
+---------------------------------------+---------------------------------------+
| iqm_json\_{qc_alias}.json             | An input for QRMI task runner,        |
|                                       | which contains additional             |
|                                       | properties - ``job_type``,            |
|                                       | ``use_timeslot`` and ``tag``.         |
+---------------------------------------+---------------------------------------+
