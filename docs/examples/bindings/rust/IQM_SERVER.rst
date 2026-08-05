.. _iqm_server_rust:

IQM Server QRMI - Examples in Rust
==================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/iqm_server


Prerequisites
-------------

-  Python 3.11 or 3.12
-  Build the :ref:`QRMI Rust library <install_source>`


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


How to build this example
-------------------------

.. code-block:: shell-session

   cargo clean
   cargo build --release


How to run this example
-----------------------

.. code-block:: shell-session

   ../target/release/qrmi-example-iqm-server -h
   QRMI for IQM Server - Example

   Usage: qrmi-example-iqm-server [OPTIONS] --qc-alias <QC_ALIAS> --iqmjson <IQMJSON> --job-type <JOB_TYPE>

   Options:
     -q, --qc-alias <QC_ALIAS>          QC alias name
     -i, --iqmjson <IQMJSON>            IQM JSON file
     -j, --job-type <JOB_TYPE>          Job type('circuit','run', or 'sweep')
     -u, --use-timeslot <USE_TIMESLOT>  use_timeslot [possible values: true, false]
     -t, --tag <TAG>                    tag
     -h, --help                         Print help
     -V, --version                      Print version

For example:

.. code-block:: shell-session

   export garnet_mock_QRMI_IQM_ISA_ENDPOINT=https://resonance.meetiqm.com
   export garnet_mock_QRMI_IQM_ISA_TOKEN=your api token

   ../target/release/qrmi-example-iqm-server -q garnet_mock -i /shared/qrmi/examples/task_runner/iqm/iqm_json_garnet\:mock_params_only.json -j circuit 
