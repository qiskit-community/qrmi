.. _task_runner_qiskit:

Tools to Generate EstimatorV2/SamplerV2 Primitive Input
=======================================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/qiskit

The tools demonstrate the generation of EstimatorV2/SamplerV2 inputs
from a quantum circuit example.


Prerequisites
-------------

-  Python 3.11 or above


Install dependencies
--------------------

.. code-block:: shell-session

   pip install -f requirements.txt


Tools
-----

``gen_estimator_input.py``
~~~~~~~~~~~~~~~~~~~~~~~~~~

Generates EstimatorV2 input for the circuit introduced in IBM's `Get Started with Estimator documentation`_.

.. _Get Started with Estimator documentation: https://quantum.cloud.ibm.com/docs/en/guides/get-started-with-estimator

Usage:

.. code-block:: shell-session

   usage: gen_estimator_inputs.py [-h] [--iam_url IAM_URL] backend base_url apikey crn

   A tool to generate EstimatorV2 input for testing

   positional arguments:
     backend     Backend name
     base_url    API endpoint
     apikey      IAM API key
     crn         Service CRN of your instance

   options:
     -h, --help  show this help message and exit
     --iam_url IAM_URL  IAM endpoint

Example:

.. code-block:: bash

   python gen_estimator_input.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance>

Output:

+-----------------------------------+-----------------------------------+
| Files                             | Descriptions                      |
+===================================+===================================+
| estimator_input                   | EstimatorV2 input                 |
| \_{backend_name}_params_only.json | parameters (`EstimatorV2 Schema`_)|
+-----------------------------------+-----------------------------------+
| est                               | An input for QRMI task runner,    |
| imator_input\_{backend_name}.json | which contains 2 properties -     |
|                                   | ``program_id`` (=\ ``estimator``) |
|                                   | and ``parameters`` (EstimatorV2   |
|                                   | input parameters).                |
+-----------------------------------+-----------------------------------+

.. _EstimatorV2 Schema: https://quantum.cloud.ibm.com/docs/en/api/qiskit-runtime-rest/tags/jobs

``gen_sampler_input.py``
~~~~~~~~~~~~~~~~~~~~~~~~

Generates SamplerV2 input for the circuit introduced in IBM's `Get Started with Sampler documentation`_.

.. _Get Started with Sampler documentation: https://quantum.cloud.ibm.com/docs/en/guides/get-started-with-sampler

Usage:

.. code-block:: shell-session

   usage: gen_sampler_inputs.py [-h] [--iam_url IAM_URL] backend base_url apikey crn

   A tool to generate SamplerV2 input for testing

   positional arguments:
     backend     Backend name
     base_url    API endpoint
     apikey      IAM API key
     crn         Service CRN of your instance

   options:
     -h, --help  show this help message and exit
     --iam_url IAM_URL  IAM endpoint

Example:

.. code-block:: bash

   python gen_sampler_input.py ibm_marrakesh https://quantum.cloud.ibm.com/api <your apikey> <your instance>

Output:

+-----------------------------------+-----------------------------------+
| Files                             | Descriptions                      |
+===================================+===================================+
| sampler_input                     | SamplerV2 input                   |
| \_{backend_name}_params_only.json | parameters(`SamplerV2             |
|                                   | s                                 |
|                                   | chema <https://github.com/Qiskit/ |
|                                   | ibm-quantum-schemas/blob/main/sch |
|                                   | emas/sampler_v2_schema.json>`__). |
+-----------------------------------+-----------------------------------+
| s                                 | An input for QRMI task runner,    |
| ampler_input\_{backend_name}.json | which contains 2 properties -     |
|                                   | ``program_id``\ (=\ ``sampler``)  |
|                                   | and ``parameters``\ (SamplerV2    |
|                                   | input parameters).                |
+-----------------------------------+-----------------------------------+
