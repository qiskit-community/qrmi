.. _config_c:

Parsing QRMI Config File in C
=============================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/config


Prerequisites
-------------

-  C compiler/linker, cmake and make
-  Build the :ref:`QRMI Rust library <install_source>`


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

   ./build/
   qrmi_config <qrmi_config.json file> <resource name>

For example:

.. code-block:: shell-session

   ./build/qrmi_config /etc/slurm/qrmi_config.json ibm_fez
