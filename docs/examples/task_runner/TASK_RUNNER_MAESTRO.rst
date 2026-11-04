.. _task_runner_maestro:

Maestro Local examples
======================

Maestro Local supports legacy QASM execute/estimate payloads and native schema-2
requests. Native requests expose symbolic simulator selection, noise models,
state queries, incremental evolution, checkpoint batches and GPU distribution.
The :doc:`native request guide <../../maestro-native-api>` describes configuration,
results, diagnostics and supported operations.

Setup and execution
-------------------

Build and install QRMI. Install `maestro-local-server <https://github.com/QoroQuantum/maestro-local-server>`__ and the Maestro
library it uses by following the server's `installation guide <https://github.com/QoroQuantum/maestro-local-server/blob/main/docs/installation.md>`__, which uses a
prebuilt Maestro release. QRMI needs no
optional Python extra for native requests. Python is an optional client; the
server and workers use Rust/C/C++.

From the QRMI checkout, with an already acquired Maestro session:

.. code-block:: sh

   export QRMI_JOB_QPU_RESOURCES=MAESTRO_LOCAL
   export QRMI_JOB_QPU_TYPES=maestro-local
   export MAESTRO_LOCAL_QRMI_JOB_ACQUISITION_TOKEN=<existing-session-id>
   # Set QRMI_MAESTRO_SOCKET if the daemon does not use /run/maestro.sock.
   task_runner MAESTRO_LOCAL examples/task_runner/maestro_local/native-noise.json

Both the Rust and Python task runners accept these files directly. The runner
uses the supplied session; its owner should release that session after the
workload finishes.

Exact amplitude damping
-----------------------

This CPU example estimates X and Z after exact T1 damping, giving expectation
values ``[0.8, 0.36]``. It needs the QCSim density-matrix backend.

.. literalinclude:: ../../../examples/task_runner/maestro_local/native-noise.json
   :language: json
   :caption: native-noise.json

Downloadable native requests
----------------------------

.. list-table:: Native examples
   :header-rows: 1
   :widths: 28 32 40

   * - Input
     - Feature
     - Expected result or prerequisite
   * - :download:`native-noise.json <../../../examples/task_runner/maestro_local/native-noise.json>`
     - Exact amplitude damping
     - CPU expectations ``[0.8, 0.36]``.
   * - :download:`native-query.json <../../../examples/task_runner/maestro_local/native-query.json>`
     - Amplitudes and mixed-state diagnostics
     - Selected amplitudes ``[1, 0]`` and purity ``0.8848``.
   * - :download:`native-thermal-idle.json <../../../examples/task_runner/maestro_local/native-thermal-idle.json>`
     - Thermal gate/idle relaxation and detuning
     - Density matrix; durations in seconds and detuning in Hz.
   * - :download:`native-correlated-noise.json <../../../examples/task_runner/maestro_local/native-correlated-noise.json>`
     - Seeded correlated OU noise and MPS configuration
     - 128 realizations with reported standard errors.
   * - :download:`native-checkpoint.json <../../../examples/task_runner/maestro_local/native-checkpoint.json>`
     - Noisy suffixes after an ideal prefix
     - Both suffixes return ``{"1": 32}``.
   * - :download:`native-incremental.json <../../../examples/task_runner/maestro_local/native-incremental.json>`
     - Repeated exact Kraus steps
     - Z expectations ``[-1, -0.5, -0.125, 0.3671875]`` at steps ``[0, 1, 2, 4]``.
   * - :download:`native-distributed-gpu.json <../../../examples/task_runner/maestro_local/native-distributed-gpu.json>`
     - Local distributed statevector
     - Two physical GPU ordinals and the matching plugin, runtime and license.
   * - :download:`native-mpi-gpu.json <../../../examples/task_runner/maestro_local/native-mpi-gpu.json>`
     - MPI distributed GPU statevector
     - An administrator-defined server profile, rank-local GPU binding,
       MPI GPU plugin/license and CUDA-aware MPI.

Estimators and queries omit ``execution.shots``; only ``execute`` and
``checkpoint_batch`` accept it. Checkpoint noise applies to suffixes after an
ideal prefix. Seeds describe the complete ordered checkpoint request, so changing
earlier suffixes can change later samples. The incremental example applies its
Kraus channel at every step, including unobserved intermediate steps.

For MPI requests, adapt the ``launch`` profile name to the server configuration.
``launch`` is server routing metadata and is rejected by the standalone
``maestro-request`` validator. See the native guide for validation and execution
tests, including the separate opt-ins for local GPU and MPI GPU tests.

Other Maestro examples
----------------------

Legacy QASM generator scripts and their requirements are in the
`task runner example directory <https://github.com/qiskit-community/qrmi/tree/main/examples/task_runner/maestro_local>`_.
Legacy numeric simulator IDs depend on the Maestro build; consult ``target()``
for the installed catalog or use symbolic names in native requests.

The repository also includes `Rust <https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/rust/maestro_local>`_,
`Python <https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/python/maestro_local>`_,
`C <https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/c/maestro_local>`_,
`Lua <https://github.com/qiskit-community/qrmi/tree/main/examples/qrmi/lua/maestro>`_
and `Qiskit primitive <https://github.com/qiskit-community/qrmi/tree/main/examples/qiskit_primitives/maestro_local>`_
examples. The Qiskit examples require Qiskit to construct circuits.
