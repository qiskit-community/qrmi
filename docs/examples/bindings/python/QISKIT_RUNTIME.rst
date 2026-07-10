Qiskit Runtime Service QRMI - Examples in Python
================================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/python/qiskit_runtime_service

Prerequisites
-------------

-  Rust 1.85.1 or above
-  Python 3.11 or 3.12
-  `QRMI python package installation <../../../../README.md>`__

Install dependencies
--------------------

.. code:: shell-session

   $ source ~/py311_qrmi_venv/bin/activate
   $ pip install -r ../requirements.txt

Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. This example assumes
that a ``.env`` file is available under the current directory.

+-----------------------------------+-----------------------------------+
| Environment variables             | Descriptions                      |
+===================================+===================================+
| {res                              | Qiskit Runtime Service endpoint   |
| ource_name}_QRMI_IBM_QRS_ENDPOINT | URL(e.g. ``htt                    |
|                                   | ps://quantum.cloud.ibm.com/api``) |
+-----------------------------------+-----------------------------------+
| {resourc                          | IBM Cloud IAM endpoint            |
| e_name}_QRMI_IBM_QRS_IAM_ENDPOINT | URL(e.                            |
|                                   | g. ``https://iam.cloud.ibm.com``) |
+-----------------------------------+-----------------------------------+
| {resou                            | IBM Cloud IAM API Key             |
| rce_name}_QRMI_IBM_QRS_IAM_APIKEY |                                   |
+-----------------------------------+-----------------------------------+
| {resour                           | Cloud Resource Name(CRN) of the   |
| ce_name}_QRMI_IBM_QRS_SERVICE_CRN | provisioned Qiskit Runtime        |
|                                   | Service instance, starting with   |
|                                   | ``crn:v1:``.                      |
+-----------------------------------+-----------------------------------+
| {resourc                          | Execution mode to run the session |
| e_name}_QRMI_IBM_QRS_SESSION_MODE | in, ``default='dedicated'``,      |
|                                   | ``batch`` or ``dedicated``.       |
+-----------------------------------+-----------------------------------+
| {resource_n                       | The maximum time (in seconds) for |
| ame}_QRMI_IBM_QRS_SESSION_MAX_TTL | the session to run, subject to    |
|                                   | plan limits, default: ``28800``.  |
+-----------------------------------+-----------------------------------+
| {resource_n                       | (Optional) Cost of the job as the |
| ame}_QRMI_IBM_QRS_TIMEOUT_SECONDS | estimated time it should take to  |
|                                   | complete (in seconds). Should not |
|                                   | exceed the cost of the program,   |
|                                   | default: ``None``.                |
+-----------------------------------+-----------------------------------+
| {resou                            | (Optional) Session ID, can be     |
| rce_name}_QRMI_IBM_QRS_SESSION_ID | obtanied by acquire function. If  |
|                                   | exists, used in the target        |
|                                   | functions.                        |
+-----------------------------------+-----------------------------------+

Create Qiskit Primitive input file as input
-------------------------------------------

Refer `this tool <../../../../examples/task_runner/qiskit>`__ to
generate. You can customize quantum circuits by editing the code.

   [!NOTE] Use the file with name ending with ``_params_only.json``,
   e.g. ``sampler_input_ibm_torino_params_only.json``.

How to run
----------

.. code:: shell-session

   $ python example.py -h
   usage: example.py [-h] backend input program_id

   An example of IBM Qiskit Runtime Service QRMI

   positional arguments:
     backend     backend name
     input       primitive input file
     program_id  'estimator' or 'sampler'

   options:
     -h, --help  show this help message and exit

For example,

.. code:: shell-session

   export ibm_torino_QRMI_IBM_QRS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
   export ibm_torino_QRMI_IBM_QRS_IAM_ENDPOINT=https://iam.cloud.ibm.com
   export ibm_torino_QRMI_IBM_QRS_IAM_APIKEY=your_apikey
   export ibm_torino_QRMI_IBM_QRS_SERVICE_CRN=your_instance

   python example.py ibm_torino sampler_input.json sampler
