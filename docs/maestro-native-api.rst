.. _maestro_native_api:

Maestro native requests through QRMI
====================================

Maestro Local now supports a versioned native request through the existing
``Payload.MaestroLocal`` layout. Set ``job_type="request"`` and put the complete
schema-2 document in ``input``. Legacy execute/estimate payloads still work. Rebuild
Maestro, maestro-local-server, and QRMI together to use the new path. Older servers
produce an explicit unsupported-API error during capability negotiation.

The server uses only Rust/C/C++; Python is an optional client interface.

Python client
-------------

.. code:: python

   import json
   import time
   from qrmi import QuantumResource, ResourceType, TaskStatus
   from qrmi.maestro import request_payload

   resource = QuantumResource("MAESTRO_LOCAL", ResourceType.MaestroLocal)
   print(json.loads(resource.target().value))
   token = resource.acquire()
   try:
       document = {
           "schema_version": 2,
           "operation": "estimate",
           "circuit": {
               "num_qubits": 1,
               "source": "OPENQASM 2.0;\nqreg q[1]; h q[0];",
           },
           "simulator": {"backend": "qcsim", "method": "density_matrix"},
           "observables": ["X", "Z"],
           "execution": {"seed": 123},
           "noise": {
               "evaluation": "exact",
               "channels": [{"kind": "t1", "targets": [0], "gamma": 0.36}],
           },
       }
       task = resource.task_start(request_payload(document))
       deadline = time.monotonic() + 60
       while True:
           status = resource.task_status(task)
           if status in (TaskStatus.Completed, TaskStatus.Failed):
               break
           if time.monotonic() >= deadline:
               resource.task_stop(task)
               raise TimeoutError("Maestro task exceeded 60 seconds")
           time.sleep(0.1)
       if status == TaskStatus.Failed:
           raise RuntimeError(resource.task_logs(task))
       result = json.loads(resource.task_result(task).value)
       print(result["expectation_values"])  # [0.8, 0.36]
       print(resource.task_logs(task))
   finally:
       resource.release(token)

``target().value`` contains the complete public/native capability document.
``task_logs()`` returns serialized JSON containing captured stdout/stderr, the native
error, and a failure message. Results retain native execution metadata, seeds, noise
approximations, timing, and distribution details. Result retrieval still consumes the
result; logs remain available until session cleanup. The logs' ``seeds`` array
retains each resolved execution ``seed`` and, when present, ``noise_seed`` after
result retrieval. Each entry's ``result_path`` is empty for a single request or
contains successive result indices for nested batches. Replay the corresponding
original request with those values as ``execution.seed`` and ``noise.seed``.
Seed history is limited to 256 entries and four batch levels;
``seeds_truncated`` indicates omitted results. It lasts until session cleanup or
server restart. Seeded samples replay only on the same native library build;
newer servers record that build as ``native_build`` (``{version,
source_revision}``) in the same logs, or ``null`` for MPI tasks and tasks that
never returned a result. Updated servers also retain
logs after successful cancellation, including final output drained during worker
shutdown. Observe failed status before
requesting a result; QRMI raises ``TaskNotReady`` for failed tasks. If execution and
cleanup both fail, ``error.code`` is ``cleanup_unconfirmed``, with
``cleanup_confirmed:false`` and a bounded ``causes`` array retaining earlier error
codes/messages. Causes may be nested; inspect them recursively for the original native
failure. An active cancellation still fails if cleanup cannot be confirmed. Submission
errors retain typed QRMI categories: unsupported capabilities map to
``UnsupportedFunction`` / Python ``UnsupportedFunctionError``; invalid configuration and
missing/expired sessions map to ``InvalidConfig`` / Python ``ConfigError``. A missing
session calls for acquiring a new session, not choosing a new backend. Malformed
launch/rank input maps to ``InvalidInputError``; a broken profile file or
missing/nonexecutable launcher, worker or Slurm cancellation tool maps to
``ConfigError``. Selecting an unknown profile maps to ``UnsupportedFunctionError``.

These categories follow ``QuantumResource::task_start``'s contract: every error
except the generic one (Rust ``QrmiErrorKind::Other``, Python ``QrmiError``, C
``QRMI_RETURN_CODE_ERROR``) means that no task was created. Only a well-formed
API v2 refusal from the server establishes that. After the request has been
sent, a lost connection, a missing ``task_id``, a reply without the ``OK``
prefix or with another API version, or any other unrecognized reply is the
generic error, whose message says that the task may have been created. Do not
resubmit blindly in the same session after such an error: releasing the
acquisition token deletes the session and cancels any task it still runs.

Cancelling a task and deleting a session are acknowledged only after the server
has stopped the task's worker; an MPI launcher gets 20 seconds to clean up its
remote ranks and 5 more before it is reaped. QRMI therefore waits up to 60
seconds for these two replies, and 15 seconds for others. Cancellation removes
the task. When a missing task's retained diagnostics end with a ``cancelled``
event, because another client cancelled it or a cancellation acknowledgement was
lost, ``task_status`` reports ``Cancelled`` and ``task_stop`` succeeds, so task
runners stop polling. Any other missing task remains ``TaskNotFound``: a missing
task alone proves neither cancellation nor failure, and diagnostics disappear
with the session or a daemon restart. Socket exchanges, including these waits,
run off the asynchronous executor.
Unsupported-API errors come only from capability negotiation, before any
submission is sent.

Updated servers add ``session_id``, ``task_id``, ``context`` and ``events`` to the
same diagnostic JSON. Each event has ``timestamp_unix_ms`` and ``state``; submitted
tasks normally progress through ``queued``, ``started`` and ``completed``,
``failed`` or ``cancelled``. A failed cancellation records ``cancellation_failed``.
Queued cancellation skips ``started``; cancellation before submission records
only ``cancelled``. These are server lifecycle transitions, not native progress.
The event array defines order even if the wall clock changes. History retains
the first and latest seven events, with ``events_truncated`` indicating omissions.
Context records requested backend/method and MPI profile/ranks when supplied;
legacy tasks expose simulator/method IDs. It excludes circuit contents and
arbitrary options. Effective native backend selection remains in result metadata.

Captured stdout/stderr retains its first and last 32 KiB. For longer output,
``logs_truncated`` is true, ``logs_omitted_bytes`` counts omitted raw bytes, and
``logs`` contains a visible truncation marker between the retained pieces.
The streams are merged in capture order and decoded with UTF-8 replacement for
invalid bytes. Diagnostics live in server memory and disappear on session
deletion, expiry or daemon restart. ``ok:true`` means retrieval succeeded, not
that the task completed successfully. QRMI forwards all these additive fields;
older API-v2 servers may omit them and retain their previous cancellation/capture
behavior. Rebuild the local server to obtain the improved diagnostics.

The server advertises ``server.session_lease_seconds`` (default 3600). Poll a running
task or retrieve its logs comfortably within that interval. Native submit and logs also
renew the lease. Explicit ``keepalive`` is available on the standalone bundled Rust
socket client, not through QRMI's ``QuantumResource`` interface or Python/C/Lua resource
bindings. QRMI does not start a background heartbeat. Cached terminal status and session
existence checks do not renew it; logs can renew it after result consumption. A client
that stops all task traffic can lose its session, and expiry cancels active work.

Rust, C, and Lua
----------------

The existing payload fields and C ABI layout are unchanged:

.. code:: rust

   let payload = Payload::MaestroLocal {
       input: serde_json::to_string(&document)?,
       job_type: "request".into(),
       qubits: 0,
       simulator_type: 0,
       simulation_method: 0,
       observables: String::new(),
       config: "{}".into(),
   };

C and Lua use the same existing Maestro payload construction with ``job_type`` set to
``request``, ``input`` set to serialized JSON, and unused legacy fields set to the
values above. In C, NULL ``observables`` and ``config`` select ``""`` and ``{}``, so
a zero-initialized payload needs only ``input`` and ``job_type``; NULL ``input`` or
``job_type`` is refused with ``QRMI_RETURN_CODE_INVALID_INPUT_ERROR`` before any
request is sent. Lua raises an error for a legacy ``qubits``, ``simulator_type`` or
``simulation_method`` outside ``0..4294967295`` rather than truncating it; request
mode ignores these fields. Backend/method selection is inside the document and uses symbolic names,
avoiding build-dependent legacy enum numbers. There is no new payload enum or Slurm
plugin ABI requirement for this feature. Rebuild QRMI itself; existing consumers must
still match the repository's existing QRMI ABI version.

The bundled Rust socket client exposes ``maestro_local_api::maestro::request::Client``
with ``capabilities``, ``submit``, ``logs``, and ``keepalive``. QRMI uses
capabilities/submit/logs and moves their blocking exchanges off its asynchronous
executor; applications using the socket client directly can also call
``keepalive(session_id)``. Successful schema negotiation is shared across client clones
and retained by the QRMI resource. Transport, malformed-response and
incompatible-version failures invalidate it; failed submissions are never automatically
retried. Explicit capability queries always fetch the current catalog. This is not
feature-by-feature preflight validation: Maestro and the server validate every submitted
request. The resource captures one socket path at construction for native requests and
all legacy session/task operations, including acquisition, status, results and cleanup.
``QRMI_MAESTRO_SOCKET`` chooses that socket; the default remains ``/run/maestro.sock``.
Construct a new resource after changing the variable to use a different daemon.

Task runners and examples
-------------------------

Both Rust and Python task runners accept a raw schema-2 request, ``{ "request": ... }``,
or the existing payload wrapper with ``job_type:"request"``. They also retain the legacy
execute/estimate form and accept a configuration object or serialized string. An omitted
or null legacy ``config`` becomes ``{}`` in both runners. Job types are
case-insensitive. A ``{ "request": ... }`` wrapper must contain only that field. Bare
native documents reject legacy envelope fields, and every Maestro form rejects fields
belonging to another backend. The explicitly selected ``job_type:"request"``
compatibility envelope accepts object or serialized JSON ``input`` and permits the known
legacy fields (``qubits``, simulator IDs, ``observables``, ``config``) outside it; those
outer fields are ignored. All native configuration belongs inside ``input``. The two
runners and schema apply these same rules to each form. Legacy jobs require positive
``qubits`` and nonnegative ``simulator_type`` and ``simulation_method``, all within
uint32 range. These forms are covered by ``qrmi_payload_v1_schema.json``. Missing
required Python helper fields raise descriptive ``ValueError``\ s before task
submission. For schema-2 requests, put ``execution.shots`` only on ``execute`` or
``checkpoint_batch``; estimators and other queries reject it. ``execution.seed`` remains
supported. To validate a sampling request, retain its operation and use the native
validation entry point rather than changing it to ``validate``.

Legacy ``execute`` and ``estimate`` also accept ``config.seed`` as a nonnegative
JSON integer up to ``2**64 - 1``. Repeating the seed reproduces sampling on the
same backend/runtime, including when each task runs in a fresh worker.

Both runners retrieve available diagnostics before cleanup on failure/cancellation,
write those diagnostics to stderr, and exit nonzero. Result retrieval and output writing
failures also produce nonzero exits. Missing log support does not hide the original
failure. The Python runner retrieves results before returning, so errors are reflected
in its exit code rather than deferred to an exit callback.

The ready-to-run documents in
:doc:`Maestro task runner examples <examples/task_runner/TASK_RUNNER_MAESTRO>`
include:

-  ``native-noise.json``: exact amplitude damping and observable estimation on CPU.
-  ``native-query.json``: a batch of state and mixed-state queries.
-  ``native-thermal-idle.json``: exact thermal relaxation, elapsed idle time and
   detuning.
-  ``native-correlated-noise.json``: seeded OU phase noise and MPS options.
-  ``native-checkpoint.json``: restore an ideal prefix and execute noisy suffixes.
-  ``native-incremental.json``: repeated exact amplitude-damping Kraus steps.
-  ``native-distributed-gpu.json``: local distribution across two physical GPU ordinals.
-  ``native-mpi-gpu.json``: MPI distribution through an administrator-defined profile.

The GPU documents require the named resources and installed plugins. Set the backend
acquisition token as for existing runner tasks:

.. code:: sh

   export QRMI_JOB_QPU_RESOURCES=MAESTRO_LOCAL
   export QRMI_JOB_QPU_TYPES=maestro-local
   export MAESTRO_LOCAL_QRMI_JOB_ACQUISITION_TOKEN=<existing-session-id>
   task_runner MAESTRO_LOCAL examples/task_runner/maestro_local/native-noise.json

Supported computation and limits
--------------------------------

Native operations include execution, expectation estimation, selected/full amplitudes
and probabilities, state probability, unitary overlap/mirror fidelity, coherent noisy
fidelity, mixed-state diagnostics/partial trace/maintenance, incremental evolution,
checkpoint suffix batches, and independent request batches. All share the native
configuration and declarative noise model. Incremental exact noise uses explicit Kraus
instructions; checkpoint noise applies to suffixes after an ideal prefix. Checkpoint
randomness is reproducible for the complete ordered request: changing/reordering earlier
suffixes may change later samples. Fidelity queries use ideal inverse preparation;
coherent noisy fidelity is not a physical mirror experiment with noise on both halves.
Runtime support can be narrower for a particular method. Operation-specific fields on
other operations are rejected. Native request validation and numerical execution each
run in isolated Rust workers calling Maestro's C ABI; capability catalog queries remain
in the daemon. An outer ``launch`` object is server routing metadata. The server
validates it and removes it before native validation/execution. Direct Maestro C ABI and
``maestro-request`` calls reject ``launch``; use the server for profile selection, or
launch ``maestro-mpi-worker`` yourself with a computational document.

The native API defaults to fixed backend selection and rejects unavailable selections.
Fixed selection preserves reuse across shots with the corrected Maestro library:
eligible circuits evolve once per execution job and sample repeatedly. QRMI forwards
the selection and shot count unchanged; no client-side switch to automatic selection
is needed. Deploy the updated ``libmaestro.so`` on the local server and all configured
native/MPI workers, then restart the server. Updating QRMI alone does not update that
native runtime. The fixed-selection integration checks can include ordinary GPU MPS
and statevector execution by setting ``QRMI_TEST_GPU=1``. The noisy-shot checks
also cover density matrix and MPO: reuse stays within each noise realization,
with independent relaxation-reset outcomes and readout flips on each shot.
Automatic ideal execute/estimate can use explicit candidate lists. Exact quantum
channels require density-matrix/MPO methods; pure-state sampled noise follows Maestro's
existing helper approximations. Distributed GPU engines currently support statevectors.
Noise results identify approximations; an MPO result is not automatically free of
truncation error.

For thermal relaxation with ``T1 < T2 <= 2*T1``, sampled execution uses effective
``T2 = T1``, matching Maestro's Python policy. This includes thermal noise after
one-qubit/two-qubit gates and during delays. The result lists
``thermal_T2_clamped_to_T1`` in ``noise.approximations``, and ``task_logs()`` retains
a warning naming the affected qubits. Exact density-matrix/MPO execution keeps the
calibrated T2. This requires a Maestro library containing the native thermal-policy
update.

Native readout rates follow the measured qubit and apply when each measurement
writes its classical bit, including conditional measurements. Classical control
sees the noisy result; skipped measurements do not apply noise. Quantum-noise
insertion follows the shared Python helpers' behavior on unconditional gates and
delays.

An explicit ``execution.seed`` or ``simulator.options.seed`` controls measurement
and readout randomness. Otherwise, an explicit ``noise.seed`` also seeds the
simulator; when neither is supplied, each request receives a fresh random uint64
seed, returned in its result's ``seed`` field. Pass that value as ``execution.seed``
to replay the request. Zero remains a valid explicit seed. MPI ranks share one
generated seed through their communicator. ``noise.seed`` drives
circuit-noise injection independently and defaults to the simulator seed's lower
32 bits. These changes require an updated Maestro library.

Counts use classical bit 0 first. Pauli strings and target-state strings use qubit 0
first. Integer amplitude/probability indices use qubit 0 as the least significant bit.
Complex entries are ``[real, imaginary]``.

Requests are bounded to 16 MiB and serialized results to 64 MiB; that native bound
covers the whole result document, and the server's ``max_result_bytes`` adds headroom
for MPI metadata. QRMI accepts a reply of that size plus its ``OK`` framing. Use selected
``basis_states`` and ``max_output_elements`` for state queries. Batches have at most 256
members and four nested levels. No interactive state persists across tasks. Composer
orchestration, decoder/Sinter execution, general pure-state Kraus trajectories, new
distributed mixed-state engines, and artifact storage remain outside this change.

Authoritative native field/option/channel details are in Maestro's
``docs/native-request-api.md``; server framing, leases and MPI profile configuration are
in maestro-local-server's ``docs/native-api-v2.md``.

Tests
-----

.. code:: sh

   cargo test --locked --lib maestro::
   cargo test --locked -p maestro-local-api --lib
   cargo test --locked --features build-binary --bin task_runner
   python -m pytest python/tests/unit/quantum_resource/test_maestro_request.py \
       python/tests/unit/quantum_resource/test_maestro_local.py \
       python/tests/unit/quantum_resource/test_maestro_runner.py

Use the rebuilt Python extension. Set ``QRMI_TEST_LIBRARY`` to a matching QRMI shared
library to enable C ABI checks. For end-to-end tests, set ``QRMI_TEST_MAESTRO_SERVER``
to a freshly built native daemon, select its matching Maestro library with
``LD_LIBRARY_PATH``, and run ``test_maestro_native_integration.py`` in that directory.
Tests start a private socket and daemon. Set ``QRMI_TEST_MAESTRO_REQUEST`` to the
matching ``maestro-request`` executable to validate all native example documents. CPU
examples are also executed through QRMI and checked numerically; hardware examples
require separate execution resources. For CPU MPI tests, ``QRMI_TEST_MPI_PROFILE`` names
a configured two-rank profile; set ``MAESTRO_MPI_PROFILES`` to its profile file for that
daemon. Set ``QRMI_TEST_DISTRIBUTED_GPU=1`` with a working licensed plugin/runtime to
include ideal and noisy local GPU distribution through QRMI. No system service is
restarted. Set ``QRMI_TEST_MPI_GPU_PROFILE`` to a profile with working rank-local GPU
binding, the MPI GPU plugin/license and CUDA-aware MPI to run ideal and noisy
``distributed_mpi_gpu`` cases. Those tests assert the actual backend and rank count; the
CPU MPI test alone does not verify GPU distribution. Set
``QRMI_REQUIRE_MAESTRO_NATIVE=1`` to make a missing daemon or ``maestro-request``
fail these tests instead of skipping them. The Maestro local server's CI does this
on CPU: it builds the server against its pinned Maestro and runs this suite from a
pinned QRMI commit, which needs no stored credentials. The live legacy suite in
``python/tests/integration/quantum_resource/test_maestro_live.py`` requires
``QRMI_TEST_MAESTRO_LIVE=1`` and skips backend cases absent from the capability catalog.

Current tensor requests and option migration
--------------------------------------------

Native schema version 2 identifies the document format, not a Maestro release.
After an upgrade, inspect ``json.loads(resource.target().value)`` again. The
``native.options`` catalog lists current names, types, applicability and, on newer
builds, ``enum`` choices, bounds and ``supported_configurations``. Conditional
``exceptions`` override general bounds (MPO accepts zero for unlimited bond
size); ``constraints`` narrow choices for particular backends. Validation remains
authoritative, including cross-field rules and output limits.

The following native JSON options have changed:

.. list-table:: Native option migration
   :header-rows: 1
   :widths: 45 55

   * - Previous option
     - Current option
   * - ``use_double_precision``
     - ``precision: "double"`` or ``"single"``
   * - ``mps_measure_no_collapse`` / ``mps_sample_measure_algorithm``
     - ``mps_sampling: "probabilities"`` or ``"apply_measure"``
   * - ``mps_use_gesvd*``, ``mpo_use_gesvd*``, ``tensor_network_use_gesvd*``
     - Corresponding ``*_svd_solver`` with ``gesvd``, ``gesvdj``, ``gesvdp`` or ``gesvdr``
   * - ``pp_pauli_weight_threshold``
     - ``pp_max_pauli_weight``
   * - ``pp_steps_between_trims``
     - ``pp_gates_between_trims``
   * - ``pp_steps_between_deduplications``
     - ``pp_gates_between_deduplications``

For boolean sampling migration, true becomes ``probabilities`` and false becomes
``apply_measure``. The former sampling strings ``mps_probabilities`` and
``mps_apply_measure`` lose their prefix. Select the SVD solver whose former flag
was true; omit the selector if all flags were false. Do not send both old and new
keys. QCSim computes in double precision; precision/SVD selections apply only to
backends advertised by the catalog.

Newer Maestro libraries add the following options. Their ranges come from the
catalog entries; per-backend limits appear as ``constraints``:

.. list-table:: Newer native options
   :header-rows: 1
   :widths: 35 25 40

   * - Option(s)
     - Applies to
     - Values and constraints
   * - ``pp_workers``
     - QCSim and GPU Pauli propagation
     - QCSim 0–1024; GPU 1–64 (default 1). ``pp_workers: 0`` is rejected on GPU.
   * - ``pp_sampling_cache_nodes``
     - QCSim and GPU Pauli propagation
     - Nonnegative; GPU default 8192, zero disables caching
   * - ``pp_memory_limit_bytes``
     - GPU Pauli propagation
     - Positive budget, default 90% of free GPU memory; a limit, not a reservation
   * - ``pp_max_fused_gates``
     - GPU Pauli propagation
     - 1–256, default 64; one disables fusion
   * - ``mps_routing_strategy``
     - QCSim MPS/MPO
     - ``legacy`` (default) or opt-in ``combined``
   * - ``mps_routing_virtual_bonds``, ``mps_routing_truncation_weight``,
       ``mps_routing_legacy_cap``
     - QCSim and GPU MPS/MPO
     - Experimental virtual-bond routing
   * - ``auto_reduce``, ``enable_causal_cone_reduction``,
       ``enable_observable_clustering``
     - All
     - Booleans, default true
   * - ``min_cluster_qubits``, ``max_cluster_qubits``
     - All
     - Positive; the minimum must not exceed the maximum
   * - ``observable_error_tolerance``
     - All
     - In (0, 1]
   * - ``allow_reset_marginalization``, ``enable_sampling_reduction``
     - All
     - Booleans, default false
   * - ``trajectory_policy``, ``trajectory_max_memory_mb``
     - QCSim statevector/density matrix (policy also path integral)
     - ``block_v1`` or ``shot_v1`` (default); positive MiB count

An option for another backend or method raises ``UnsupportedFunctionError`` and an
out-of-range value raises ``InvalidInputError``, before task allocation.
Validation accepts GPU options without a GPU; run a small request on each deployed
GPU backend, with ``QRMI_TEST_GPU=1`` for the opt-in numerical checks. Combined and
virtual-bond routing trade runtime against truncation at finite bond caps. Sampling
reduction changes a seeded run's random streams. Maestro 0.3.10 made
``coherent_depolarizing`` isotropic, so its results differ from earlier libraries;
QASM accepts qelib1.inc's ``rzz``, ``rxx`` and ``u0``.

QRMI does not translate these options. Removed keys fail native validation before
allocation of a task ID. Legacy ``SET_OPTIONS`` uses a separate contract; keys
accepted there are not necessarily valid under native ``simulator.options``.

The ``examples/task_runner/maestro_local`` directory contains complete native
requests suitable for either task runner or ``request_payload``:

* ``native-mps-operators.json`` evaluates ordered, read-only MPS operators and
  exercises routing and maintenance. Repeated targets retain their order: X then
  Y on qubit zero of the zero state returns ``[0, -1]`` because YX = -iZ.
* ``native-mpo-operators.json`` applies an operator to an MPO and compares raw and
  normalized complex expectations and dense matrices. ``normalize: false`` keeps
  the scale of A rho A-dagger; these operators change the state.
* ``native-tensor-expectations.json`` evaluates repeated, asymmetric Pauli
  observables in a single estimate request.
* ``native-tensor-batch.json`` preserves repeated observable order over successive
  evolution steps. Native batching shares prepared-state work; QRMI should not
  split these observables into independent jobs.
* ``native-tensor-bulk.json`` reads amplitudes in logical basis order and returns
  ``execution_metadata.gate_fusion`` alongside the numerical results.
* ``native-mpo-wide.json`` queries a 65-qubit product state using a bitstring.
  Wide measurement results also retain string keys rather than integer conversion.
* ``native-adjoint.json`` computes adjoint Jacobians for two parameter rows, with a
  Pauli string, a weighted Pauli sum and a probability readout.
* ``native-mirror-observables.json`` measures MPS truncation per observable with
  ``mirror_fidelity``; lowering ``max_bond_dimension`` to 1 makes values drop below 1.
* ``native-instructions.json`` uses instruction-array ``rzz``, ``noiseless``
  basis changes, a ``pauli_channel`` and per-measurement ``readout_error``.
* ``native-mixed-state.json`` batches ``fidelity_with_statevector`` of a noisy
  Bell state and ``density_matrix_overlap`` of two MPO preparations.

Adjoint differentiation, observable mirror fidelity and these instruction fields
need a Maestro library listing them in ``native.operations``, ``native.adjoint``
and ``native.instructions``; older libraries reject them before task allocation.
They use the existing ``request`` payload unchanged:

- ``adjoint_expectation_values``, ``adjoint_jacobian`` and ``adjoint_vjp``
  differentiate a unitary circuit on the QCSim CPU statevector. ``observables``
  entries are Pauli strings, ``{"terms": [[coefficient, pauli], ...]}`` or
  ``{"probabilities": [qubits]}``; ``parameter_rows`` replaces the circuit's
  angles row by row, and ``adjoint_vjp`` takes ``cotangents[row][value]``.
  Results report ``num_parameters``, ``num_values`` and ``circuit_parameters``
  (the circuit's own angles in parameter order) with ``expectation_values``,
  ``jacobian[row][value][parameter]`` or ``vjp[row][parameter]``. They carry no
  seed.
- ``mirror_fidelity`` with ``observables`` returns their expectations on the
  executed mirror state instead of ``fidelity``. Unlike plain mirror fidelity, it
  also runs on ``density_matrix`` and ``matrix_product_operator``.
- Instruction arrays accept ``rxx``/``rzz``, ``"noiseless": true`` on gates,
  ``readout_error`` on ``measure`` (a probability or ``{p_meas1_prep0,
  p_meas0_prep1}``; counts only, so operations that remove terminal measurements,
  including the adjoint operations, nested ``other_circuit``/``step_circuit`` and
  the ``checkpoint_batch`` prefix, refuse it with ``UnsupportedFunctionError``;
  checkpoint suffix measurements apply it), and ``pauli_channel`` with ``4**len(qubits)``
  ``probabilities``. Mixed-state methods apply the channel exactly; pure-state
  methods sample it per noise realization and need a trajectory ``noise`` block,
  whose ``channels`` may be omitted.
- ``density_matrix_overlap`` returns ``overlap``, the complex ``Tr(rho sigma)`` of
  ``circuit`` and ``other_circuit`` prepared separately, and
  ``fidelity_with_statevector`` returns ``fidelity``, ``<psi|rho|psi> / Tr(rho)``
  for ``target_statevector``: ``2**num_qubits`` ``[real, imaginary]`` amplitudes in
  basis-integer order, normalized to within ``1e-6``, for at most 20 qubits. They
  are the mixed-state counterparts of ``inner_product`` and ``noisy_fidelity`` and
  run only with fixed ``qcsim`` or ``gpu`` selection on ``density_matrix`` or
  ``matrix_product_operator``; other methods refuse them with
  ``UnsupportedFunctionError``. Circuits may contain resets and channels; a ``noise``
  model must use exact evaluation with one realization. The catalog lists them in
  ``native.mixed_state_queries``.

Pauli strings put qubit zero first; Qiskit Pauli labels require reversal when
constructing these native observables. Basis integers put qubit zero in the least
significant bit. Test asymmetric states/observables when converting conventions.
Complex results use ``[real, imaginary]``. Full vectors and matrices still grow
exponentially: bounded GPU staging memory does not remove native output limits
or the 64 MiB native result bound.

Resource status and deployment discovery
----------------------------------------

``resource.status()`` uses the additive server API v2 ``status`` command on a
blocking executor. It creates no session and does not renew any session lease.
It reports Online when the runner is live, its state is usable and shutdown has
not begun; otherwise it reports Offline with a reason. ``healthy`` describes the
runner, ``busy`` means active or queued execution, and ``pending_job_count`` counts
waiting jobs, excluding the active job. These snapshots can change immediately.

Capacity remains unspecified: an execution worker is not an exclusive acquisition
slot. Runner health does not certify GPU availability, licensing or plugin
compatibility. Older servers with an unsupported status command fall back to
``PING`` with health, occupancy and queue length unknown. Transport errors,
malformed replies and other server failures remain errors.

New catalogs include ``server.build`` and ``native.build`` with version and source
revision, plus ``native.diagnostics`` with supported backend/method pairs.
``native.capability_scope: "validation"`` distinguishes parser support from
runtime readiness. Missing optional GPU exports can still make a validated job
fail. Older catalogs may omit these additive fields.

Rebuild and deploy compatible Maestro, GPU plugin and server artifacts together,
then restart the service to load them. Install both Maestro's ``Interface.h`` and
``InterfaceTypes.h``. Build identifiers describe the build inputs (the native
revision is captured at CMake configuration); source archives may report
``unknown``, and modified tracked sources carry a ``-dirty`` suffix. They are
informational, not a feature-negotiation mechanism. QRMI forwards catalog and
result additions without a new payload type or C ABI change.
