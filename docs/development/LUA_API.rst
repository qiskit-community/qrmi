.. _lua_api:

Lua API Reference
=================

.. rst-class:: lead

   QRMI Lua bindings and available interfaces for developing quantum workflows in Lua.

--------------

Generated from the Doxygen comments in `lua_qrmi.c`_. This summarises
the full API exposed by the module loaded via ``require("qrmi")``, from
a Lua consumer's point of view (for C-level implementation details, see
`lua_qrmi.c`_ itself).

.. _lua_qrmi.c: https://github.com/qiskit-community/qrmi/blob/main/lua/lua_qrmi.c

The module provides two independent object types:

-  **``qrmi.resource``** — a handle to a single quantum resource
   (created via ``qrmi.new()``)
-  **``qrmi.config``** — a handle to a ``qrmi_config.json`` config file
   (created via ``qrmi.load_config()``) Completely independent lifecycle
   from ``qrmi.resource``

Every method follows the same two-value error pattern on failure:
``(value, ...)`` on success, ``(nil, errmsg)`` on failure (the same
convention Lua's ``io.open`` and similar functions use).


Design Notes
------------

1. **Converting QrmiReturnCode into Lua's (value, err) pattern** Every
   real API call returns a ``QrmiReturnCode``, so on failure
   ``qrmi_get_last_error()`` is called to fetch a detailed message,
   returned to Lua as ``(nil, errmsg)``. On success, the out-parameter's
   value is pushed directly.

2. **String ownership management** Any ``char*`` heap-allocated on the
   Rust side (via cbindgen) is freed with ``qrmi_string_free()``. This
   implementation frees it immediately after copying it into Lua
   (``lua_pushstring``), so nothing leaks.

3. **Resource lifecycle and GC** ``QrmiQuantumResource*`` is an opaque
   pointer. The userdata holds exactly one, and the ``__gc`` metamethod
   ensures it's automatically ``release``\ d and ``free``\ d even if the
   caller forgets to do so explicitly.


--------------


Module functions
----------------

``qrmi.new(resource_id, resource_type)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Creates a quantum resource handle.

+-------------------+--------+-----------------------------------------+
|     Argument      |  Type  |               Description               |
+===================+========+=========================================+
| ``resource_id``   | string | e.g. ``"ibm_kingston"``                 |
+-------------------+--------+-----------------------------------------+
| ``resource_type`` | string | Canonical name from                     |
|                   |        | ``qrmi_config_resource_type_to_str()``. |
|                   |        | One of the values below                 |
+-------------------+--------+-----------------------------------------+

Valid values for ``resource_type``: ``ibm-quantum-system`` /
``ibm-quantum-compute-service`` / ``pasqal-cloud`` / ``pasqal-local`` /
``alice-bob-felis`` / ``iqm-server``

**Returns:** on success, ``resource`` (a ``qrmi.resource``); on failure,
``nil, err``

.. code:: lua

   local resource, err = qrmi.new("ibm_kingston", "ibm-quantum-compute-service")

``qrmi.load_config(filename)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Loads a ``qrmi_config.json`` file. Entirely independent from
``qrmi.resource``.

**Returns:** on success, ``config`` (a ``qrmi.config``); on failure,
``nil, err``

.. code:: lua

   local config, err = qrmi.load_config("/etc/slurm/qrmi_config.json")


--------------


``qrmi.resource`` methods
-------------------------

``resource:is_accessible()``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Checks whether the device is reachable.

**Returns:** on success, ``accessible`` (boolean); on failure,
``nil, err``

``resource:id()``
~~~~~~~~~~~~~~~~~

Fetches the resource's identifier (the same ``resource_id`` passed to
``qrmi.new()``).

**Returns:** on success, ``id`` (string); on failure, ``nil, err``

``resource:type()``
~~~~~~~~~~~~~~~~~~~

Fetches the resource's type. Returns the canonical hyphenated string
(the same form accepted by ``qrmi.new()``) rather than the raw enum
value.

**Returns:** on success, ``type`` (string,
e.g. ``"ibm-quantum-system"``); on failure, ``nil, err``

``resource:acquire()``
~~~~~~~~~~~~~~~~~~~~~~

Acquires exclusive access to the resource. The returned token is also
cached internally, so a later ``release()`` call with no arguments, or
the ``__gc`` finalizer, can release it automatically.

If a token from an earlier, unreleased ``acquire()`` is still held, it
is properly released via ``qrmi_resource_release()`` before acquiring a
new one.

**Returns:** on success, ``token`` (string); on failure, ``nil, err``

.. code:: lua

   local token, err = resource:acquire()

``resource:release([token])``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Releases a previously acquired resource. If ``token`` is omitted, the
token cached from the last ``acquire()`` call is used automatically.

**Returns:** on success, ``ok`` (boolean, always true); on failure,
``nil, err``

.. code:: lua

   resource:release()          -- uses the cached token automatically
   resource:release(token)     -- explicit token

``resource:task_start(payload)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Starts a task. ``payload`` is a Lua table mirroring the C API's
``QrmiPayload`` tagged union: the table's single key names the payload
variant, and its value is a sub-table holding that variant's fields.
Four variants are currently supported:

**Qiskit Primitive (IBM)**

.. code:: lua

   resource:task_start({
       qiskit_primitive = {
           program_id = "estimator",   -- "estimator" or "sampler"
           input = json_str,           -- Qiskit Primitive input (JSON string)
       }
   })

**IQM Server**

.. code:: lua

   resource:task_start({
       iqm_server = {
           iqmjson = json_str,          -- IQM JSON request body
           job_type = "circuit",        -- "circuit" / "run" / "sweep"
           use_timeslot = false,        -- optional, defaults to false
           tag = "my-job",              -- optional, may be nil
       }
   })

**Pasqal Cloud or Pasqal Local** (both use the same ``pasqal_cloud``
key, since qrmi.h has only one ``QRMI_PAYLOAD_PASQAL_CLOUD`` tag)

.. code:: lua

   resource:task_start({
       pasqal_cloud = {
           sequence = pulser_sequence_str,  -- Pulser sequence
           job_runs = 100,                  -- number of runs
       }
   })

**Alice & Bob Felis**

.. code:: lua

   resource:task_start({
       alice_bob_felis = {
           human_qir = qir_str,          -- human-readable QIR input
           input_params = json_str,      -- input parameters (JSON format)
       }
   })

**Returns:** on success, ``task_id`` (string); on failure, ``nil, err``

``resource:task_status(task_id)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Fetches a task's current status.

**Returns:** on success, ``status`` (string: ``"queued"`` /
``"running"`` / ``"completed"`` / ``"failed"`` / ``"cancelled"``); on
failure, ``nil, err``

Recommended usage (poll until a terminal status is reached):

.. code:: lua

   local terminal = { completed = true, failed = true, cancelled = true }
   local status = resource:task_status(task_id)
   while status and not terminal[status] do
       os.execute("sleep 1")
       status = resource:task_status(task_id)
   end

``resource:task_result(task_id)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Fetches a completed task's result.

**Returns:** on success, ``result_json`` (string); on failure,
``nil, err``

``resource:task_logs(task_id)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Fetches a task's log messages.

**Returns:** on success, ``logs`` (string); on failure, ``nil, err``

``resource:task_stop(task_id)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Stops a running task.

**Returns:** on success, ``ok`` (boolean, always true); on failure,
``nil, err``

``resource:metadata()``
~~~~~~~~~~~~~~~~~~~~~~~

Fetches the resource's metadata as a Lua table (combines
``qrmi_resource_metadata`` + ``qrmi_resource_metadata_keys`` +
``qrmi_resource_metadata_value`` into a single call).

**Returns:** on success, ``metadata`` (table, string → string); on
failure, ``nil, err``

.. code:: lua

   local meta = resource:metadata()
   print(meta.backend_name, meta.n_qubits)

``resource:target()``
~~~~~~~~~~~~~~~~~~~~~

Fetches the device’s target information.

**Returns:** on success, ``target_json`` (string); on failure,
``nil, err``

``resource:free()``
~~~~~~~~~~~~~~~~~~~

Explicitly releases and frees the resource (auto-releases first if still
acquired). Also happens automatically via ``__gc`` if not called
explicitly, but calling it explicitly is recommended.

**Returns:** nothing

--------------

``qrmi.config`` methods
-----------------------

Independent from ``qrmi.resource``.

``config:resource_def(resource_id)``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Looks up a resource's definition in the config file.

**Returns:** on success, ``def`` (table); on failure, ``nil, err``

Structure of ``def``:

.. code:: lua

   {
       name = "ibm_kingston",
       type = "ibm-quantum-compute-service",
       is_dynamic = false,
       environments = {
           QRMI_IBM_QRS_ENDPOINT = "...",
           QRMI_IBM_QRS_IAM_ENDPOINT = "...",
       },
   }

``config:free()``
~~~~~~~~~~~~~~~~~

Explicitly frees the config. Also auto-frees via ``__gc``.

**Returns:** nothing

--------------

Common error-handling pattern
-----------------------------

Every method returns ``(nil, errmsg)`` on failure:

.. code:: lua

   local val, err = resource:some_method()
   if not val then
       print("failed:", err)
   end

Calling a method on a ``resource`` or ``config`` that has already been
``free()``\ d raises a **Lua error** (via ``error()``) rather than
returning the usual value pair:

.. code:: lua

   local ok, err = pcall(function() resource:is_accessible() end)
   -- ok = false, err = "qrmi resource already freed"

--------------

Not yet implemented (room for extension)
----------------------------------------

-  ``QrmiResourceProvider`` / ``qrmi_provider_new()`` — resource
   discovery / least-busy selection
-  Redirecting logs to Lua via ``qrmi_log_callback_set()``
