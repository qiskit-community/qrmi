.. _qiskit_iqm_primitive:

IQM Qiskit - Python Example
===========================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qiskit_primitives/iqm


Prerequisites
-------------

-  Python 3.11 or 3.12
-  Install the :ref:`QRMI Python package <install_source>`


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
required environment variables are listed below. This example assumes
that a ``.env`` file is available under the current directory.


Common
~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by the SPANK plugin.

+-----------------------------------+-----------------------------------+
| Environment variables             | Descriptions                      |
+===================================+===================================+
| QRMI_JOB_QPU_RESOURCES            | Quantum resource names.           |
|                                   | Comma-separated values,           |
|                                   | e.g. ``garnet,emerald``           |
+-----------------------------------+-----------------------------------+
| QRMI_JOB_QPU_TYPES                | Quantum resource types. Specify   |
|                                   | ``iqm-server`` for this example   |
+-----------------------------------+-----------------------------------+


IQM Server specific
~~~~~~~~~~~~~~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by users or administrator.

===================================== =======================
Environment variables                 Descriptions
===================================== =======================
{qc_alias_name}_QRMI_IQM_ISA_ENDPOINT IQM Server API endpoint
{qc_alias_name}_QRMI_IQM_ISA_TOKEN    IQM Server API token
===================================== =======================

.. note::

   Replace the ":" in the QC alias name with "\_" when
   specifying it. For example, ``sirius:mock`` -> ``sirius_mock``.


Example
^^^^^^^

.. code-block:: shell-session

   export QRMI_JOB_QPU_RESOURCES=garnet_mock
   export QRMI_JOB_QPU_TYPES=iqm-server
   export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
   export garnet_mock_QRMI_IQM_ISA_TOKEN=your token


How to run
----------

.. code-block:: shell-session

   python iqm_example.py
