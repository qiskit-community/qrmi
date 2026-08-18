.. _pasqal_cloud_lua:

Pasqal Cloud QRMI - Examples in Lua
===================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/pasqal


Prerequisites
-------------

-  :ref:`QRMI C library (libqrmi.so) <building_core_qrmi_libraries>`
-  :ref:`QRMI Lua Module (qrmi.so) <installing_lua_bindings>`


Setup
-----

.. code:: shell-session

   export LUA_CPATH="</path/to/qrmi.so-dir/>?.so;;"
   export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libqrmi.so-dir

Example:

.. code:: shell-session

   export LUA_CPATH="/shared/qrmi/lua/build/?.so;;"
   export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/shared/qrmi/target/release


Set environment variables
-------------------------

QRMI supports Pasqal Cloud configuration via environment variables. For
Pasqal Cloud auth, QRMI also supports reading ``~/.pasqal/config``
(token or username/password). ``PASQAL_CONFIG_ROOT`` may point elsewhere
and takes priority over ``<backend_name>_PASQAL_CONFIG_ROOT``; QRMI
expands ``~``, ``$VAR``, and ``${VAR}`` before appending
``.pasqal/config``. # pragma: allowlist secret

The required environment variables are listed below. This example
assumes that a ``.env`` file is available under the current directory.

+----------------------------------------------------+-------------------------------------------+
|               Environment variables                |               Descriptions                |
+====================================================+===========================================+
| ``<backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID``    | Pasqal Cloud Project ID to access         |
|                                                    | the QPU                                   |
+----------------------------------------------------+-------------------------------------------+
| ``<backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN``    | Pasqal Cloud Auth Token (optional         |
|                                                    | when username/password are                |
|                                                    | configured)                               |
+----------------------------------------------------+-------------------------------------------+
| ``<backend_name>_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT`` | (Optional) Auth endpoint URL/path         |
|                                                    | for token retrieval. Default:             |
|                                                    | ``authenticate.pasqal.cloud/oauth/token`` |
+----------------------------------------------------+-------------------------------------------+
| ``PASQAL_USERNAME``                                | Pasqal Cloud username (optional,          |
|                                                    | user-provided)                            |
+----------------------------------------------------+-------------------------------------------+
| ``PASQAL_PASSWORD``                                | Pasqal Cloud password (optional,          |
|                                                    | user-provided)                            |
+----------------------------------------------------+-------------------------------------------+


``~/.pasqal/config`` (optional)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Create ``~/.pasqal/config``:

.. code-block:: text
   :caption: config

   username=<your username>
   password=<your password>
   # or:
   # token=<your token>
   # or:
   # client_id=<your client id>
   # client_secret=<your client secret>  # pragma: allowlist secret

   # optional override:
   # project_id=<your project id>
   # auth_endpoint=<auth endpoint URL/path>


Create Pulser Sequence file as input
------------------------------------

Given a Pulser sequence ``sequence``, we can convert it to a JSON string
and write it to a file like this:

.. code-block:: python
   :linenos:

   serialized_sequence = sequence.to_abstract_repr()

   with open("pulser_seq.json", "w") as f:
       f.write(serialized_sequence)


How to run `this example`_
--------------------------

.. _this example: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/pasqal

Run `example.lua`_:

.. _example.lua: https://github.com/qiskit-community/qrmi/blob/main/examples/qrmi/lua/pasqal/example.lua

.. code:: shell-session

   lua example.lua <backend name> <resource type> <input file>

For example:

.. code:: shell-session

   lua example.lua FRESNEL pasqal-cloud input.json
