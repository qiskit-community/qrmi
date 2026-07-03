Pulser Connection with Pasqal Cloud QRMI - Python Example
=========================================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/pulser/pasqal

Prerequisites
-------------

-  Python 3.11 or 3.12
-  `Installation of QRMI primitives python
   package(``qiskit-qrmi-primitives``) <../../README.md>`__

Install dependencies
--------------------

Assuming your python virtual environment is located at
``~/py311venv_qrmi_primitives/bin/activate``,

.. code:: shell-session

   $ source ~/py311venv_qrmi_primitives/bin/activate
   $ pip install -r requirements.txt

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
|                                   | e.g. ``FRESNEL``                  |
+-----------------------------------+-----------------------------------+
| QRMI_JOB_QPU_TYPES                | Quantum resource types.           |
|                                   | Comma-separated values            |
|                                   | corresponding to each Quantum     |
|                                   | resource name specified by        |
|                                   | ``Q                               |
|                                   | RMI_JOB_QPU_RESOURCES``.Supported |
|                                   | types:                            |
+-----------------------------------+-----------------------------------+

How to run
----------

SamplerV2
~~~~~~~~~

Use Pulser’s ``QPUBackend`` with ``PulserQRMIConnection``.

.. code:: shell-session

   $ python pulser_qrmi.py
