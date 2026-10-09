.. _pasqal_local_c:

Pasqal Local QRMI - Examples in C
=================================

.. container:: buttons

   `GitHub`_

.. _GitHub: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/pasqal_local

--------------

Prerequisites
-------------

-  C compiler/linker, cmake and make
-  Build the :ref:`QRMI Rust library <install_source>`
-  `Munge`_

.. _Munge: https://dun.github.io/munge/


Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. `This example`_ assumes
that a ``.env`` file is available under the current directory.

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/pasqal_local

+-----------------------------------------------+------------------------------------+
| Environment variables                         | Descriptions                       |
+===============================================+====================================+
| ``<backend_name>_QRMI_WARDEN_URL``            | URL of the Warden middleware       |
|                                               | (e.g. ``http://localhost:4207``).  |
|                                               | ``<backend_name>_QRMI_URL`` is a   |
|                                               | deprecated fallback.               |
+-----------------------------------------------+------------------------------------+
| ``QRMI_JOB_UID``                              | ID of the user executing the job   |
+-----------------------------------------------+------------------------------------+
| ``QRMI_JOB_ID``                               | ID of the scheduler job, including |
|                                               | the array-task ID for array jobs   |
+-----------------------------------------------+------------------------------------+
| ``QRMI_JOB_QPU_SLOTS``                        | (optional) QPU slots to claim when |
|                                               | acquiring a Warden session.        |
|                                               | Positive integer, default ``1``.   |
+-----------------------------------------------+------------------------------------+
| ``<backend_name>_QRMI_JOB_ACQUISITION_TOKEN`` | (optional) Warden session acquired |
|                                               | by the scheduler for this job      |
+-----------------------------------------------+------------------------------------+

Where ``<backend_name>`` is the backend name passed as the first
argument (e.g. ``PASQAL_LOCAL``).


Create Pulser Sequence file as input
------------------------------------

Given a Pulser sequence ``sequence``, we can convert it to a JSON string
and write it to a file like this:

.. code-block:: python
   :linenos:

   serialized_sequence = sequence.to_abstract_repr()

   with open("pulser_seq.json", "w") as f:
       f.write(serialized_sequence)


How to build `this example`_
----------------------------

.. code-block:: bash

   mkdir build
   cd build
   cmake ..
   make


How to run `this example`_
--------------------------

.. code-block:: bash

   ./build/pasqal_local
   pasqal_local <backend name> <input file>

For example:

.. code-block:: bash

   ./build/pasqal_local PASQAL_LOCAL input.json
