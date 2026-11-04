"""Unit tests for the task_runner.main module."""

import json
import os
import time
from unittest.mock import Mock, call

from logging import DEBUG, ERROR, INFO

import pytest
from qrmi import ResourceType, TaskStatus
from qrmi.tools.task_runner.main import App, _get_loglevel


@pytest.fixture(autouse=True)
def isolate_signal_handlers(monkeypatch):
    """Keep App construction from replacing the test process signal handlers."""
    monkeypatch.setattr("qrmi.tools.task_runner.main.signal.signal", lambda *_: None)


def test_get_loglevel_default(monkeypatch):
    """Test that the default log level is INFO when SRUN_DEBUG is not set."""
    monkeypatch.delenv("SRUN_DEBUG", raising=False)

    assert _get_loglevel() == INFO


def test_get_loglevel_quiet(monkeypatch):
    """Test that the log level is ERROR when SRUN_DEBUG is set to 2."""
    monkeypatch.setenv("SRUN_DEBUG", "2")

    assert _get_loglevel() == ERROR


def test_get_loglevel_verbose(monkeypatch):
    """Test that the log level is DEBUG when SRUN_DEBUG is set to 4 or higher."""
    monkeypatch.setenv("SRUN_DEBUG", "4")

    assert _get_loglevel() == DEBUG


def test_get_loglevel_invalid(monkeypatch):
    """Test that the log level is INFO when SRUN_DEBUG is set to an invalid value."""
    monkeypatch.setenv("SRUN_DEBUG", "invalid")

    assert _get_loglevel() == INFO


def test_signal_handler_stops_application():
    """Test that the signal handler sets is_running to False."""
    app = App("qpu", "input.json", "output.json")

    assert app.is_running is True

    app._signal_handler(None, None)

    assert app.is_running is False


@pytest.mark.parametrize(
    ("resource_name", "resource_type"),
    [
        ("pasqal-cloud", ResourceType.PasqalCloud),
        ("maestro-local", ResourceType.MaestroLocal),
    ],
)
def test_find_qpu_type(monkeypatch, resource_name, resource_type):
    """Test that _find_qpu_type returns the correct ResourceType for a given QPU name."""
    app = App("qpu1", "input.json", "output.json")

    monkeypatch.setattr(
        "qrmi.tools.task_runner.main.get_job_qpu_resources_and_types",
        lambda: (
            ["qpu1"],
            [resource_name],
        ),
    )

    assert app._find_qpu_type("qpu1") == resource_type


def test_find_qpu_type_not_found(monkeypatch):
    """Test that _find_qpu_type raises a ValueError when the QPU name is not found."""
    app = App("missing", "input.json", "output.json")

    monkeypatch.setattr(
        "qrmi.tools.task_runner.main.get_job_qpu_resources_and_types",
        lambda: (
            ["qpu1"],
            ["pasqal-cloud"],
        ),
    )

    with pytest.raises(ValueError, match="missing is not available"):
        app._find_qpu_type("missing")


def test_cleanup_without_qrmi():
    """Test that cleanup is safe when resource creation failed."""
    app = App("qpu", "input.json", None)

    app._qrmi = None

    app._cleanup()


def test_cleanup_stops_task():
    """Test that cleanup stops a submitted task without retrieving its result."""
    app = App("qpu", "input.json", None)

    qrmi = Mock()
    app._qrmi = qrmi
    app._task_id = "task-1"

    app._cleanup()

    qrmi.task_stop.assert_called_once_with("task-1")
    qrmi.task_result.assert_not_called()


def test_cleanup_without_task_id():
    """Test that cleanup does not stop a task when submission failed."""
    app = App("qpu", "input.json", None)
    qrmi = Mock()
    app._qrmi = qrmi

    app._cleanup()

    qrmi.task_stop.assert_not_called()


def test_is_running_property():
    """Test that the is_running property returns the correct value."""
    app = App("qpu", "input.json", None)

    assert app.is_running is True


def test_task_id_property():
    """Test that the task_id property returns the correct value."""
    app = App("qpu", "input.json", None)

    app._task_id = "abc123"

    assert app.task_id == "abc123"


def test_run_fails_for_unwritable_directory(monkeypatch, caplog):
    """Test that an unwritable output produces an error and a failing exit status."""
    app = App("qpu", "input.json", "/bad/path/output.json")

    monkeypatch.setattr(os, "access", lambda *_: False)

    assert app.run() == 1
    assert "cannot be created" in caplog.text
    assert app.is_running is False
    assert app._qrmi is None


@pytest.mark.parametrize("to_file", [False, True])
def test_run_completes_successfully(monkeypatch, tmp_path, capsys, to_file):
    """Test that a successful run writes one result before cleaning up the task."""
    input_file = tmp_path / "input.json"

    input_file.write_text(
        json.dumps(
            {
                "sequence": {},
                "job_runs": 1,
            }
        )
    )

    output_file = tmp_path / "results.json"
    app = App("qpu", str(input_file), str(output_file) if to_file else None)

    qrmi = Mock()
    qrmi.task_start.return_value = "task-1"
    qrmi.task_status.return_value = TaskStatus.Completed
    qrmi.task_result.return_value.value = '{"result": 1}'

    monkeypatch.setattr(
        app,
        "_find_qpu_type",
        lambda _: ResourceType.PasqalCloud,
    )

    monkeypatch.setattr(
        "qrmi.tools.task_runner.main.QuantumResource",
        lambda *_: qrmi,
    )

    monkeypatch.setattr(time, "sleep", lambda *_: None)

    assert app.run() == 0

    assert app._task_id == "task-1"
    assert app._succeeded is True
    assert app.is_running is False
    qrmi.task_result.assert_called_once_with("task-1")
    qrmi.task_stop.assert_called_once_with("task-1")
    assert qrmi.method_calls[-2:] == [
        call.task_result("task-1"),
        call.task_stop("task-1"),
    ]
    if to_file:
        assert output_file.read_text() == '{"result": 1}'
        assert capsys.readouterr().out == ""
    else:
        assert capsys.readouterr().out == '{"result": 1}\n'
        assert not output_file.exists()


@pytest.mark.parametrize("status", [TaskStatus.Failed, TaskStatus.Cancelled])
def test_run_handles_failed_task(monkeypatch, tmp_path, caplog, capsys, status):
    """Test that failed runs report diagnostics before cleanup and return failure."""
    input_file = tmp_path / "input.json"

    input_file.write_text(
        json.dumps(
            {
                "sequence": {},
                "job_runs": 1,
            }
        )
    )

    app = App("qpu", str(input_file), None)

    qrmi = Mock()
    qrmi.task_start.return_value = "task-1"
    qrmi.task_status.return_value = status
    qrmi.task_logs.return_value = "backend diagnostic"

    monkeypatch.setattr(
        app,
        "_find_qpu_type",
        lambda _: ResourceType.PasqalCloud,
    )

    monkeypatch.setattr(
        "qrmi.tools.task_runner.main.QuantumResource",
        lambda *_: qrmi,
    )

    monkeypatch.setattr(time, "sleep", lambda *_: None)

    assert app.run() == 1
    assert app._succeeded is False
    assert app.is_running is False
    assert "backend diagnostic" in caplog.text
    assert capsys.readouterr().out == ""
    qrmi.task_result.assert_not_called()
    qrmi.task_stop.assert_called_once_with("task-1")
    assert qrmi.method_calls[-2:] == [
        call.task_logs("task-1"),
        call.task_stop("task-1"),
    ]


def test_run_retries_after_status_exception(monkeypatch, tmp_path):
    """Test that run retries after a temporary exception when checking task status."""
    input_file = tmp_path / "input.json"
    input_file.write_text(
        json.dumps(
            {
                "sequence": {},
                "job_runs": 1,
            }
        )
    )

    app = App("qpu", str(input_file), None)

    qrmi = Mock()
    qrmi.task_start.return_value = "task-1"

    qrmi.task_status.side_effect = [
        RuntimeError("temporary"),
        TaskStatus.Completed,
    ]
    qrmi.task_result.return_value.value = '{"result": 1}'

    monkeypatch.setattr(
        app,
        "_find_qpu_type",
        lambda _: ResourceType.PasqalCloud,
    )

    monkeypatch.setattr(
        "qrmi.tools.task_runner.main.QuantumResource",
        lambda *_: qrmi,
    )

    monkeypatch.setattr(time, "sleep", lambda *_: None)

    assert app.run() == 0

    assert app._succeeded is True
    assert qrmi.task_status.call_count == 2
    qrmi.task_result.assert_called_once_with("task-1")
    qrmi.task_stop.assert_called_once_with("task-1")
