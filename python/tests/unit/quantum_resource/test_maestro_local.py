"""Maestro binding regressions using a scripted Unix socket, without a daemon."""

import ctypes
import json
import os
import socket
import subprocess
import tempfile
import threading
from collections import deque
from pathlib import Path

import pytest

from qrmi import (
    ConfigError,
    InvalidInputError,
    Payload,
    QrmiError,
    QRMIService,
    QuantumResource,
    ResourceType,
    ResourceStatusCode,
    TaskNotFoundError,
    TaskNotReadyError,
    UnsupportedFunctionError,
    UnsupportedPayloadError,
)


@pytest.fixture(name="server")
def fixture_server(monkeypatch):
    """Serve exact command/reply pairs; fail on missing or unexpected traffic."""
    pending = deque()
    failures = []
    stopping = threading.Event()
    with tempfile.TemporaryDirectory(prefix="qrmi-") as directory:
        path = str(Path(directory) / "maestro.sock")
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as listener:
            listener.bind(path)
            listener.listen()
            listener.settimeout(0.05)
            monkeypatch.setenv("QRMI_MAESTRO_SOCKET", path)

            def serve():
                while not stopping.is_set():
                    try:
                        connection, _ = listener.accept()
                    except socket.timeout:
                        continue
                    with connection:
                        connection.settimeout(2)
                        try:
                            with connection.makefile("rb") as stream:
                                command = stream.readline().decode().rstrip("\r\n")
                            expected, reply = pending.popleft()
                            if command != expected:
                                failures.append(
                                    f"Expected {expected!r}, received {command!r}"
                                )
                        except (OSError, IndexError) as error:
                            failures.append(str(error))
                            reply = "ERROR unexpected request"
                        connection.sendall((reply + "\n").encode())

            worker = threading.Thread(target=serve)
            worker.start()
            try:
                yield pending
            finally:
                stopping.set()
                worker.join(timeout=3)
                assert not worker.is_alive(), "mock server did not stop"
                assert not failures, failures
                assert not pending, f"Unconsumed requests: {list(pending)}"


@pytest.fixture(autouse=True)
def fixture_clean_job_environment(monkeypatch):
    """Do not inherit scheduler state from the machine running the tests."""
    for name in (
        "QRMI_PLUGIN_ERROR",
        "py_maestro_QRMI_JOB_ACQUISITION_TOKEN",
        "offline_maestro_QRMI_JOB_ACQUISITION_TOKEN",  # pragma: allowlist secret
    ):
        monkeypatch.delenv(name, raising=False)
    monkeypatch.setenv("QRMI_LIST_DELIMITER", ",")


def test_python_errors_are_classified_without_a_session(monkeypatch):
    """Expose QRMI error categories before contacting a Maestro session."""
    resource = QuantumResource("py_maestro", ResourceType.MaestroLocal)
    with pytest.raises(InvalidInputError):
        resource.task_status("invalid")
    with pytest.raises(ConfigError):
        resource.task_status("1")
    with pytest.raises(InvalidInputError):
        resource.release("invalid")
    with pytest.raises(ConfigError):
        resource.task_logs("1")
    with pytest.raises(UnsupportedPayloadError):
        resource.task_start(Payload.QiskitPrimitive(input="{}", program_id="sampler"))
    with pytest.raises(InvalidInputError):
        resource.task_start(
            Payload.MaestroLocal(
                input="OPENQASM 2.0; qreg q[1];",
                job_type="execute",
                qubits=1,
                simulator_type=0,
                simulation_method=0,
                observables="",
                config="{",
            )
        )
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "invalid")
    with pytest.raises(ConfigError):
        QuantumResource("py_maestro", ResourceType.MaestroLocal)


def test_config_preserves_socket_session_and_status(server, monkeypatch):
    """The config factory uses explicit settings despite conflicting environment."""
    socket_path = os.environ["QRMI_MAESTRO_SOCKET"]
    monkeypatch.setenv("QRMI_MAESTRO_SOCKET", "/wrong.sock")
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "invalid")
    resource = QuantumResource.from_config(
        "py_maestro",
        ResourceType.MaestroLocal,
        {"qrmi_maestro_socket": socket_path, "qrmi_job_acquisition_token": "42"},
    )
    server.extend(
        [
            ('API {"version":2,"command":"status"}', "ERROR Unknown command"),
            ("PING", "OK YES"),
            ('API {"version":2,"command":"status"}', "ERROR Unknown command"),
            ("PING", "OK NO"),
            ("SESSION 42 EXISTS", "OK YES"),
            ("SESSION 42 DELETE", "OK YES"),
        ]
    )
    assert resource.status().to_dict() == {
        "status": "online",
        "status_reason": None,
        "healthy": None,
        "busy": None,
        "capacity": None,
        "pending_job_count": None,
    }
    assert resource.status().status == ResourceStatusCode.Offline
    assert resource.acquire() == "42"
    resource.release("42")
    assert os.environ["QRMI_MAESTRO_SOCKET"] == "/wrong.sock"
    with pytest.raises(ConfigError):
        QuantumResource.from_config(
            "py_maestro",
            ResourceType.MaestroLocal,
            {"QRMI_JOB_ACQUISITION_TOKEN": "invalid"},
        )


def test_provider_values_preserve_upstream_abi():
    """OQTOPUS keeps the upstream value; Maestro moves after it."""
    assert int(ResourceType.OQTOPUS) == 7
    assert int(ResourceType.MaestroLocal) == 8


@pytest.mark.parametrize("mode", ["execute", "estimate"])
def test_lua_maestro_payload_dispatch(server, monkeypatch, mode):
    """Exercise the Lua/C adapters, defaults, errors and detached sessions."""
    module = os.environ.get("QRMI_TEST_LUA_MODULE")
    executable = os.environ.get("QRMI_TEST_LUA")
    if not module or not executable:
        pytest.skip("set QRMI_TEST_LUA_MODULE and QRMI_TEST_LUA for Lua checks")
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "8")
    server.extend(
        [
            ("SESSION 8 EXISTS", "OK YES"),
            ("SESSION 8 TASK CREATE", "OK 1"),
            (f"SESSION 8 TASK 1 SET_TYPE {mode.upper()}", "OK YES"),
        ]
    )
    if mode == "estimate":
        server.append(("SESSION 8 TASK 1 SET_OBSERVABLES Z", "OK YES"))
    simulator, method, config = (
        (2, 3, '{"shots":8}') if mode == "estimate" else (0, 0, "{}")
    )
    server.extend(
        [
            ("SESSION 8 TASK 1 SET_QUBITS 1", "OK YES"),
            (f"SESSION 8 TASK 1 SET_SIM_TYPE {simulator}", "OK YES"),
            (f"SESSION 8 TASK 1 SET_METHOD {method}", "OK YES"),
            ("SESSION 8 TASK 1 SET_QASM OPENQASM 2.0; qreg q[1];", "OK YES"),
            (f"SESSION 8 TASK 1 SET_OPTIONS {config}", "OK YES"),
            ("SESSION 8 TASK 1 EXECUTE", "OK YES"),
            ("SESSION 8 DELETE", "OK YES"),
        ]
    )
    script = (
        Path(__file__).resolve().parents[4] / "lua/tests/maestro_local.lua"
    ).read_text(encoding="utf-8")
    result = subprocess.run(
        [executable, "-", mode],
        input=script,
        text=True,
        capture_output=True,
        check=False,
        timeout=15,
    )
    assert result.returncode == 0, result.stdout + result.stderr


def test_service_preserves_maestro_session(server, monkeypatch):
    """Discover accessible resources and retain their acquired sessions."""
    monkeypatch.setenv("QRMI_JOB_QPU_RESOURCES", "py_maestro,offline_maestro")
    monkeypatch.setenv("QRMI_JOB_QPU_TYPES", "maestro-local,maestro-local")
    server.extend(
        [
            ('API {"version":2,"command":"status"}', "ERROR Unknown command"),
            ("PING", "OK YES"),
            ('API {"version":2,"command":"status"}', "ERROR Unknown command"),
            ("PING", "OK NO"),
            ("SESSION CREATE", "OK 42"),
            ("SESSION 42 EXISTS", "OK YES"),
            ("SESSION 42 DELETE", "OK YES"),
        ]
    )
    service = QRMIService()
    resources = service.resources()
    assert len(resources) == 1
    resource = service.resource("py_maestro")
    assert resource is resources[0]
    assert service.resource("offline_maestro") is None
    assert resource.resource_id() == "py_maestro"
    assert resource.resource_type() == ResourceType.MaestroLocal
    assert resource.acquire() == "42"
    assert service.resource("py_maestro").acquire() == "42"
    resource.release("42")
    with pytest.raises(ConfigError):
        resource.task_status("1")


def test_missing_and_failed_tasks_raise_distinct_errors(server, monkeypatch):
    """Keep missing tasks distinct from known failures in Python."""
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "8")
    server.extend(
        [
            ("SESSION 8 TASK 1 EXISTS", "OK NO"),
            ("SESSION 8 TASK 2 EXISTS", "OK YES"),
            ("SESSION 8 TASK 2 FAILED", "OK YES"),
        ]
    )
    resource = QuantumResource("py_maestro", ResourceType.MaestroLocal)
    with pytest.raises(TaskNotFoundError):
        resource.task_status("1")
    with pytest.raises(TaskNotReadyError, match="Failed"):
        resource.task_result("2")
    resource.task_stop("2")


class _Resources(ctypes.Structure):
    _fields_ = [
        ("resources", ctypes.POINTER(ctypes.c_void_p)),
        ("length", ctypes.c_size_t),
    ]


class _MaestroBody(ctypes.Structure):  # pylint: disable=too-few-public-methods
    """QrmiPayload_MaestroLocal_Body."""

    _fields_ = [
        ("input", ctypes.c_char_p),
        ("job_type", ctypes.c_char_p),
        ("qubits", ctypes.c_uint32),
        ("simulator_type", ctypes.c_uint32),
        ("simulation_method", ctypes.c_uint32),
        ("observables", ctypes.c_char_p),
        ("config", ctypes.c_char_p),
    ]


class _Payload(ctypes.Structure):  # pylint: disable=too-few-public-methods
    """QrmiPayload with its QRMI_PAYLOAD_MAESTRO_LOCAL variant."""

    _fields_ = [("tag", ctypes.c_int), ("maestro", _MaestroBody)]


QRMI_PAYLOAD_MAESTRO_LOCAL = 5


@pytest.fixture(name="c_api")
def fixture_c_api():
    """Opt in with QRMI_TEST_LIBRARY pointing to the matching C library."""
    path = os.environ.get("QRMI_TEST_LIBRARY")
    if path is None:
        pytest.skip(
            "set QRMI_TEST_LIBRARY to the rebuilt libqrmi.so to check the C ABI"
        )
    api = ctypes.CDLL(path)
    signatures = {
        "qrmi_resource_new": ([ctypes.c_char_p, ctypes.c_int], ctypes.c_void_p),
        "qrmi_resource_free": ([ctypes.c_void_p], ctypes.c_int),
        "qrmi_resource_type": (
            [ctypes.c_void_p, ctypes.POINTER(ctypes.c_int)],
            ctypes.c_int,
        ),
        "qrmi_resource_id": (
            [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)],
            ctypes.c_int,
        ),
        "qrmi_resource_target": (
            [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p)],
            ctypes.c_int,
        ),
        "qrmi_resource_task_status": (
            [ctypes.c_void_p, ctypes.c_char_p, ctypes.POINTER(ctypes.c_int)],
            ctypes.c_int,
        ),
        "qrmi_get_last_error": ([], ctypes.c_void_p),
        "qrmi_get_last_error_kind": ([], ctypes.c_int),
        "qrmi_string_free": ([ctypes.c_void_p], ctypes.c_int),
        "qrmi_service_resources": ([ctypes.POINTER(_Resources)], ctypes.c_int),
        "qrmi_service_resources_free": ([ctypes.POINTER(_Resources)], ctypes.c_int),
        "qrmi_resource_task_start": (
            [
                ctypes.c_void_p,
                ctypes.POINTER(_Payload),
                ctypes.POINTER(ctypes.c_void_p),
            ],
            ctypes.c_int,
        ),
    }
    for name, (arguments, result) in signatures.items():
        function = getattr(api, name)
        function.argtypes = arguments
        function.restype = result
    return api


def test_c_error_codes_and_maestro_enum(c_api, server, monkeypatch):
    """Keep C error codes and the Maestro enum aligned with Python."""
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "invalid")
    assert not c_api.qrmi_resource_new(b"py_maestro", int(ResourceType.MaestroLocal))
    assert c_api.qrmi_get_last_error_kind() == 103  # ParseError
    message = c_api.qrmi_get_last_error()
    assert message
    try:
        assert b"ACQUISITION_TOKEN" in ctypes.string_at(message)
    finally:
        c_api.qrmi_string_free(message)

    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "8")
    server.extend(
        [
            ('API {"version":2,"command":"capabilities"}', "ERROR Unknown command"),
            ("SESSION 8 TASK 1 EXISTS", "OK NO"),
        ]
    )
    resource = c_api.qrmi_resource_new(b"py_maestro", int(ResourceType.MaestroLocal))
    assert resource
    try:
        kind = ctypes.c_int()
        assert c_api.qrmi_resource_type(resource, ctypes.byref(kind)) == 0
        assert kind.value == int(ResourceType.MaestroLocal)
        value = ctypes.c_void_p()
        assert c_api.qrmi_resource_target(resource, ctypes.byref(value)) == 105
        assert (
            c_api.qrmi_resource_task_status(resource, b"invalid", ctypes.byref(kind))
            == 109
        )
        assert (
            c_api.qrmi_resource_task_status(resource, b"1", ctypes.byref(kind)) == 112
        )
        assert c_api.qrmi_get_last_error_kind() == 112
    finally:
        c_api.qrmi_resource_free(resource)


def test_c_service_discovers_maestro(c_api, server, monkeypatch):
    """Return accessible Maestro resources through the shared C service."""
    monkeypatch.setenv("QRMI_JOB_QPU_RESOURCES", "py_maestro,offline_maestro")
    monkeypatch.setenv("QRMI_JOB_QPU_TYPES", "maestro-local,maestro-local")
    server.extend(
        [
            ('API {"version":2,"command":"status"}', "ERROR Unknown command"),
            ("PING", "OK YES"),
            ('API {"version":2,"command":"status"}', "ERROR Unknown command"),
            ("PING", "OK NO"),
        ]
    )
    resources = _Resources()
    assert c_api.qrmi_service_resources(ctypes.byref(resources)) == 0
    try:
        assert resources.length == 1
        resource = resources.resources[0]
        identifier = ctypes.c_void_p()
        assert c_api.qrmi_resource_id(resource, ctypes.byref(identifier)) == 0
        try:
            assert ctypes.string_at(identifier) == b"py_maestro"
        finally:
            c_api.qrmi_string_free(identifier)
        kind = ctypes.c_int()
        assert c_api.qrmi_resource_type(resource, ctypes.byref(kind)) == 0
        assert kind.value == int(ResourceType.MaestroLocal)
    finally:
        c_api.qrmi_service_resources_free(ctypes.byref(resources))


def test_target_negotiates_old_server(server):
    """Report unsupported capabilities when a legacy daemon rejects the API."""
    server.append(
        ('API {"version":2,"command":"capabilities"}', "ERROR Unknown command")
    )
    with pytest.raises(UnsupportedFunctionError):
        QuantumResource("py_maestro", ResourceType.MaestroLocal).target()


@pytest.mark.parametrize(
    "code,exception",
    [
        ("unsupported_capability", UnsupportedFunctionError),
        ("unsupported_operation", UnsupportedFunctionError),
        ("invalid_config", ConfigError),
        ("session_not_found", ConfigError),
    ],
)
def test_native_error_codes_reach_python_categories(
    server, monkeypatch, code, exception
):
    """Preserve native error categories and messages in Python exceptions."""
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "4")
    server.append(
        (
            'API {"version":2,"command":"logs","session_id":4,"task_id":8}',
            "OK "
            + json.dumps(
                {
                    "version": 2,
                    "ok": False,
                    "error": {"code": code, "message": "review probe"},
                }
            ),
        )
    )
    resource = QuantumResource("py_maestro", ResourceType.MaestroLocal)
    with pytest.raises(exception, match="review probe"):
        resource.task_logs("8")


NATIVE_REQUEST = '{"schema_version":2,"operation":"execute"}'
SUBMIT = (
    'API {"version":2,"command":"submit","session_id":3,"request":'
    + NATIVE_REQUEST
    + "}"
)
CAPABILITIES = (
    'API {"version":2,"command":"capabilities"}',
    'OK {"version":2,"ok":true,"native":{"schema_version":2}}',
)
UNRECOGNIZED_SUBMISSION_REPLIES = [
    'OK {"ok":true,"task_id":73}',
    "garbled response",
    "ERROR Unknown command",
    'OK {"version":3,"ok":true,"task_id":73}',
    'OK {"version":2,"ok":true}',
]


def native_payload():
    """A native request payload; legacy positional fields are unused."""
    return Payload.MaestroLocal(
        input=NATIVE_REQUEST,
        job_type="request",
        qubits=0,
        simulator_type=0,
        simulation_method=0,
        observables="",
        config="{}",
    )


@pytest.mark.parametrize("reply", UNRECOGNIZED_SUBMISSION_REPLIES)
def test_unrecognized_submit_reply_is_generic(server, monkeypatch, reply):
    """Only the generic error may follow a sent request without a valid refusal."""
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "3")
    server.extend([CAPABILITIES, (SUBMIT, reply)])
    resource = QuantumResource("py_maestro", ResourceType.MaestroLocal)
    with pytest.raises(QrmiError, match="may have been created") as raised:
        resource.task_start(native_payload())
    assert raised.type is QrmiError


def test_submit_refusal_keeps_its_category(server, monkeypatch):
    """A well-formed v2 refusal still proves that no task was created."""
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "3")
    refusal = {
        "version": 2,
        "ok": False,
        "error": {"code": "invalid_input", "message": "review probe"},
    }
    server.extend([CAPABILITIES, (SUBMIT, "OK " + json.dumps(refusal))])
    resource = QuantumResource("py_maestro", ResourceType.MaestroLocal)
    with pytest.raises(InvalidInputError, match="review probe"):
        resource.task_start(native_payload())


def test_c_submit_codes_follow_task_start_contract(c_api, server, monkeypatch):
    """C callers see QRMI_RETURN_CODE_ERROR (100) when a task may exist."""
    monkeypatch.setenv("py_maestro_QRMI_JOB_ACQUISITION_TOKEN", "3")
    refusal = (
        'OK {"version":2,"ok":false,"error":'
        '{"code":"invalid_input","message":"review probe"}}'
    )
    cases = [(reply, 100) for reply in UNRECOGNIZED_SUBMISSION_REPLIES]
    cases.append((refusal, 109))
    resource = c_api.qrmi_resource_new(b"py_maestro", int(ResourceType.MaestroLocal))
    assert resource
    try:
        for index, (reply, expected) in enumerate(cases):
            # Every protocol failure resets negotiation; a refusal keeps it.
            if index == 0 or cases[index - 1][1] == 100:
                server.append(CAPABILITIES)
            server.append((SUBMIT, reply))
            payload = _Payload(
                QRMI_PAYLOAD_MAESTRO_LOCAL,
                _MaestroBody(NATIVE_REQUEST.encode(), b"request", 0, 0, 0, b"", b"{}"),
            )
            task = ctypes.c_void_p()
            code = c_api.qrmi_resource_task_start(
                resource, ctypes.byref(payload), ctypes.byref(task)
            )
            assert code == expected, reply
    finally:
        c_api.qrmi_resource_free(resource)
