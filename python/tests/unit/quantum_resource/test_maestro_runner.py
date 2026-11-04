"""Runner outcomes and schema acceptance; no simulation service is needed."""

import json
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock

import jsonschema
import pytest

from qrmi import ResourceType, TaskStatus
from qrmi.maestro import payload_from_input
from qrmi.tools.task_runner import main

EXAMPLES = Path(__file__).resolve().parents[4] / "examples/task_runner/maestro_local"


def test_oqtopus_cannot_bypass_maestro_validation():
    """An OQTOPUS field must not rescue an invalid Maestro request."""
    schema = json.loads(
        (EXAMPLES.parents[2] / "qrmi_payload_v1_schema.json").read_text()
    )
    validator = jsonschema.Draft202012Validator(schema)
    validator.check_schema(schema)
    assert validator.is_valid(
        {"job_spec": '{"job_type":"sampling","program":["qasm"]}'}
    )
    for invalid in [
        {"schema_version": 1},
        {"request": 123},
        {"job_type": "execute"},
        {"job_type": "REQUEST", "input": 123},
    ]:
        assert not validator.is_valid({**invalid, "job_spec": "{}"})


@pytest.mark.parametrize(
    "path", sorted(EXAMPLES.glob("native-*.json")), ids=lambda p: p.stem
)
def test_native_examples_match_schema_and_helper(path):
    """Accept every native example in both the schema and payload helper."""
    document = json.loads(path.read_text())
    schema = json.loads(
        (EXAMPLES.parents[2] / "qrmi_payload_v1_schema.json").read_text()
    )
    jsonschema.Draft202012Validator(schema).validate(document)
    assert json.loads(payload_from_input(document).input) == document


@pytest.mark.parametrize(
    "status,logs_fail,result_fail,cancelled,message",
    [
        (TaskStatus.Completed, False, False, False, None),
        (TaskStatus.Failed, False, False, False, "Failed"),
        (TaskStatus.Cancelled, False, False, False, "Cancelled"),
        (TaskStatus.Failed, True, False, False, "Failed"),
        (TaskStatus.Completed, False, True, False, "result unavailable"),
        (TaskStatus.Running, False, False, True, "cancelled"),
    ],
)
def test_runner_outcome_and_cleanup(
    monkeypatch,
    tmp_path,
    caplog,
    capsys,
    status,
    logs_fail,
    result_fail,
    cancelled,
    message,
):
    """Report task outcomes and collect diagnostics before cleanup."""
    calls = []
    resource = Mock()
    resource.task_start.return_value = "1"
    resource.task_status.return_value = status
    resource.task_result.return_value = SimpleNamespace(value="native-result")
    if result_fail:
        resource.task_result.side_effect = RuntimeError("result unavailable")

    def logs(_task):
        calls.append("logs")
        if logs_fail:
            raise RuntimeError("logs unavailable")
        return "native failure detail"

    resource.task_logs.side_effect = logs
    resource.task_stop.side_effect = lambda _task: calls.append("stop")
    monkeypatch.setattr(main, "QuantumResource", lambda *_: resource)
    monkeypatch.setattr(main.signal, "signal", lambda *_: None)
    monkeypatch.setattr(
        main.App, "_find_qpu_type", lambda *_: ResourceType.MaestroLocal
    )
    source = tmp_path / "input.json"
    source.write_text('{"schema_version":2,"operation":"execute"}')
    app = main.App("test", str(source), None)
    if cancelled:
        app._signal_handler(None, None)
    result = app.run()
    assert result == (1 if message else 0)
    assert not app.is_running
    resource.task_stop.assert_called_once_with("1")
    if message:
        assert message in caplog.text
        assert (
            "logs unavailable" if logs_fail else "native failure detail"
        ) in caplog.text
        assert calls == ["logs", "stop"]
        assert "native failure detail" not in capsys.readouterr().out
    else:
        assert calls == ["stop"]
        assert "native-result" in capsys.readouterr().out


def test_cli_entrypoint_returns_app_exit_status(monkeypatch):
    """Propagate the application's failure status through the CLI entry point."""
    monkeypatch.setattr(main.sys, "argv", ["task_runner", "test", "input.json"])
    monkeypatch.setattr(main.App, "run", lambda _: 1)
    monkeypatch.setattr(main.signal, "signal", lambda *_: None)
    assert main.run() == 1


def test_native_and_legacy_schema_matches_helper():
    """Keep schema validation consistent with native and legacy parsing."""
    root = Path(__file__).resolve().parents[4]
    schema = json.loads((root / "qrmi_payload_v1_schema.json").read_text())
    validator = jsonschema.Draft202012Validator(schema)
    validator.check_schema(schema)
    native = {"schema_version": 2, "operation": "execute"}
    legacy = {
        "job_type": "execute",
        "input": "qasm",
        "qubits": 1,
        "simulator_type": 0,
        "simulation_method": 0,
    }
    for value in [
        native,
        {"request": native},
        {"request": json.dumps(native)},
        {"job_type": "request", "input": native},
        {"job_type": "request", "input": json.dumps(native)},
        {**legacy, "job_type": "request", "input": native},
        legacy,
        {**legacy, "config": {"shots": 4}},
        {**legacy, "config": '{"shots":4}'},
    ]:
        validator.validate(value)
        payload_from_input(value)
    for value in [
        {"job_type": "execute", "input": "qasm"},
        {**legacy, "qubits": 0},
        {**legacy, "input": {}},
        {"job_type": "request", "input": {"schema_version": 1}},
        {"job_type": "request"},
    ]:
        assert not validator.is_valid(value), value
        with pytest.raises(ValueError):
            payload_from_input(value)


def test_shared_payload_contract_cases():
    """Apply the shared Rust, Python, and JSON-schema payload cases."""
    root = Path(__file__).resolve().parents[4]
    schema = json.loads((root / "qrmi_payload_v1_schema.json").read_text())
    validator = jsonschema.Draft202012Validator(schema)
    cases = json.loads((root / "tests/fixtures/maestro_payloads.json").read_text())
    for case in cases:
        assert validator.is_valid(case["input"]) == case.get(
            "schema_valid", case["valid"]
        ), case["name"]
        if case["valid"]:
            payload = payload_from_input(case["input"])
            assert json.loads(payload.config) == json.loads(case["config"]), case[
                "name"
            ]
        else:
            with pytest.raises(ValueError):
                payload_from_input(case["input"])


@pytest.mark.parametrize("kind", ["execute", "estimate"])
@pytest.mark.parametrize(
    "field", ["input", "qubits", "simulator_type", "simulation_method"]
)
def test_missing_legacy_fields_have_descriptive_errors(kind, field):
    """Name the missing legacy field for each supported legacy operation."""
    document = {
        "job_type": kind,
        "input": "qasm",
        "qubits": 1,
        "simulator_type": 0,
        "simulation_method": 0,
    }
    del document[field]
    with pytest.raises(ValueError, match=f"^Missing legacy Maestro fields: {field}$"):
        payload_from_input(document)


def test_missing_native_and_legacy_fields():
    """Describe absent native input and all missing legacy fields."""
    with pytest.raises(ValueError, match="^Missing Maestro request field: input$"):
        payload_from_input({"job_type": "REQUEST"})
    with pytest.raises(
        ValueError,
        match="^Missing legacy Maestro fields: input, qubits, simulator_type, simulation_method$",
    ):
        payload_from_input({"job_type": "execute"})


def test_runner_rejects_missing_field(monkeypatch, tmp_path, caplog):
    """Log a missing input field without submitting or cancelling a task."""
    resource = Mock()
    monkeypatch.setattr(main, "QuantumResource", lambda *_: resource)
    monkeypatch.setattr(main.signal, "signal", lambda *_: None)
    monkeypatch.setattr(
        main.App, "_find_qpu_type", lambda *_: ResourceType.MaestroLocal
    )
    source = tmp_path / "missing-input.json"
    source.write_text('{"job_type":"request"}')
    assert main.App("test", str(source), None).run() == 1
    assert "Missing Maestro request field: input" in caplog.text
    resource.task_start.assert_not_called()
    resource.task_stop.assert_not_called()
