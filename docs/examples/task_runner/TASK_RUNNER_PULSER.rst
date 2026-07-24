.. _task_runner_pulser:

Tools to Generate Input for Task Runner from Pulser
===================================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/pulser


Prerequisites
-------------

-  Python 3.11 or above


Install dependencies
--------------------

.. code-block:: shell-session

   pip install -f requirements.txt


Tools
-----

task_runner_input.py
~~~~~~~~~~~~~~~~~~~~

Generates input file in the correct format using `Pulser`_.

.. _Pulser: https://github.com/pasqal-io/pulser

Example:

.. code-block:: bash

   python task_runner_input.py

Output: ``sequence.json`` will be created.
