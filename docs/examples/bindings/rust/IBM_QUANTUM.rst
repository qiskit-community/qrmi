Quantum System QRMI - Examples in Rust
======================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/ibm_quantum_system

Prerequisites
-------------

-  Python 3.11 or 3.12
-  `QRMI Rust library <../../../../README.md>`__

Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. This example assumes
that a ``.env`` file is available under the current directory.

+-----------------------------------+-----------------------------------+
| Environment variables             | Descriptions                      |
+===================================+===================================+
| {re                               | Quantum System endpoint URL       |
| source_name}_QRMI_IBM_QS_ENDPOINT |                                   |
+-----------------------------------+-----------------------------------+
| {resour                           | IBM Cloud IAM endpoint            |
| ce_name}_QRMI_IBM_QS_IAM_ENDPOINT | URL(e.                            |
|                                   | g. ``https://iam.cloud.ibm.com``) |
+-----------------------------------+-----------------------------------+
| {reso                             | IBM Cloud IAM API Key             |
| urce_name}_QRMI_IBM_QS_IAM_APIKEY |                                   |
+-----------------------------------+-----------------------------------+
| {resou                            | Cloud Resource Name(CRN) of the   |
| rce_name}_QRMI_IBM_QS_SERVICE_CRN | provisioned Quantum System        |
|                                   | instance, starting with           |
|                                   | ``crn:v1:``.                      |
+-----------------------------------+-----------------------------------+
| {resource_na                      | AWS Access Key ID to access S3    |
| me}_QRMI_IBM_QS_AWS_ACCESS_KEY_ID | bucket                            |
+-----------------------------------+-----------------------------------+
| {resource_name}_                  | AWS Secret Access Key to access   |
| QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY | S3 bucket                         |
+-----------------------------------+-----------------------------------+
| {resou                            | S3 endpoint URL                   |
| rce_name}_QRMI_IBM_QS_S3_ENDPOINT |                                   |
+-----------------------------------+-----------------------------------+
| {res                              | S3 bucket name                    |
| ource_name}_QRMI_IBM_QS_S3_BUCKET |                                   |
+-----------------------------------+-----------------------------------+
| {res                              | S3 bucket region                  |
| ource_name}_QRMI_IBM_QS_S3_REGION | name(e.g. ``us-east``)            |
+-----------------------------------+-----------------------------------+
| {resour                           | Time (in seconds) after which job |
| ce_name}_QRMI_JOB_TIMEOUT_SECONDS | should time out and get           |
|                                   | cancelled. It is based on system  |
|                                   | execution time (not wall clock    |
|                                   | time). System execution time is   |
|                                   | the amount of time that the       |
|                                   | system is dedicated to processing |
|                                   | your job.                         |
+-----------------------------------+-----------------------------------+

Create Qiskit Primitive input file as input
-------------------------------------------

Refer `this tool <../../../../examples/task_runner/qiskit>`__ to
generate. You can customize quantum circuits by editing the code.

   [!NOTE] Use the file with name ending with ``_params_only.json``,
   e.g. ``sampler_input_ibm_torino_params_only.json``.

How to build this example
-------------------------

.. code:: shell-session

   $ cargo clean
   $ cargo build --release

How to run this example
-----------------------

.. code:: shell-session

   $ ../target/release/qrmi-example-ibm-quantum-system --help
   QRMI for IBM Quantum System - Example

   Usage: qrmi-example-ibm-quantum-system --backend <BACKEND> --input <INPUT> --program-id <PROGRAM_ID>

   Options:
     -b, --backend <BACKEND>        backend name
     -i, --input <INPUT>            primitive input file
     -p, --program-id <PROGRAM_ID>  program id
     -h, --help                     Print help
     -V, --version                  Print version

For example,

.. code:: shell-session

   export test_eagle_QRMI_IBM_QS_ENDPOINT=http://localhost:8080
   export test_eagle_QRMI_IBM_QS_IAM_ENDPOINT=https://iam.cloud.ibm.com
   export test_eagle_QRMI_IBM_QS_IAM_APIKEY=your_apikey
   export test_eagle_QRMI_IBM_QS_SERVICE_CRN=your_instance
   export test_eagle_QRMI_IBM_QS_AWS_ACCESS_KEY_ID=your_aws_access_key_id
   export test_eagle_QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY=your_aws_secret_access_key
   export test_eagle_QRMI_IBM_QS_S3_ENDPOINT=https://s3.us-east.cloud-object-storage.appdomain.cloud
   export test_eagle_QRMI_IBM_QS_S3_BUCKET=test
   export test_eagle_QRMI_IBM_QS_S3_REGION=us-east
   export test_eagle_QRMI_JOB_TIMEOUT_SECONDS=86400

   ../target/release/qrmi-example-ibm-quantum-system -b test_eagle -i sampler_input.json -p sampler
