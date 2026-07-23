QRMI Provider - Examples in Python
==================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/python/resource_providers

Prerequisites
-------------

-  Rust 1.85.1 or above
-  Python 3.11 or 3.12
-  `QRMI python package installation <../../../../README.md>`__

Install dependencies
--------------------

.. code-block:: shell-session

   $ source ~/py311_qrmi_venv/bin/activate
   $ pip install qrmi[ibm]

How to run
----------

.. code-block:: shell-session

   usage: example.py [-h] [--filters FILTERS] config_file resource_name

   Unified QRMI Provider Example

   positional arguments:
     config_file        Path to qrmi_config.json
     resource_name      Name of the dynamic resource definition (is_dynamic=true)

   options:
     -h, --help         show this help message and exit
     --filters FILTERS  Optional filter string e.g. 'num_qubits=127&name=ibm_*'

For example,

.. code-block:: shell-session

   python example.py /etc/slurm/qrmi_config.json ibm_inst1 --filters "num_qubits=127&max_shots=10000"
