# This code is part of Qiskit.
#
# (C) Copyright Qoro Quantum 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Opt-in QRMI -> isolated native daemon -> C++ feature tests.

Set QRMI_TEST_MAESTRO_SERVER to the rebuilt daemon, and its native library search
path in LD_LIBRARY_PATH. The test harness starts no Python server or worker.
QRMI_TEST_MAESTRO_REQUEST names the native maestro-request executable. With
QRMI_REQUIRE_MAESTRO_NATIVE=1, as in CI, a missing binary fails instead of skipping.
"""

import copy
import importlib.util
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time

import pytest

from qrmi import (
    QuantumResource,
    ResourceType,
    TaskStatus,
    InvalidInputError,
    ConfigError,
    UnsupportedFunctionError,
    QrmiError,
)
from qrmi.maestro import payload_from_input, request_payload

EXAMPLES = Path(__file__).resolve().parents[4] / "examples/task_runner/maestro_local"


def _load_native_examples():
    spec = importlib.util.spec_from_file_location(
        "gen_native_input", EXAMPLES / "gen_native_input.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.NATIVE_EXAMPLES


NATIVE_EXAMPLES = _load_native_examples()


def example_doc(name: str):
    """Return a fresh copy of a named native request example."""
    return copy.deepcopy(NATIVE_EXAMPLES[name])


def required_binary(variable, reason):
    """A configured native binary; required runs fail rather than skip without it."""
    path = os.environ.get(variable)
    if not path:
        if os.environ.get("QRMI_REQUIRE_MAESTRO_NATIVE") == "1":
            pytest.fail(f"QRMI_REQUIRE_MAESTRO_NATIVE=1 but {variable} is unset")
        pytest.skip(reason)
    return path


@pytest.mark.parametrize("name", sorted(NATIVE_EXAMPLES))
def test_example_native_validation(name):
    """Validate every example's computational document with the native CLI."""
    worker = required_binary(
        "QRMI_TEST_MAESTRO_REQUEST",
        "set QRMI_TEST_MAESTRO_REQUEST to the native maestro-request executable",
    )
    document = example_doc(name)
    # Server profile selection is tested separately; the C ABI accepts only the
    # computational document, including for hardware-dependent examples.
    document.pop("launch", None)
    result = subprocess.run(
        [worker, "--validate", "--framed-result"],
        input=json.dumps(document),
        text=True,
        capture_output=True,
        timeout=15,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    frames = [
        line.removeprefix("MAESTRO_RESULT_V2 ")
        for line in result.stdout.splitlines()
        if line.startswith("MAESTRO_RESULT_V2 ")
    ]
    assert len(frames) == 1
    assert json.loads(frames[0])["ok"] is True


@pytest.mark.parametrize(
    "name",
    ["noise", "query", "thermal-idle", "correlated-noise", "checkpoint", "incremental"],
)
def test_cpu_examples_through_qrmi(native_resource, name):
    """Check numerical results from the CPU examples through the full stack."""
    document = example_doc(name)
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    if name == "noise":
        assert result["expectation_values"] == pytest.approx([0.8, 0.36])
    elif name == "query":
        assert result["results"][0]["amplitudes"] == [[1, 0], [0, 0]]
        assert result["results"][1]["purity"] == pytest.approx(0.8848)
    elif name == "thermal-idle":
        expected = [
            math.exp(-0.4 / 0.6) * math.cos(2 * math.pi * 0.25 * 0.3),
            0.8 * (1 - math.exp(-0.4)),
        ]
        assert result["expectation_values"] == pytest.approx(expected, abs=1e-9)
    elif name == "correlated-noise":
        assert -1 <= result["expectation_values"][0] < 0.99
        assert result["expectation_values"][1] == pytest.approx(0, abs=1e-9)
        repeated = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, repeated) == TaskStatus.Completed
        assert json.loads(native_resource.task_result(repeated).value)[
            "expectation_values"
        ] == pytest.approx(result["expectation_values"], abs=1e-12)
    elif name == "checkpoint":
        assert result["noise_scope"] == "suffix_only_ideal_prefix"
        assert [suffix["counts"] for suffix in result["results"]] == [
            {"1": 32},
            {"1": 32},
        ]
    elif name == "incremental":
        assert [values[0] for values in result["expectation_values"]] == pytest.approx(
            [-1, -0.5, -0.125, 0.3671875]
        )


@pytest.mark.parametrize("backend", ["qcsim", "gpu"])
@pytest.mark.parametrize("method", ["statevector", "matrix_product_state"])
@pytest.mark.parametrize("selection", [None, "fixed"])
def test_fixed_selection_multiple_shots(native_resource, backend, method, selection):
    """Forward fixed/default selection without changing the method or shot count."""
    if backend == "gpu" and os.environ.get("QRMI_TEST_GPU") != "1":
        pytest.skip("set QRMI_TEST_GPU=1 with an available GPU plugin")
    document = native_request("execute", method)
    document["circuit"] = {
        "num_qubits": 2,
        "num_clbits": 2,
        "source": (
            "OPENQASM 2.0; qreg q[2]; creg c[2]; " "h q[0]; cx q[0],q[1]; measure q->c;"
        ),
    }
    document["simulator"]["backend"] = backend
    if selection is not None:
        document["simulator"]["selection"] = selection
    document["execution"]["shots"] = 4096
    previous = None
    for _ in range(2):
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        result = json.loads(native_resource.task_result(task).value)
        assert result["execution_metadata"]["backend"] == backend
        assert result["execution_metadata"]["method"] == method
        assert result["execution_metadata"]["selection"] == "fixed"
        assert result["shots"] == 4096
        assert set(result["counts"]) == {"00", "11"}
        assert sum(result["counts"].values()) == 4096
        assert min(result["counts"].values()) > 1024
        if previous is not None:
            assert result["counts"] == previous
        previous = result["counts"]


@pytest.mark.parametrize("backend", ["qcsim", "gpu"])
@pytest.mark.parametrize(
    "method",
    [
        "statevector",
        "matrix_product_state",
        "density_matrix",
        "matrix_product_operator",
    ],
)
def test_fixed_noisy_shots_remain_independent(native_resource, backend, method):
    """Relaxation and readout must not freeze one outcome across reused shots."""
    if backend == "gpu" and os.environ.get("QRMI_TEST_GPU") != "1":
        pytest.skip("set QRMI_TEST_GPU=1 with an available GPU plugin")
    document = native_request("execute", method)
    document["simulator"]["backend"] = backend
    document["simulator"]["selection"] = "fixed"
    document["circuit"] = {
        "num_qubits": 2,
        "num_clbits": 2,
        "source": (
            "OPENQASM 2.0; qreg q[2]; creg c[2]; " "h q[0]; cx q[0],q[1]; measure q->c;"
        ),
    }
    document["execution"]["shots"] = 4096
    document["noise"] = {
        "realizations": 1,
        "seed": 23,
        "channels": [
            {"kind": "t1_2q", "targets": [0], "gamma": 1.0},
            {"kind": "readout", "targets": [0], "probability": 0.5},
        ],
    }
    previous = None
    for _ in range(2):
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        result = json.loads(native_resource.task_result(task).value)
        assert result["execution_metadata"]["backend"] == backend
        assert result["execution_metadata"]["method"] == method
        assert result["execution_metadata"]["selection"] == "fixed"
        assert result["noise"]["realizations"] == 1
        assert set(result["counts"]) == {"00", "01", "10", "11"}
        assert sum(result["counts"].values()) == 4096
        assert all(700 < count < 1350 for count in result["counts"].values())
        if previous is not None:
            assert result["counts"] == previous
        previous = result["counts"]


@pytest.fixture(name="native_resource")
def fixture_native_resource(tmp_path, monkeypatch, request):
    """Start a private native daemon and clean up its acquired session."""
    binary = required_binary(
        "QRMI_TEST_MAESTRO_SERVER",
        "set QRMI_TEST_MAESTRO_SERVER to enable native end-to-end tests",
    )
    socket = str(tmp_path / "maestro.sock")
    monkeypatch.setenv("QRMI_MAESTRO_SOCKET", socket)
    monkeypatch.delenv("native_test_QRMI_JOB_ACQUISITION_TOKEN", raising=False)
    env = {**os.environ, "MAESTRO_SOCKET": socket}
    scenario = getattr(request, "param", None)
    if scenario in {"invalid_profiles", "missing_scancel", "double_failure"}:
        profiles = tmp_path / "profiles.json"
        profiles.write_text(
            "{"
            if scenario == "invalid_profiles"
            else json.dumps(
                {
                    "version": 1,
                    "profiles": {
                        "broken": {
                            "kind": "srun",
                            "launcher": (
                                "/bin/sh"
                                if scenario == "double_failure"
                                else "/bin/true"
                            ),
                            "worker": "/bin/true",
                            "scancel": (
                                "/bin/false"
                                if scenario == "double_failure"
                                else str(tmp_path / "missing-scancel")
                            ),
                            "args": (
                                [
                                    "-c",
                                    "exec 0<&-; echo 'MAESTRO_STEP_V2 123.4'; "
                                    'echo \'MAESTRO_RESULT_V2 {"ok":false,"error":'
                                    '{"code":"native_test_failure",'
                                    '"message":"original native cause"}}\'; '
                                    "exec sleep 30",
                                ]
                                if scenario == "double_failure"
                                else []
                            ),
                            "max_ranks": 2,
                        }
                    },
                }
            )
        )
        env["MAESTRO_MPI_PROFILES"] = str(profiles)
    if scenario == "short_lease":
        env["MAESTRO_SESSION_LEASE_SECONDS"] = "1"
    with (
        (tmp_path / "daemon.log").open("w+") as log,
        subprocess.Popen([binary], env=env, stdout=log, stderr=log) as process,
    ):
        resource = None
        token = None
        try:
            deadline = time.monotonic() + 10
            while not Path(socket).exists():
                assert process.poll() is None, "Native daemon exited at startup"
                assert time.monotonic() < deadline, "Native daemon startup timed out"
                time.sleep(0.02)
            resource = QuantumResource("native_test", ResourceType.MaestroLocal)
            # bind() creates the socket before listen() makes it ready. Retry
            # only connection refusal: no session could have been allocated.
            while True:
                try:
                    token = resource.acquire()
                    break
                except QrmiError as error:
                    if "Connection refused" not in str(error):
                        raise
                    assert process.poll() is None, "Native daemon exited at startup"
                    assert (
                        time.monotonic() < deadline
                    ), "Native daemon startup timed out"
                    time.sleep(0.02)
            yield resource
        finally:
            try:
                if token is not None:
                    resource.release(token)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


def native_request(operation="estimate", method="density_matrix"):
    """Build a seeded one-qubit request with operation-specific fields."""
    return {
        "schema_version": 2,
        "operation": operation,
        "circuit": {
            "num_qubits": 1,
            "source": "OPENQASM 2.0;\n// preserved comment\nqreg q[1]; creg c[1]; h q[0]; measure q->c;",
        },
        "simulator": {"backend": "qcsim", "method": method},
        **({"observables": ["X", "Z"]} if operation == "estimate" else {}),
        "execution": {
            "seed": 123,
            **({"shots": 16} if operation in {"execute", "checkpoint_batch"} else {}),
        },
    }


def await_task(resource, task):
    """Wait at most thirty seconds for native execution to finish or fail."""
    deadline = time.monotonic() + 30
    while True:
        status = resource.task_status(task)
        if status in (TaskStatus.Completed, TaskStatus.Failed):
            return status
        assert time.monotonic() < deadline, "Native task timed out"
        time.sleep(0.02)


@pytest.mark.parametrize("noisy", [False, True])
@pytest.mark.parametrize("mpi", [False, True])
def test_omitted_seeds_are_random_and_replay(native_resource, noisy, mpi):
    """Replay CPU/MPI sampling from reported and retained random seeds."""
    document = native_request("execute", "statevector")
    if mpi:
        profile = os.environ.get("QRMI_TEST_MPI_GPU_PROFILE")
        if not profile:
            pytest.skip("set QRMI_TEST_MPI_GPU_PROFILE to test native MPI seeds")
        document["simulator"] = {
            "backend": "distributed_mpi_gpu",
            "distribution": {"backend": "ex"},
        }
        document["launch"] = {"profile": profile, "ranks": 2}
    document["circuit"] = {
        "num_qubits": 4,
        "source": "OPENQASM 2.0; qreg q[4]; creg c[4]; h q; measure q->c;",
    }
    document["execution"] = {"shots": 4096}
    if noisy:
        document["noise"] = {
            "realizations": 16,
            "channels": [{"kind": "readout", "targets": [0], "probability": 0.3}],
        }

    def run():
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        result = json.loads(native_resource.task_result(task).value)
        if mpi:
            assert result["execution_metadata"]["backend"] == "distributed_mpi_gpu"
            assert result["mpi"]["ranks"] == 2
        diagnostics = json.loads(native_resource.task_logs(task))
        saved = {"result_path": [], "seed": result["seed"]}
        if noisy:
            saved["noise_seed"] = result["noise"]["seed"]
        assert diagnostics["seeds"] == [saved]
        assert diagnostics["seeds_truncated"] is False
        return result, saved

    (first, saved), (second, _) = run(), run()
    assert first["seed"] != second["seed"]
    assert first["counts"] != second["counts"]
    if noisy:
        assert first["noise"]["seed"] == first["seed"] & (2**32 - 1)
    document["execution"]["seed"] = saved["seed"]
    if noisy:
        document["noise"]["seed"] = saved["noise_seed"]
    assert run()[0]["counts"] == first["counts"]
    document["execution"]["seed"] = 0
    explicit_zero, _ = run()
    assert explicit_zero["seed"] == 0
    assert run()[0]["counts"] == explicit_zero["counts"]


def test_batch_seeds_survive_result_consumption(native_resource):
    """Keep seed paths for nested native batches after consuming the result."""
    first = native_request("execute", "statevector")
    first["execution"].pop("seed")
    second = native_request("execute", "statevector")
    second["execution"]["seed"] = 2**64 - 1
    second["noise"] = {
        "seed": 17,
        "channels": [{"kind": "readout", "targets": [0], "probability": 0.3}],
    }
    document = {
        "schema_version": 2,
        "operation": "batch",
        "requests": [
            first,
            {"schema_version": 2, "operation": "batch", "requests": [second]},
        ],
    }
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    diagnostics = json.loads(native_resource.task_logs(task))
    assert diagnostics["seeds"] == [
        {"result_path": [0], "seed": result["results"][0]["seed"]},
        {"result_path": [1, 0], "seed": 2**64 - 1, "noise_seed": 17},
    ]
    assert diagnostics["seeds_truncated"] is False


def test_exact_noise_capabilities_and_retained_logs(native_resource):
    """Check exact noise, capabilities, and logs after consuming the result."""
    resource = native_resource
    caps = json.loads(resource.target().value)
    assert caps["native"]["python_required"] is False
    assert "checkpoint_batch" in caps["native"]["operations"]
    document = native_request()
    document["noise"] = {
        "evaluation": "exact",
        "channels": [{"kind": "t1", "targets": [0], "gamma": 0.36}],
    }
    task = resource.task_start(request_payload(document))
    assert await_task(resource, task) == TaskStatus.Completed
    result = json.loads(resource.task_result(task).value)
    assert result["expectation_values"] == pytest.approx([0.8, 0.36])
    assert result["noise"]["evaluation"] == "exact_channels"
    diagnostics = json.loads(resource.task_logs(task))
    assert diagnostics["ok"] is True
    assert diagnostics["task_id"] == int(task)
    assert diagnostics["logs_truncated"] is False
    assert diagnostics["logs_omitted_bytes"] == 0
    assert [event["state"] for event in diagnostics["events"]] == [
        "queued",
        "started",
        "completed",
    ]
    assert all(event["timestamp_unix_ms"] > 0 for event in diagnostics["events"])
    assert diagnostics["context"]["simulator"] == {
        "backend": "qcsim",
        "method": "density_matrix",
    }
    document["simulator"]["options"] = {"unknown_option": True}
    with pytest.raises(InvalidInputError):
        resource.task_start(request_payload(document))


@pytest.mark.parametrize(
    "method",
    [
        "statevector",
        "matrix_product_state",
        "density_matrix",
        "matrix_product_operator",
    ],
)
def test_thermal_approximation_matches_python_policy(native_resource, method):
    """Accept sampled T2>T1 and retain its warning through the native worker."""
    document = native_request(method=method)
    document["noise"] = {
        "realizations": 256,
        "channels": [
            {
                "kind": "thermal_relaxation",
                "targets": [0],
                "duration": 0.3,
                "t1": 1.0,
                "t2": 1.5,
            }
        ],
    }
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    logs = json.loads(native_resource.task_logs(task))["logs"]
    marker = "thermal_T2_clamped_to_T1"
    if method in {"statevector", "matrix_product_state"}:
        assert marker in result["noise"]["approximations"]
        assert logs.count("effective T2 clamped to T1") == 1
        assert "circuit qubits [0]" in logs
        document["noise"]["channels"][0]["t2"] = 1.0
        clamped_task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, clamped_task) == TaskStatus.Completed
        clamped = json.loads(native_resource.task_result(clamped_task).value)
        assert result["expectation_values"] == pytest.approx(
            clamped["expectation_values"], abs=1e-12
        )
        assert marker not in clamped["noise"]["approximations"]
        assert (
            "effective T2 clamped to T1"
            not in json.loads(native_resource.task_logs(clamped_task))["logs"]
        )
    else:
        assert marker not in result["noise"]["approximations"]
        assert "effective T2 clamped to T1" not in logs
        assert result["expectation_values"][0] == pytest.approx(math.exp(-0.3 / 1.5))


@pytest.mark.parametrize(
    "method",
    [
        "statevector",
        "matrix_product_state",
        "density_matrix",
        "matrix_product_operator",
    ],
)
@pytest.mark.parametrize("operation", ["execute", "checkpoint_batch"])
def test_readout_controls_native_conditionals(native_resource, method, operation):
    """Readout follows qubits, reaches classical control, and respects skipped reads."""
    document = native_request(operation, method)
    header = "OPENQASM 2.0; qreg q[2]; creg c[4]; "
    circuit = {
        "num_qubits": 2,
        "num_clbits": 4,
        "source": header
        + "measure q[0]->c[2]; if(c==4) x q[1]; measure q[1]->c[0]; "
        + "if(c==5) measure q[0]->c[3];",
    }
    document["circuit"] = circuit
    if operation == "checkpoint_batch":
        document["circuit"] = {**circuit, "source": header}
        document["suffixes"] = [circuit]
    document["noise"] = {
        "realizations": 1,
        "channels": [{"kind": "readout", "targets": [0], "probability": 1.0}],
    }
    for source, expected in [
        (circuit["source"], "1011"),
        (header + "if(c==1) measure q[0]->c[0];", "0000"),
    ]:
        circuit["source"] = source
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        result = json.loads(native_resource.task_result(task).value)
        if operation == "checkpoint_batch":
            result = result["results"][0]
        assert result["counts"] == {expected: 16}


@pytest.mark.parametrize("method", ["statevector", "density_matrix"])
@pytest.mark.parametrize("realizations", [1, 128])
def test_native_readout_uses_execution_seed(native_resource, method, realizations):
    """Keep seeded readout reproducible and independent of the noise-injection RNG."""
    document = native_request("execute", method)
    document["circuit"][
        "source"
    ] = "OPENQASM 2.0; qreg q[1]; creg c[1]; measure q[0]->c[0];"
    document["execution"]["shots"] = 128
    document["noise"] = {
        "realizations": realizations,
        "channels": [{"kind": "readout", "targets": [0], "probability": 0.5}],
    }
    counts = []
    for injection_seed in [11, 11, 23]:
        document["noise"]["seed"] = injection_seed
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        counts.append(json.loads(native_resource.task_result(task).value)["counts"])
    assert counts[0] == counts[1] == counts[2]
    assert sum(counts[0].values()) == 128
    assert 24 < counts[0].get("0", 0) < 104


@pytest.mark.parametrize("method", [0, 1])
@pytest.mark.parametrize("seed", [0, 123, 2**64 - 1])
def test_legacy_execute_honors_seed(native_resource, method, seed):
    """Carry the full legacy integer seed through QRMI and fresh native workers."""
    capabilities = json.loads(native_resource.target().value)["native"]
    backend = next(
        item["legacy_id"]
        for item in capabilities["backends"]
        if item["name"] == "qcsim"
    )
    counts = []
    for value in [seed, seed, seed ^ 1]:
        payload = payload_from_input(
            {
                "job_type": "execute",
                "input": "OPENQASM 2.0; qreg q[2]; creg c[2]; "
                "h q[0]; h q[1]; measure q->c;",
                "qubits": 2,
                "simulator_type": backend,
                "simulation_method": method,
                "config": {"shots": 512, "seed": value},
            }
        )
        task = native_resource.task_start(payload)
        assert await_task(native_resource, task) == TaskStatus.Completed
        counts.append(json.loads(native_resource.task_result(task).value)["counts"])
    assert counts[0] == counts[1]
    assert counts[0] != counts[2]
    assert all(sum(result.values()) == 512 for result in counts)


def test_native_failure_is_retained_in_task_logs(native_resource):
    """Retain a structured error when the requested GPU ordinal is unavailable."""
    # Backend support for a query can be narrower than the schema. Use an
    # unavailable GPU ordinal for a deterministic worker-side failure instead.
    document = native_request("execute", "statevector")
    document["simulator"] = {"backend": "gpu", "options": {"gpu_device": 2147483647}}
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Failed
    diagnostics = json.loads(native_resource.task_logs(task))
    assert diagnostics["failure_message"]
    assert diagnostics["error"]["code"]


def test_mpi_launch_profile_returns_one_result(native_resource):
    """Return one numerical result and rank metadata from a two-rank launch."""
    profile = os.environ.get("QRMI_TEST_MPI_PROFILE")
    if not profile:
        pytest.skip(
            "set QRMI_TEST_MPI_PROFILE for a configured two-rank CPU smoke test"
        )
    document = native_request("estimate", "statevector")
    document["launch"] = {"profile": profile, "ranks": 2}
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    assert result["mpi"]["ranks"] == 2
    assert result["expectation_values"] == pytest.approx([1, 0], abs=1e-12)


def test_mpi_rank_failure_keeps_native_error(native_resource):
    """Propagate a native rank failure through the MPI supervisor."""
    profile = os.environ.get("QRMI_TEST_MPI_PROFILE")
    if not profile:
        pytest.skip("set QRMI_TEST_MPI_PROFILE to test native rank failures")
    document = native_request("execute", "statevector")
    document["simulator"] = {"backend": "gpu", "options": {"gpu_device": 2147483647}}
    document["launch"] = {"profile": profile, "ranks": 2}
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Failed
    diagnostics = json.loads(native_resource.task_logs(task))
    assert diagnostics["error"]["code"]


def test_local_gpu_distribution_through_qrmi(native_resource):
    """Execute ideal and noisy circuits using the local distributed GPU plugin."""
    if os.environ.get("QRMI_TEST_DISTRIBUTED_GPU") != "1":
        pytest.skip("set QRMI_TEST_DISTRIBUTED_GPU=1 with a licensed local GPU plugin")
    document = native_request("execute", "statevector")
    document["circuit"] = {
        "num_qubits": 2,
        "source": "OPENQASM 2.0; qreg q[2]; creg c[2]; x q[0]; measure q->c;",
    }
    document.pop("observables", None)
    document["simulator"] = {
        "backend": "distributed_gpu",
        "distribution": {
            "backend": "conventional",
            "devices": [0, 0],
            "global_qubits": [1],
            "flags": 17,
        },
    }
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    assert result["counts"] == {"10": 16}
    assert result["execution_metadata"]["backend"] == "distributed_gpu"

    document["noise"] = {
        "mode": "pauli",
        "evaluation": "trajectories",
        "realizations": 2,
        "channels": [{"kind": "bit_flip", "targets": [0], "probability": 1}],
    }
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    noisy_result = json.loads(native_resource.task_result(task).value)
    assert noisy_result["counts"] == {"00": 16}


def test_validation_errors_leave_daemon_usable(native_resource):
    """Classify invalid native requests and accept a valid request afterward."""
    document = native_request("execute", "statevector")
    document["circuit"] = {
        "format": "instructions",
        "num_qubits": 2,
        "source": [{"name": "cx", "qubits": [0]}],
    }
    with pytest.raises(InvalidInputError, match="number of qubits"):
        native_resource.task_start(request_payload(document))
    document = native_request("execute", "statevector")
    document["other_circuit"] = {"unknown_nested_field": True}
    with pytest.raises(InvalidInputError, match="other_circuit"):
        native_resource.task_start(request_payload(document))
    document = native_request("unknown_operation", "statevector")
    with pytest.raises(UnsupportedFunctionError, match="Unknown operation"):
        native_resource.task_start(request_payload(document))
    task = native_resource.task_start(request_payload(native_request()))
    assert await_task(native_resource, task) == TaskStatus.Completed
    assert json.loads(native_resource.task_result(task).value)[
        "expectation_values"
    ] == pytest.approx([1, 0], abs=1e-12)


@pytest.mark.parametrize(
    "operation", ["estimate", "statevector", "probabilities", "validate"]
)
def test_irrelevant_shots_do_not_allocate_tasks(native_resource, operation):
    """Reject shots on nonsampling operations without consuming a task ID."""
    document = native_request(operation, "statevector")
    document["execution"]["shots"] = 9
    with pytest.raises(
        InvalidInputError, match=f"execution.shots does not apply to {operation}"
    ):
        native_resource.task_start(request_payload(document))
    del document["execution"]["shots"]
    task = native_resource.task_start(request_payload(document))
    assert task == "0"
    assert await_task(native_resource, task) == TaskStatus.Completed


@pytest.mark.parametrize("launch", [None, {"profile": "absent", "ranks": 2}])
def test_checkpoint_suffix_launch_is_rejected(native_resource, launch):
    """Reject nested launch metadata before allocating a checkpoint task."""
    document = native_request("checkpoint_batch", "statevector")
    document["circuit"]["source"] = "OPENQASM 2.0; qreg q[1]; x q[0];"
    document["suffixes"] = [{**native_request("execute")["circuit"], "launch": launch}]
    with pytest.raises(InvalidInputError, match="Unknown field: launch"):
        native_resource.task_start(request_payload(document))
    del document["suffixes"][0]["launch"]
    task = native_resource.task_start(request_payload(document))
    assert task == "0"
    assert await_task(native_resource, task) == TaskStatus.Completed


@pytest.mark.parametrize("native_resource", ["double_failure"], indirect=True)
def test_native_and_cleanup_failures_reach_logs(native_resource):
    """Preserve the original native error when Slurm cleanup also fails."""
    document = native_request("execute", "statevector")
    document["launch"] = {"profile": "broken", "ranks": 2}
    # Valid input larger than a pipe forces the fake launcher to close mid-upload.
    document["circuit"]["source"] = (
        "//" + "x" * (1024 * 1024) + "\n" + document["circuit"]["source"]
    )
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Failed
    error = json.loads(native_resource.task_logs(task))["error"]
    assert error["code"] == "cleanup_unconfirmed"
    assert error["cleanup_confirmed"] is False
    supervisor = error["causes"][0]
    assert "scancel rejected" in supervisor["message"]
    assert supervisor["causes"][0] == {
        "code": "native_test_failure",
        "message": "original native cause",
    }


@pytest.mark.parametrize("runner", ["python", "rust"])
def test_cli_failure_exits_nonzero_and_logs_to_stderr(
    native_resource, tmp_path, runner
):
    """Make both CLIs exit unsuccessfully and write diagnostics to stderr."""
    if runner == "rust":
        binary = os.environ.get("QRMI_TEST_RUST_RUNNER")
        if not binary:
            pytest.skip("set QRMI_TEST_RUST_RUNNER to test the Rust CLI")
        command = [binary]
    else:
        command = [sys.executable, "-m", "qrmi.tools.task_runner.main"]
    document = native_request("execute", "statevector")
    document["simulator"] = {"backend": "gpu", "options": {"gpu_device": 2147483647}}
    source = tmp_path / "failure.json"
    source.write_text(json.dumps(document))
    env = {
        **os.environ,
        "QRMI_JOB_QPU_RESOURCES": "native_test",
        "QRMI_JOB_QPU_TYPES": "maestro-local",
        "native_test_QRMI_JOB_ACQUISITION_TOKEN": native_resource.acquire(),
    }
    result = subprocess.run(
        [*command, "native_test", str(source)],
        env=env,
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    assert result.returncode != 0, result.stdout
    assert "Failed" in result.stderr
    assert "failure_message" in result.stderr
    assert "failure_message" not in result.stdout


@pytest.mark.parametrize(
    "native_resource", ["invalid_profiles", "missing_scancel"], indirect=True
)
def test_mpi_configuration_errors_are_typed(native_resource):
    """Classify malformed profiles and missing cancellation tools as config errors."""
    document = native_request("execute", "statevector")
    document["launch"] = {"profile": "broken", "ranks": 2}
    with pytest.raises(ConfigError):
        native_resource.task_start(request_payload(document))


def test_launch_validation_and_batch_metadata_contract(native_resource):
    """Validate launch routing and retain MPI metadata on a batch result."""
    profile = os.environ.get("QRMI_TEST_MPI_PROFILE")
    if not profile:
        pytest.skip("set QRMI_TEST_MPI_PROFILE for the launch contract test")
    document = native_request("execute", "statevector")
    for launch, error in [
        ({"profile": profile, "ranks": 3}, InvalidInputError),
        ({"profile": "not-a-configured-profile", "ranks": 2}, UnsupportedFunctionError),
        (None, InvalidInputError),
    ]:
        document["launch"] = launch
        with pytest.raises(error):
            native_resource.task_start(request_payload(document))
    document["launch"] = {"profile": profile, "ranks": 2}
    nested = {"schema_version": 2, "operation": "batch", "requests": [document]}
    with pytest.raises(InvalidInputError):
        native_resource.task_start(request_payload(nested))
    document.pop("launch")
    nested["launch"] = {"profile": profile, "ranks": 2}
    task = native_resource.task_start(request_payload(nested))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    assert result["mpi"]["ranks"] == 2
    assert result["results"][0]["shots"] == 16


@pytest.mark.parametrize("native_resource", ["short_lease"], indirect=True)
def test_task_logs_renew_the_documented_session_lease(native_resource):
    """Keep a short-lived session alive by retrieving retained task logs."""
    assert (
        json.loads(native_resource.target().value)["server"]["session_lease_seconds"]
        == 1
    )
    task = native_resource.task_start(
        request_payload(native_request("execute", "statevector"))
    )
    assert await_task(native_resource, task) == TaskStatus.Completed
    native_resource.task_result(task)
    deadline = time.monotonic() + 2.4
    while time.monotonic() < deadline:
        assert json.loads(native_resource.task_logs(task))["ok"] is True
        time.sleep(0.2)


@pytest.mark.parametrize("noisy", [False, True])
def test_real_mpi_gpu_execution(native_resource, noisy):
    """Check ideal and noisy execution on actual MPI GPU ranks when configured."""
    profile = os.environ.get("QRMI_TEST_MPI_GPU_PROFILE")
    if not profile:
        pytest.skip(
            "set QRMI_TEST_MPI_GPU_PROFILE with rank-local GPUs and CUDA-aware MPI"
        )
    document = native_request("execute", "statevector")
    document["circuit"] = {
        "num_qubits": 2,
        "source": "OPENQASM 2.0; qreg q[2]; creg c[2]; x q[0]; measure q->c;",
    }
    document["simulator"] = {
        "backend": "distributed_mpi_gpu",
        "options": {"gpu_device": 0},
        "distribution": {
            "backend": "ex",
            "global_qubits": [1],
            "mpi_p2p_bits": 0,
        },
    }
    document["launch"] = {"profile": profile, "ranks": 2}
    if noisy:
        document["noise"] = {
            "mode": "pauli",
            "evaluation": "trajectories",
            "realizations": 2,
            "channels": [{"kind": "bit_flip", "targets": [0], "probability": 1}],
        }
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    assert result["execution_metadata"]["backend"] == "distributed_mpi_gpu"
    assert result["mpi"]["ranks"] == 2
    assert result["counts"] == {("00" if noisy else "10"): 16}


@pytest.mark.parametrize("backend", ["qcsim", "gpu"])
@pytest.mark.parametrize(
    "name",
    [
        "mps-operators",
        "mpo-operators",
        "tensor-batch",
        "tensor-expectations",
        "tensor-bulk",
        "mpo-wide",
    ],
)
def test_current_tensor_examples_through_qrmi(native_resource, name, backend):
    """Preserve ordered operators, complex scale, bit order and fusion metadata."""
    if backend == "gpu" and os.environ.get("QRMI_TEST_GPU") != "1":
        pytest.skip("set QRMI_TEST_GPU=1 with an available GPU plugin")
    document = example_doc(name)
    document["simulator"]["backend"] = backend
    task = native_resource.task_start(request_payload(document))
    assert await_task(native_resource, task) == TaskStatus.Completed
    result = json.loads(native_resource.task_result(task).value)
    if name == "mps-operators":
        assert result["operator_expectation"] == pytest.approx([0, -1])
    elif name == "mpo-operators":
        assert result["trace"] == pytest.approx([4, 0])
        assert result["element"] == pytest.approx([4, 0])
        for field, scale in [
            ("expectations_complex", 1),
            ("unnormalized_expectations", 4),
        ]:
            assert len(result[field]) == 3
            for actual, expected in zip(result[field], [-scale, scale, -scale]):
                assert actual == pytest.approx([expected, 0])
        for field, scale in [("density_matrix", 1), ("unnormalized_density_matrix", 4)]:
            assert result[field]["dimension"] == 4
            assert len(result[field]["row_major"]) == 16
            for index, value in enumerate(result[field]["row_major"]):
                assert value == pytest.approx([scale if index == 10 else 0, 0])
    elif name == "tensor-batch":
        assert result["steps"] == [0, 1, 2]
        assert len(result["expectation_values"]) == 3
        for values, expected in zip(result["expectation_values"], [1, -1, 1]):
            assert values == pytest.approx([expected, 1, expected])
    elif name == "tensor-expectations":
        assert result["expectation_values"] == pytest.approx([-1, 1, -1])
    elif name == "tensor-bulk":
        assert result["basis_states"] == [0, 1, 2, 3]
        for index, amplitude in enumerate(result["amplitudes"]):
            assert amplitude == pytest.approx([1 if index == 1 else 0, 0])
        assert result["execution_metadata"]["gate_fusion"]["requested"] is True
        document["operation"] = "probabilities"
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        assert json.loads(native_resource.task_result(task).value)[
            "probabilities"
        ] == pytest.approx([0, 1, 0, 0])
    elif name == "mpo-wide":
        assert result["probability"] == pytest.approx(1)
        target = document.pop("target_state")
        document["operation"] = "execute"
        document["execution"]["shots"] = 3
        document["circuit"][
            "source"
        ] = "OPENQASM 2.0; qreg q[65]; creg c[65]; x q[0]; x q[64]; measure q->c;"
        task = native_resource.task_start(request_payload(document))
        assert await_task(native_resource, task) == TaskStatus.Completed
        assert json.loads(native_resource.task_result(task).value)["counts"] == {
            target: 3
        }


def test_status_and_current_capabilities_through_qrmi(native_resource):
    """Expose server health and native discovery without dropping additive fields."""
    status = native_resource.status()
    assert status.healthy is True
    assert status.busy is False
    assert status.pending_job_count == 0
    assert status.capacity is None
    caps = json.loads(native_resource.target().value)
    assert "status" in caps["server"]["commands"]
    assert caps["server"]["build"]["source_revision"]
    assert caps["native"]["build"]["source_revision"]
    assert caps["native"]["capability_scope"] == "validation"
    diagnostics = {entry["name"]: entry for entry in caps["native"]["diagnostics"]}
    assert {"backend": "qcsim", "method": "matrix_product_state"} in diagnostics[
        "operator_expectation"
    ]["supported_configurations"]
    options = {entry["name"]: entry for entry in caps["native"]["options"]}
    assert options["precision"]["enum"] == ["single", "double"]
    assert options["pp_workers"]["maximum"] == 1024
    assert "use_double_precision" not in options
    document = example_doc("tensor-bulk")
    document["simulator"]["options"] = {"mps_measure_no_collapse": True}
    with pytest.raises(InvalidInputError, match="Unknown simulator option"):
        native_resource.task_start(request_payload(document))


def run_native(resource, document):
    """Submit through QRMI and return the decoded native result."""
    task = resource.task_start(request_payload(document))
    assert await_task(resource, task) == TaskStatus.Completed
    return task, json.loads(resource.task_result(task).value)


def option_entry(caps, name):
    """One native option's discovery metadata."""
    return next(entry for entry in caps["native"]["options"] if entry["name"] == name)


def test_current_option_contract_through_qrmi(native_resource):
    """Discovery metadata and backend-specific bounds reach QRMI unchanged."""
    caps = json.loads(native_resource.target().value)
    workers = option_entry(caps, "pp_workers")
    assert workers["maximum"] == 1024
    assert workers["constraints"] == [{"backend": "gpu", "minimum": 1, "maximum": 64}]
    assert option_entry(caps, "pp_max_fused_gates")["maximum"] == 256
    strategy = option_entry(caps, "mps_routing_strategy")
    assert strategy["enum"] == ["combined", "legacy"]
    assert strategy["default"] == "legacy"
    assert {"backend": "gpu", "method": "matrix_product_state"} not in strategy[
        "supported_configurations"
    ]
    assert option_entry(caps, "trajectory_policy")["default"] == "shot_v1"

    # Accepted combinations validate without a GPU, plugin or licence.
    for backend, method, options in [
        ("gpu", "pauli_propagator", {"pp_workers": 64, "pp_max_fused_gates": 1}),
        ("gpu", "pauli_propagator", {"pp_memory_limit_bytes": 1 << 30}),
        ("qcsim", "pauli_propagator", {"pp_workers": 0}),
        ("qcsim", "matrix_product_state", {"mps_routing_strategy": "combined"}),
        ("gpu", "matrix_product_operator", {"mps_routing_virtual_bonds": True}),
        ("qcsim", "density_matrix", {"trajectory_policy": "block_v1"}),
        (
            "qcsim",
            "statevector",
            {"min_cluster_qubits": 2, "max_cluster_qubits": 4, "auto_reduce": False},
        ),
    ]:
        document = native_request("validate", method)
        document["simulator"]["backend"] = backend
        document["simulator"]["options"] = options
        _, result = run_native(native_resource, document)
        assert result["valid"] is True, options

    # Rejections keep their QRMI categories and never create a task.
    for backend, method, options, error in [
        ("gpu", "pauli_propagator", {"pp_workers": 0}, InvalidInputError),
        ("gpu", "pauli_propagator", {"pp_workers": 65}, InvalidInputError),
        ("qcsim", "pauli_propagator", {"pp_workers": 1025}, InvalidInputError),
        ("gpu", "pauli_propagator", {"pp_max_fused_gates": 257}, InvalidInputError),
        (
            "qcsim",
            "pauli_propagator",
            {"pp_memory_limit_bytes": 1024},
            UnsupportedFunctionError,
        ),
        (
            "gpu",
            "matrix_product_state",
            {"mps_routing_strategy": "combined"},
            UnsupportedFunctionError,
        ),
        (
            "qcsim",
            "statevector",
            {"min_cluster_qubits": 5, "max_cluster_qubits": 4},
            InvalidInputError,
        ),
        (
            "qcsim",
            "matrix_product_state",
            {"trajectory_policy": "shot_v1"},
            UnsupportedFunctionError,
        ),
    ]:
        document = native_request("execute", method)
        document["simulator"]["backend"] = backend
        document["simulator"]["options"] = options
        with pytest.raises(error):
            native_resource.task_start(request_payload(document))


def test_new_native_examples_through_qrmi(native_resource):
    """Newer native operations and instruction fields pass through unchanged."""
    caps = json.loads(native_resource.target().value)
    assert {"adjoint_jacobian", "adjoint_vjp"} <= set(caps["native"]["operations"])
    assert caps["native"]["instructions"]["fields"]["noiseless"] == ["gates"]

    document = example_doc("adjoint")
    task, result = run_native(native_resource, document)
    assert result["num_parameters"] == 2
    assert result["num_values"] == 4
    # The second row has every angle zero: |00>, P(q1=1) = 0.
    assert result["expectation_values"][1] == pytest.approx([1, 0.5, 1, 0])
    # d<ZI>/d(ry angle) = -sin(0.4); the rx angle cannot change <ZI>.
    assert result["jacobian"][0][0] == pytest.approx([-math.sin(0.4), 0], abs=1e-12)
    logs = json.loads(native_resource.task_logs(task))
    assert logs["seeds"] == []
    assert logs["native_build"] == caps["native"]["build"]

    document["operation"] = "adjoint_vjp"
    document["cotangents"] = [[1, 0, 0, 0], [0, 0, 0, 0]]
    _, vjp = run_native(native_resource, document)
    assert vjp["vjp"][0] == pytest.approx(result["jacobian"][0][0])
    document["cotangents"] = [[1, 0, 0, 0]]
    with pytest.raises(InvalidInputError, match="cotangents"):
        native_resource.task_start(request_payload(document))

    document = example_doc("mirror-observables")
    _, result = run_native(native_resource, document)
    assert result["expectation_values"] == pytest.approx([1, 1, 1, 1])
    # A bond cap of one truncates the GHZ entanglement the mirror must undo.
    document["simulator"]["options"]["max_bond_dimension"] = 1
    _, result = run_native(native_resource, document)
    assert min(result["expectation_values"]) < 0.5

    document = example_doc("instructions")
    _, result = run_native(native_resource, document)
    assert result["counts"] == {"01": 32}
    # Pure states sample the channel and need a trajectory noise block.
    document["simulator"]["method"] = "statevector"
    with pytest.raises(UnsupportedFunctionError, match="trajectory"):
        native_resource.task_start(request_payload(document))
    document["noise"] = {"realizations": 32}
    _, result = run_native(native_resource, document)
    assert result["counts"] == {"01": 32}


def test_readout_refusal_and_mixed_state_mirror_through_qrmi(native_resource):
    """Removed measurements refuse readout; mixed-state mirror observables run."""
    annotated = {
        "format": "instructions",
        "num_qubits": 1,
        "source": [
            {"name": "ry", "qubits": [0], "params": [0.4]},
            {"name": "measure", "qubits": [0], "readout_error": 1},
        ],
    }
    plain = {"num_qubits": 1, "source": "OPENQASM 2.0; qreg q[1]; ry(0.4) q[0];"}
    base = {
        "schema_version": 2,
        "simulator": {"backend": "qcsim", "method": "statevector"},
        "execution": {"seed": 123},
    }
    refused = [
        {
            **base,
            "operation": operation,
            "circuit": annotated,
            "observables": ["Z"],
            **({"cotangents": [[1]]} if operation == "adjoint_vjp" else {}),
        }
        for operation in (
            "adjoint_expectation_values",
            "adjoint_jacobian",
            "adjoint_vjp",
        )
    ]
    refused.append(
        {
            **base,
            "operation": "inner_product",
            "circuit": plain,
            "other_circuit": annotated,
        }
    )
    refused.append(
        {
            **base,
            "operation": "incremental_evolve",
            "circuit": plain,
            "step_circuit": annotated,
            "steps": [1],
            "observables": ["Z"],
        }
    )
    # A checkpoint prefix's terminal measurements are removed as well.
    refused.append(
        {
            **base,
            "operation": "checkpoint_batch",
            "circuit": annotated,
            "suffixes": [
                {
                    "num_qubits": 1,
                    "source": "OPENQASM 2.0; qreg q[1]; creg c[1]; measure q[0] -> c[0];",
                }
            ],
            "execution": {"seed": 123, "shots": 8},
        }
    )
    for document in refused:
        with pytest.raises(UnsupportedFunctionError, match="readout_error"):
            native_resource.task_start(request_payload(document))

    # Mirror expectations need no pure-state overlap; plain mirror fidelity does.
    document = example_doc("mirror-observables")
    for method in ("density_matrix", "matrix_product_operator"):
        document["simulator"] = {"backend": "qcsim", "method": method}
        _, result = run_native(native_resource, document)
        assert result["expectation_values"] == pytest.approx([1, 1, 1, 1])
        fidelity = {
            key: value for key, value in document.items() if key != "observables"
        }
        with pytest.raises(UnsupportedFunctionError, match="mixed state"):
            native_resource.task_start(request_payload(fidelity))


def test_mixed_state_queries_through_qrmi(native_resource):
    """Mixed-state overlap and statevector fidelity use the request payload."""
    caps = json.loads(native_resource.target().value)
    operations = {"density_matrix_overlap", "fidelity_with_statevector"}
    assert operations <= set(caps["native"]["operations"])
    queries = caps["native"]["mixed_state_queries"]
    assert set(queries["operations"]) == operations
    assert queries["max_target_statevector_qubits"] == 20

    document = example_doc("mixed-state")
    _, result = run_native(native_resource, document)
    fidelity, overlap = result["results"]
    assert fidelity["fidelity"] == pytest.approx(0.7624)
    assert overlap["overlap"] == pytest.approx([0.5, 0])

    # Without noise both mixed-state methods reproduce the Bell state exactly.
    query = document["requests"][0]
    del query["noise"]
    for method in ("density_matrix", "matrix_product_operator"):
        query["simulator"]["method"] = method
        _, result = run_native(native_resource, query)
        assert result["fidelity"] == pytest.approx(1)
    query["target_statevector"] = [[1, 0], [1, 0], [0, 0], [0, 0]]
    with pytest.raises(InvalidInputError, match="normalized"):
        native_resource.task_start(request_payload(query))
    # Pure states use inner_product and the statevector queries instead.
    query["target_statevector"] = [[1, 0], [0, 0], [0, 0], [0, 0]]
    query["simulator"]["method"] = "statevector"
    with pytest.raises(UnsupportedFunctionError, match="density_matrix"):
        native_resource.task_start(request_payload(query))
