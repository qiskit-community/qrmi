"""Opt-in tests against a real Maestro server, using only fresh test sessions."""

import json
import os
import socket
import time
import uuid

import pytest

from qrmi import (
    ConfigError,
    Payload,
    QRMIService,
    QuantumResource,
    ResourceType,
    TaskNotFoundError,
    TaskNotReadyError,
    TaskStatus,
)

pytestmark = pytest.mark.skipif(
    os.environ.get("QRMI_TEST_MAESTRO_LIVE") != "1",
    reason="set QRMI_TEST_MAESTRO_LIVE=1 to use a real Maestro server",
)

PREPARE = """OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];
x q[0];
cx q[0],q[1];
"""
MEASURE = "measure q -> c;"
BELL = PREPARE.replace("x q[0];", "h q[0];")


@pytest.fixture(name="command")
def fixture_command():
    """Inspect and clean up our sessions independently of the QRMI adapter."""
    path = os.environ.get("QRMI_MAESTRO_SOCKET", "/run/maestro.sock")

    def send(request):
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(15)
            connection.connect(path)
            connection.sendall((request + "\n").encode())
            connection.shutdown(socket.SHUT_WR)
            with connection.makefile("rb") as stream:
                reply = stream.readline().decode().strip()
        assert reply, f"No Maestro response to {request!r}"
        return reply

    return send


@pytest.fixture(name="live_resource")
def fixture_live_resource(monkeypatch, command):
    """Discover Maestro and allocate a session with verified cleanup."""
    name = f"qrmi_live_{uuid.uuid4().hex}"
    monkeypatch.delenv("QRMI_PLUGIN_ERROR", raising=False)
    monkeypatch.setenv("QRMI_LIST_DELIMITER", ",")
    monkeypatch.setenv("QRMI_JOB_QPU_RESOURCES", name)
    monkeypatch.setenv("QRMI_JOB_QPU_TYPES", "maestro-local")
    service = QRMIService()
    resource = service.resource(name)
    assert resource is not None, "Maestro is not accessible through QRMIService"
    assert service.resources() == [resource]
    assert resource.resource_type() == ResourceType.MaestroLocal
    token = resource.acquire()
    try:
        assert resource.acquire() == token
        assert command(f"SESSION {token} EXISTS") == "OK YES"
        yield resource, token
    finally:
        # Delete only the session allocated above, including after test failures.
        if command(f"SESSION {token} EXISTS") == "OK YES":
            assert command(f"SESSION {token} DELETE") in ("OK", "OK YES")
        assert command(f"SESSION {token} EXISTS") == "OK NO"


def payload(circuit, simulator, method, job_type="execute", observables=""):
    """Build a small Maestro task for either supported simulator."""
    return Payload.MaestroLocal(
        input=circuit,
        job_type=job_type,
        qubits=2,
        simulator_type=simulator,
        simulation_method=method,
        observables=observables,
        config=json.dumps({"shots": 128}),
    )


def wait_for_completion(resource, task_id):
    """Wait up to a minute, reporting failure or cancellation immediately."""
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline:
        status = resource.task_status(task_id)
        if status == TaskStatus.Completed:
            return
        assert status in (
            TaskStatus.Queued,
            TaskStatus.Running,
        ), f"Task {task_id} ended with {status}"
        time.sleep(0.05)
    pytest.fail(f"Task {task_id} did not complete within 60 seconds")


def backend_id(resource, name):
    """Legacy IDs depend on the server build; discover them by backend name."""
    capabilities = json.loads(resource.target().value)
    for backend in capabilities["native"]["backends"]:
        if backend["name"] == name:
            return backend["legacy_id"]
    return pytest.skip(f"The current Maestro build does not include {name}")


@pytest.mark.parametrize(
    "expected_simulator,method,expected_method",
    [("aer", 0, "statevector"), ("qcsim", 1, "matrix_product_state")],
)
def test_execute_and_consumed_results(
    live_resource, command, method, expected_simulator, expected_method
):
    """Validate counts and retain completion after consuming real results."""
    resource, token = live_resource
    simulator = backend_id(resource, expected_simulator)
    task_id = resource.task_start(payload(PREPARE + MEASURE, simulator, method))
    wait_for_completion(resource, task_id)
    result = json.loads(resource.task_result(task_id).value)
    assert result["counts"] == {"11": 128}
    assert result["simulator"] == expected_simulator
    assert result["method"] == expected_method
    assert command(f"SESSION {token} TASK {task_id} EXISTS") == "OK NO"
    assert resource.task_status(task_id) == TaskStatus.Completed
    diagnostics = json.loads(resource.task_logs(task_id))
    assert diagnostics["ok"] is True
    assert diagnostics["failure_message"] is None
    resource.task_stop(task_id)
    with pytest.raises(TaskNotFoundError):
        resource.task_result(task_id)


@pytest.mark.parametrize("backend,method", [("aer", 0), ("qcsim", 1)])
def test_estimate_bell_state(live_resource, backend, method):
    """Check three Bell-state expectation values against their exact values."""
    resource, _ = live_resource
    simulator = backend_id(resource, backend)
    task_id = resource.task_start(
        payload(BELL, simulator, method, job_type="estimate", observables="ZZ;XX;YY")
    )
    wait_for_completion(resource, task_id)
    result = json.loads(resource.task_result(task_id).value)
    assert result["expectation_values"] == pytest.approx([1.0, 1.0, -1.0], abs=1e-5)
    resource.task_stop(task_id)


def test_cancel_unsubmitted_task(live_resource, command):
    """Require acknowledged cancellation and retain the cancelled status."""
    resource, token = live_resource
    # An unsubmitted task cannot finish before the cancellation request.
    reply = command(f"SESSION {token} TASK CREATE")
    assert reply.startswith("OK "), reply
    task_id = str(int(reply.removeprefix("OK ")))
    assert resource.task_status(task_id) == TaskStatus.Queued
    with pytest.raises(TaskNotReadyError):
        resource.task_result(task_id)
    resource.task_stop(task_id)
    assert resource.task_status(task_id) == TaskStatus.Cancelled
    assert command(f"SESSION {token} TASK {task_id} EXISTS") == "OK NO"
    resource.task_stop(task_id)
    with pytest.raises(TaskNotReadyError):
        resource.task_result(task_id)


def test_release_and_stale_token_replacement(live_resource, command, monkeypatch):
    """Release sessions explicitly and replace a confirmed stale token."""
    resource, token = live_resource
    with pytest.raises(TaskNotFoundError):
        resource.task_status("4294967295")
    capabilities = json.loads(resource.target().value)
    assert capabilities["version"] == 2
    assert capabilities["native"]["python_required"] is False
    with pytest.raises(TaskNotFoundError):
        resource.task_logs("4294967295")
    resource.release(token)
    assert command(f"SESSION {token} EXISTS") == "OK NO"
    with pytest.raises(ConfigError):
        resource.task_status("1")

    name = resource.resource_id()
    monkeypatch.setenv(f"{name}_QRMI_JOB_ACQUISITION_TOKEN", token)
    replacement = QuantumResource(name, ResourceType.MaestroLocal)
    new_token = replacement.acquire()
    try:
        assert command(f"SESSION {new_token} EXISTS") == "OK YES"
        assert replacement.acquire() == new_token
        replacement.release(new_token)
        assert command(f"SESSION {new_token} EXISTS") == "OK NO"
        with pytest.raises(ConfigError):
            replacement.task_status("1")
    finally:
        if command(f"SESSION {new_token} EXISTS") == "OK YES":
            assert command(f"SESSION {new_token} DELETE") in ("OK", "OK YES")
        assert command(f"SESSION {new_token} EXISTS") == "OK NO"
