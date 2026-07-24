.. _iqm_server_python:

IQM Server QRMI - Examples in Python
====================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/python/iqm_server


Prerequisites
-------------

-  Rust 1.85.1 or above
-  Python 3.11 or 3.12
-  Install the :ref:`QRMI Python package <install_source>`


Install dependencies
--------------------

.. code-block:: shell-session

   source ~/py311_qrmi_venv/bin/activate
   pip install -r ../requirements.txt


Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. This example assumes
that a ``.env`` file is available under the current directory.

===================================== ===========================
Environment variables                 Descriptions
===================================== ===========================
{qc_alias_name}_QRMI_IQM_ISA_ENDPOINT IQM Server API endpoint URL
{qc_alias_name}_QRMI_IQM_ISA_TOKEN    IQM Server API token
===================================== ===========================

.. note::

   Replace the ":" in the QC alias name with "\_" when
   specifying it. For example, ``sirius:mock`` -> ``sirius_mock``.


Create IQM JSON input file as input
-----------------------------------

Refer to :ref:`this tool <task_runner_iqm>` to
generate. You can customise quantum circuits by editing the code.

.. note::

   Use the file with name ending ``_params_only.json``,
   e.g. ``iqm_json_sirius_params_only.json``.


How to run
----------

.. code-block:: shell-session

   python example.py -h
   usage: example.py [-h] qc_alias input job_type

   An example of IBM Quantum System QRMI

   positional arguments:
     qc_alias    QC alias name
     input       IQM JSON input file
     job_type    'circuit','run', or 'sweep'

   options:
     -h, --help  show this help message and exit

For example:

.. code-block:: shell-session

   export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
   export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

   python example.py garnet_mock /shared/qrmi/examples/task_runner/iqm/iqm_json_garnet\:mock_params_only.json circuit
