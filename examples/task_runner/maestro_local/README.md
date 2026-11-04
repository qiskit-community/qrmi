# Tools to generate task runner input for Maestro Local

The tools demonstrate the generation of `task_runner` input from a Qiskit circuit,
targeting a Maestro Local resource.

## Prerequisites
* Python 3.11 or above

## Install dependencies

```shell-session
pip install -r requirements.txt
```

## Tools

### gen_sampler_input.py

Builds a GHZ-state circuit, converts it to QASM, and writes a `task_runner`
input file for an `execute` job (i.e. measurement counts).

Usage:
```shell-session
usage: gen_sampler_input.py [-h] [--qubits QUBITS] [--shots SHOTS]
                             [--simulator-type SIMULATOR_TYPE]
                             [--simulation-method SIMULATION_METHOD]
                             [-o OUTPUT]

options:
  -h, --help            show this help message and exit
  --qubits QUBITS       Number of qubits
  --shots SHOTS         Number of shots
  --simulator-type SIMULATOR_TYPE
                        Simulator type (0 = aer, 1 = qcsim)
  --simulation-method SIMULATION_METHOD
                        Simulation method (0 = statevector, 1 = matrix_product_state)
  -o OUTPUT, --output OUTPUT
                        Output filename
```

Example:
```bash
python gen_sampler_input.py --qubits 3 -o sampler_input.json
```

### gen_estimator_input.py

Builds a Bell-state circuit and a pair of Pauli observables (`ZZ`, `XX`),
converts the circuit to QASM, and writes a `task_runner` input file for an
`estimate` job (i.e. expectation values).

Usage:
```shell-session
usage: gen_estimator_input.py [-h] [--simulator-type SIMULATOR_TYPE]
                               [--simulation-method SIMULATION_METHOD]
                               [-o OUTPUT]
```

Example:
```bash
python gen_estimator_input.py -o estimator_input.json
```

## Output

Both tools write a single JSON file matching the `maestro-local` entry of the
[QRMI payload schema](../../../qrmi_payload_v1_schema.json):

| Field | Description |
| ---- | ---- |
| `input` | QASM string for the circuit |
| `job_type` | `execute` or `estimate` |
| `qubits` | Number of qubits |
| `simulator_type` | Simulator type (0 = aer, 1 = qcsim) |
| `simulation_method` | Simulation method (0 = statevector, 1 = matrix_product_state) |
| `observables` | Pauli strings separated by `;` (required for `estimate`) |
| `config` | Task configuration, as a JSON-formatted string (e.g. `{"shots": 1000}`) |

## Running with task_runner

```shell-session
export QRMI_JOB_QPU_RESOURCES=MAESTRO_LOCAL
export QRMI_JOB_QPU_TYPES=maestro-local
export MAESTRO_LOCAL_QRMI_JOB_ACQUISITION_TOKEN=<existing session ID>

task_runner MAESTRO_LOCAL sampler_input.json
```

See the [task_runner documentation](../../../python/qrmi/tools/task_runner/README.md)
for more details.

## Native request API

The `native-*.json` examples use the new schema-2 native API. They require the
updated Maestro library, native server, and QRMI. Both task runners accept these
files directly. See [the native API guide](../../../docs/maestro-native-api.rst)
for noise semantics, query batches, GPU prerequisites, and MPI profile setup.
Legacy integer simulator IDs above depend on the Maestro build; query `target()`
for the actual mapping, or use symbolic names in native requests.

| Example | What it demonstrates | Expected result / prerequisites |
|---|---|---|
| [native-noise.json](native-noise.json) | Exact T1 damping and estimation | Expectations `[0.8, 0.36]` on CPU. |
| [native-query.json](native-query.json) | Independent amplitude and mixed-state diagnostic queries | Selected amplitudes `[1, 0]`, purity `0.8848`. |
| [native-thermal-idle.json](native-thermal-idle.json) | Thermal gate relaxation plus delay relaxation and detuning | Exact density-matrix expectations; durations are seconds, detuning is Hz. |
| [native-correlated-noise.json](native-correlated-noise.json) | Correlated OU phase noise with 128 seeded realizations, MPS options | Repeatable estimates within the same runtime; reported standard errors describe realization variation. |
| [native-checkpoint.json](native-checkpoint.json) | Save an ideal prefix and run two noisy suffixes | Both suffixes return `{"1":32}`; the prefix receives no noise. |
| [native-incremental.json](native-incremental.json) | Prepare once and apply an exact amplitude-damping Kraus step repeatedly | Z expectations `[-1, -0.5, -0.125, 0.3671875]` at steps `[0,1,2,4]`. |
| [native-distributed-gpu.json](native-distributed-gpu.json) | Local statevector distribution | Two physical GPU ordinals plus the plugin, runtime and license. |
| [native-mpi-gpu.json](native-mpi-gpu.json) | MPI GPU statevector distribution | Configure the named server profile, rank-local GPU binding, MPI plugin/license and CUDA-aware MPI. |

Native estimators and queries omit `execution.shots`; only `execute` and
`checkpoint_batch` accept it. Checkpoint suffixes are circuit objects and cannot
contain `launch` or their own simulator configuration. Noise seeds apply to the
complete ordered checkpoint request, so reordering suffixes can change samples.
The incremental example uses a Kraus channel with damping probability 0.25 per
step, including the skipped intermediate step when advancing from step 2 to 4.

With the existing session environment shown above, run a CPU example using either
task runner:

```sh
task_runner MAESTRO_LOCAL native-thermal-idle.json
task_runner MAESTRO_LOCAL native-incremental.json
```

From this directory, standalone native validation is also available for CPU files:

```sh
maestro-request --validate < native-checkpoint.json
```

The MPI example's `launch` field is interpreted by the local server and is rejected
by the standalone CLI. Tests validate all eight computational documents and run
the six CPU examples through a private native server with result assertions. GPU
validation does not establish device, allocation or license availability.

## Current tensor examples

The following native schema 2 documents work through `request_payload(document)`
and either task runner, using symbolic backend/method names:

| File | Purpose |
| --- | --- |
| `native-mps-operators.json` | Ordered MPS operator expectation, routing and maintenance |
| `native-mpo-operators.json` | MPO state updates and normalized/raw complex results |
| `native-tensor-expectations.json` | Batched estimates with repeated asymmetric observables |
| `native-tensor-batch.json` | Repeated observables at successive evolution steps |
| `native-tensor-bulk.json` | Bulk amplitudes and gate-fusion metadata |
| `native-mpo-wide.json` | A 65-qubit MPO probability using a bitstring |

## Differentiation, mirror and instruction examples

These need a Maestro library that lists `adjoint_jacobian` in
`native.operations` and an `instructions` catalog, and for the mixed-state
queries a `mixed_state_queries` catalog; older libraries reject them before
allocating a task.

| File | Purpose | Expected result |
| --- | --- | --- |
| [native-adjoint.json](native-adjoint.json) | Adjoint Jacobian for two parameter rows, with Pauli, weighted-sum and probability readouts | Four values and a 4×2 Jacobian per row; the all-zero row gives values `[1, 0.5, 1, 0]`. |
| [native-mirror-observables.json](native-mirror-observables.json) | Per-observable mirror fidelity of a GHZ-like MPS | `[1, 1, 1, 1]`; set `max_bond_dimension` to 1 to see truncation. |
| [native-instructions.json](native-instructions.json) | Instruction-array `rzz`, `noiseless` basis changes, a `pauli_channel` and `readout_error` on a density matrix | Counts `{"01": 32}`. On `statevector`, add `"noise": {"realizations": 32}` to sample the channel. |
| [native-mixed-state.json](native-mixed-state.json) | `fidelity_with_statevector` of a noisy Bell state on a density matrix, and `density_matrix_overlap` of two MPO preparations | Fidelity `0.7624` against the ideal Bell state; overlap `[0.5, 0]` for `|+>` and `|1>`. |

These use QCSim by default. GPU support depends on matching Maestro/plugin builds
and runtime availability. Consult `target()` after upgrades: schema version 2
alone does not identify supported options. See the
[native API guide](../../../docs/maestro-native-api.rst) for option migration,
ordering conventions, output limits and resource-status semantics.
