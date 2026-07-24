.. _alice_and_bob_felis_python:

Alice and Bob Felis QRMI - Examples in Python
=============================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/python/alice_bob_felis


Prerequisites
-------------

-  Rust 1.85.1 or above
-  Python 3.11 or 3.12
-  Install the :ref:`QRMI Python package <install_source>`


Install dependencies
--------------------

.. code-block:: shell-session

   pip install -r requirements.txt


.. _alice_and_bob_felis_python_env:

Set environment variables
-------------------------

Because QRMI is an environment variable driven software library, all
configuration parameters must be specified in environment variables. The
required environment variables are listed below. You can either create a
``.env`` file in the current directory or set them in the shell
directly.

=========================== =============
Environment variables       Descriptions
=========================== =============
QRMI_AB_FELIS_BASE_ENDPOINT Felis URL
QRMI_AB_FELIS_API_KEY       Felis API Key
=========================== =============

You may also use separate URLs and API keys for different target
resources:

=========================================== =============
Environment variables                       Descriptions
=========================================== =============
{resource_name}_QRMI_AB_FELIS_BASE_ENDPOINT Felis URL
{resource_name}_QRMI_AB_FELIS_API_KEY       Felis API Key
=========================================== =============


.. _alice_and_bob_felis_python_qri:

Generate QIR input file
-----------------------

Felis accepts circuits expressed in human-readable QIR, a language which
expresses intermediate representations of quantum circuits. It supports
standard QIR gates together with a few custom Alice & Bob gates.

You can optionally modify the Qiskit circuit in
``generate_input_native_gates.py`` before generating it in QIR form as
follows:

.. code-block:: shell-session

   export QRMI_AB_FELIS_BASE_ENDPOINT='<felis endpoint>'
   export QRMI_AB_FELIS_API_KEY='<your felis api key>'
   python generate_input_native_gates.py EMU:1Q:LESCANNE_2020 > generated_circuit.ll

The environment variables are necessary for transpilation, which
requires access to the Felis API.

If you installed ``qrmi`` without the ``alice-bob`` dependency, which
installs `qiskit-alice-bob-provider`_, you will still be able to run trivial circuits using
`generate_input_generic.py`_, however you won't be able to use Alice and Bob native gates
like ``measure_x``. To do so:

.. _qiskit-alice-bob-provider: https://github.com/Alice-Bob-SW/qiskit-alice-bob-provider
.. _generate_input_generic.py: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/python/alice_bob_felis/generate_input_generic.py

.. code-block:: shell-session

   cd examples/qrmi/python/alice_bob/
   python generate_input_generic.py > generated_circuit.ll


How to run
----------

.. code-block:: shell-session

   python example.py -h
   usage: example.py [-h] target qir_file

   An example of a Quantum Resource from Alice and Bob's Felis API

   positional arguments:
     target      backend name
     qir_file    qir input file

   options:
     -h, --help  show this help message and exit

For example:

.. code-block:: shell-session

   export ab_emu_1q_lescanne_2020_QRMI_AB_FELIS_BASE_ENDPOINT='https://api.alice-bob.com/'
   export ab_emu_1q_lescanne_2020_QRMI_AB_FELIS_API_KEY='<your felis api key>'

   python example.py 'ab_emu_1q_lescanne_2020' generated_circuit.ll
