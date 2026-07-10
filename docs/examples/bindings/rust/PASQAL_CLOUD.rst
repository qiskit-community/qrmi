Pasqal CLoud QRMI - Examples in Rust
====================================

`GitHub Repository`_

.. _GitHub Repository: https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/pasqal_cloud

Prerequisites
-------------

-  Python 3.11 or 3.12
-  `QRMI Rust library <../../../../README.md>`__

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

+-----------------------------------+-----------------------------------+
| Environment variables             | Descriptions                      |
+===================================+===================================+
| \_QRMI_PASQAL_CLOUD_PROJECT_ID    | Pasqal Cloud Project ID to access |
|                                   | the QPU                           |
+-----------------------------------+-----------------------------------+
| \_QRMI_PASQAL_CLOUD_AUTH_TOKEN    | Pasqal Cloud Auth Token (optional |
|                                   | when username/password are        |
|                                   | configured)                       |
+-----------------------------------+-----------------------------------+
| \_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT | (Optional) Auth endpoint URL/path |
|                                   | for token retrieval. Default:     |
|                                   | ``authen                          |
|                                   | ticate.pasqal.cloud/oauth/token`` |
+-----------------------------------+-----------------------------------+
| PASQAL_USERNAME                   | Pasqal Cloud username (optional,  |
|                                   | user-provided)                    |
+-----------------------------------+-----------------------------------+
| PASQAL_PASSWORD                   | Pasqal Cloud password (optional,  |
|                                   | user-provided)                    |
+-----------------------------------+-----------------------------------+

~/.pasqal/config (optional)
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Create ``~/.pasqal/config``:

::

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

.. code:: python

   serialized_sequence = sequence.to_abstract_repr()

   with open("pulser_seq.json", "w") as f:
       f.write(serialized_sequence)

How to build this example
-------------------------

.. code:: shell-session

   $ cargo clean
   $ cargo build --release

How to run this example
-----------------------

.. code:: shell-session

   $ ../target/release/qrmi-example-pasqal-cloud --help
   QRMI for Pasqal Cloud - Example

   Usage: qrmi-example-pasqal-cloud --backend <BACKEND> --input <INPUT>

   Options:
     -b, --backend <BACKEND>        backend name
     -i, --input <INPUT>            sequence input file
     -h, --help                     Print help
     -V, --version                  Print version

For example,

.. code:: shell-session

   $ ../target/release/qrmi-example-pasqal-cloud -b FRESNEL -i input.json
