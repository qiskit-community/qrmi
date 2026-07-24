.. _iqm_server_c:

IQM Server QRMI - Examples in C
===============================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/iqm_server


Prerequisites
-------------

-  C compiler/linker, cmake and make
-  Build the :ref:`QRMI Rust library <install_source>`


Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. This example assumes
that a ``.env`` file is available under the current directory.

===================================== =======================
Environment variables                 Descriptions
===================================== =======================
{qc_alias_name}_QRMI_IQM_ISA_ENDPOINT IQM Server API endpoint
{qc_alias_name}_QRMI_IBM_ISA_TOKEN    IQM Server API token
===================================== =======================

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


How to build this example
-------------------------

.. code-block:: shell-session

   mkdir build
   cd build
   cmake ..
   make


How to run this example
-----------------------

.. code-block:: shell-session

   ./build/iqm_server
   iqm_server <qc_alias> <IQM JSON> <job_type('circuit','run' or 'sweep')

For example:

.. code-block:: shell-session

   export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
   export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

   ./iqm_server garnet_mock /shared/qrmi/examples/task_runner/iqm/iqm_json_garnet\:mock_params_only.json circuit
