.. _qiskit_ibm_primitive:

Sampler/Estimator Primitives with IBM Quantum System QRMI - Python Example
==========================================================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/qiskit_primitives/ibm

--------------

Prerequisites
-------------

-  Python 3.11 or 3.12
-  :ref:`Installation of QRMI primitives Python package (qiskit-qrmi-primitives) <qiskit_examples>`


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
required environment variables are listed below. `This example`_ assumes
that a ``.env`` file is available under the current directory.

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qiskit_primitives/ibm


Common
~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by the SPANK plugin.

+----------------------------+------------------------------------------+
|   Environment variables    |               Descriptions               |
+============================+==========================================+
| ``QRMI_JOB_QPU_RESOURCES`` | Quantum resource names.                  |
|                            | Comma-separated values,                  |
|                            | e.g. ``ibm_torino,ibm_brisbane``         |
+----------------------------+------------------------------------------+
| ``QRMI_JOB_QPU_TYPES``     | Quantum resource types.                  |
|                            | Comma-separated values                   |
|                            | corresponding to each Quantum            |
|                            | resource name specified by               |
|                            | ``QRMI_JOB_QPU_RESOURCES``.              |
|                            | Supported types: ``ibm-quantum-system``, |
|                            | ``ibm-quantum-compute-service``          |
+----------------------------+------------------------------------------+


IBM Quantum System specific
~~~~~~~~~~~~~~~~~~~~~~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by users or administrator.

+-------------------------------------------------------+------------------------------------------+
|                 Environment variables                 |               Descriptions               |
+=======================================================+==========================================+
| ``{resource_name}_QRMI_IBM_QS_ENDPOINT``              | Quantum System endpoint URL              |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_IAM_ENDPOINT``          | IBM Cloud IAM endpoint                   |
|                                                       | URL (e.g. ``https://iam.cloud.ibm.com``) |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_IAM_APIKEY``            | IBM Cloud IAM API Key                    |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_SERVICE_CRN``           | Cloud Resource Name (CRN) of the         |
|                                                       | provisioned Quantum System               |
|                                                       | instance, starting with                  |
|                                                       | ``crn:v1:``.                             |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_AWS_ACCESS_KEY_ID``     | AWS Access Key ID to access S3           |
|                                                       | bucket                                   |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY`` | AWS Secret Access Key to access          |
|                                                       | S3 bucket                                |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_S3_ENDPOINT``           | S3 endpoint URL                          |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_S3_BUCKET``             | S3 bucket name                           |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_S3_REGION``             | S3 bucket region                         |
|                                                       | name (e.g. ``us-east``)                  |
+-------------------------------------------------------+------------------------------------------+
| ``{resource_name}_QRMI_IBM_QS_TIMEOUT_SECONDS``       | Time (in seconds) after which job        |
|                                                       | should time out and get                  |
|                                                       | cancelled. It is based on system         |
|                                                       | execution time (not wall clock           |
|                                                       | time). System execution time is          |
|                                                       | the amount of time that the              |
|                                                       | system is dedicated to processing        |
|                                                       | your job.                                |
+-------------------------------------------------------+------------------------------------------+


Example
^^^^^^^

.. code-block:: shell-session

   export QRMI_JOB_QPU_RESOURCES=test_eagle
   export QRMI_JOB_QPU_TYPES=ibm-quantum-system
   export test_eagle_QRMI_IBM_QS_ENDPOINT=http://localhost:8080
   export test_eagle_QRMI_IBM_QS_IAM_ENDPOINT=https://iam.cloud.ibm.com
   export test_eagle_QRMI_IBM_QS_IAM_APIKEY=your_apikey
   export test_eagle_QRMI_IBM_QS_SERVICE_CRN=your_instance
   export test_eagle_QRMI_IBM_QS_AWS_ACCESS_KEY_ID=your_aws_access_key_id
   export test_eagle_QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY=your_aws_secret_access_key
   export test_eagle_QRMI_IBM_QS_S3_ENDPOINT=https://s3.us-east.cloud-object-storage.appdomain.cloud
   export test_eagle_QRMI_IBM_QS_S3_BUCKET=test
   export test_eagle_QRMI_IBM_QS_S3_REGION=us-east
   export test_eagle_QRMI_IBM_QS_TIMEOUT_SECONDS=86400


IBM Quantum Compute Service specific
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

When run as a job in a Slurm cluster, these environment variables are
set by users or administrator.

+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
|              Environment variables               |                                              Descriptions                                               |
+==================================================+=========================================================================================================+
| ``{resource_name}_QRMI_IBM_QCS_ENDPOINT``        | Quantum Compute Service endpoint                                                                        |
|                                                  | URL (e.g. ``https://quantum.cloud.ibm.com/api``)                                                        |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QCS_IAM_ENDPOINT``    | IBM Cloud IAM endpoint                                                                                  |
|                                                  | URL (e.g. ``https://iam.cloud.ibm.com``)                                                                |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QCS_IAM_APIKEY``      | IBM Cloud IAM API Key                                                                                   |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QCS_SERVICE_CRN``     | Cloud Resource Name (CRN) of the                                                                        |
|                                                  | provisioned Quantum System                                                                              |
|                                                  | instance, starting with                                                                                 |
|                                                  | ``crn:v1:``.                                                                                            |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QCS_TIMEOUT_SECONDS`` | Time (in seconds) after which job                                                                       |
|                                                  | should time out and get                                                                                 |
|                                                  | cancelled. It is based on system                                                                        |
|                                                  | execution time (not wall clock                                                                          |
|                                                  | time). System execution time is the amount of time that the system is dedicated to processing your job. |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QCS_SESSION_MODE``    | Session mode,                                                                                           |
|                                                  | default='dedicated', batch or                                                                           |
|                                                  | dedicated.                                                                                              |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+
| ``{resource_name}_QRMI_IBM_QCS_SESSION_ID``      | Session ID, set by acquire                                                                              |
|                                                  | function. Optional for acquire                                                                          |
|                                                  | function, however, required other                                                                       |
|                                                  | functions.                                                                                              |
+--------------------------------------------------+---------------------------------------------------------------------------------------------------------+



Example
^^^^^^^

.. code-block:: shell-session

   export QRMI_JOB_QPU_RESOURCES=ibm_torino,ibm_marrakesh
   export QRMI_JOB_QPU_TYPES=ibm-quantum-compute-service,ibm-quantum-compute-service
   export ibm_torino_QRMI_IBM_QCS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
   export ibm_torino_QRMI_IBM_QCS_IAM_ENDPOINT=https://iam.cloud.ibm.com
   export ibm_torino_QRMI_IBM_QCS_IAM_APIKEY=your_apikey
   export ibm_torino_QRMI_IBM_QCS_SERVICE_CRN=your_instance
   export ibm_marrakesh_QRMI_IBM_QCS_ENDPOINT=https://quantum.cloud.ibm.com/api/v1
   export ibm_marrakesh_QRMI_IBM_QCS_IAM_ENDPOINT=https://iam.cloud.ibm.com
   export ibm_marrakesh_QRMI_IBM_QCS_IAM_APIKEY=your_apikey
   export ibm_marrakesh_QRMI_IBM_QCS_SERVICE_CRN=your_instance


How to run `this example`_
--------------------------

SamplerV2
~~~~~~~~~

Code is based on the `"Get started with Sampler" tutorial`_.

.. _"Get started with Sampler" tutorial: https://quantum.cloud.ibm.com/docs/en/guides/get-started-with-sampler

Run `sampler.py`_:

.. _sampler.py: https://github.com/qiskit-community/qrmi/blob/main/examples/qiskit_primitives/ibm/sampler.py

.. code-block:: shell-session

   python sampler.py


EstimatorV2
~~~~~~~~~~~

Code is based on the `"Get started with Estimator" tutorial`_.

.. _"Get started with Estimator" tutorial: https://quantum.cloud.ibm.com/docs/en/guides/get-started-with-estimator

Run `estimator.py`_:

.. _estimator.py: https://github.com/qiskit-community/qrmi/blob/main/examples/qiskit_primitives/ibm/estimator.py

.. code-block:: shell-session

   python estimator.py


SQD Tutorial
~~~~~~~~~~~~

`01_chemistry_hamiltonian.ipynb`_ is a QRMI primitive port of `Improving energy estimation of a chemistry
Hamiltonian with SQD`_.

Start Jupyter Notebook and run all cells from the beginning.

.. _01_chemistry_hamiltonian.ipynb: https://github.com/qiskit-community/qrmi/blob/main/examples/qiskit_primitives/ibm/01_chemistry_hamiltonian.ipynb
.. _Improving energy estimation of a chemistry Hamiltonian with SQD: https://quantum.cloud.ibm.com/docs/en/tutorials/sample-based-quantum-diagonalization
