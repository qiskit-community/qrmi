Tools to Generate Input for Task Runner from Qiskit Pasqal Provider
===================================================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/qpp

Prerequisites
-------------

-  Python 3.11 or above

Install dependencies
--------------------

.. code:: shell-session

   pip install -f requirements.txt

Tools
-----

task_runner_input.py
~~~~~~~~~~~~~~~~~~~~

Generates input file in the correct format using
`Pulser <https://github.com/qiskit-community/qiskit-pasqal-provider>`__

Example:

.. code:: bash

   python task_runner_input.py

Output: ``sequence.json`` will be created.
