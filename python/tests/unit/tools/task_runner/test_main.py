"""Unit tests for the task_runner.main module."""

import json
import os
import time
from unittest.mock import Mock

from logging import DEBUG, ERROR, INFO

import pytest
from qrmi import ResourceType, TaskStatus
from qrmi.tools.task_runner.main import App, _get_loglevel


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


def test_find_qpu_type(monkeypatch):
    """Test that _find_qpu_type returns the correct ResourceType for a given QPU name."""
    app = App("qpu1", "input.json", "output.json")

    monkeypatch.setattr(
        "qrmi.tools.task_runner.main.get_job_qpu_resources_and_types",
        lambda: (
            ["qpu1"],
            ["pasqal-cloud"],
        ),
    )

    assert app._find_qpu_type("qpu1") == ResourceType.PasqalCloud


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


def test_exit_callback_without_qrmi():
    """Test that _exit_callback does nothing when _qrmi is None."""
    app = App("qpu", "input.json", None)

    app._qrmi = None

    app._exit_callback()


def test_exit_callback_stops_task():
    """Test that _exit_callback stops the task when _qrmi is not None."""
    app = App("qpu", "input.json", None)

    qrmi = Mock()
    app._qrmi = qrmi
    app._task_id = "task-1"

    app._exit_callback()

    qrmi.task_stop.assert_called_once_with("task-1")


def test_exit_callback_writes_output(tmp_path):
    """Test that _exit_callback writes the task result to the output file when _succeeded is True."""
    outfile = tmp_path / "results.json"

    app = App("qpu", "input.json", str(outfile))

    qrmi = Mock()
    qrmi.task_result.return_value.value = '{"result": 1}'

    app._qrmi = qrmi
    app._task_id = "task-1"
    app._succeeded = True

    app._exit_callback()

    assert outfile.read_text() == '{"result": 1}'


def test_is_running_property():
    """Test that the is_running property returns the correct value."""
    app = App("qpu", "input.json", None)

    assert app.is_running is True


def test_task_id_property():
    """Test that the task_id property returns the correct value."""
    app = App("qpu", "input.json", None)

    app._task_id = "abc123"

    assert app.task_id == "abc123"


def test_run_raises_for_unwritable_directory(monkeypatch):
    """Test that run raises a RuntimeError when the output directory is not writable."""
    app = App("qpu", "input.json", "/bad/path/output.json")

    monkeypatch.setattr(os, "access", lambda *_: False)

    with pytest.raises(RuntimeError, match="cannot be created"):
        app.run()


def test_run_completes_successfully(monkeypatch, tmp_path):
    """Test that run completes successfully when the task succeeds."""
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
    qrmi.task_status.return_value = TaskStatus.Completed

    print_mock = Mock()
    monkeypatch.setattr("builtins.print", print_mock)

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

    app.run()

    assert app._task_id == "task-1"
    assert app._succeeded is True


def test_run_handles_failed_task(monkeypatch, tmp_path):
    """Test that run handles a failed task by setting _succeeded to False."""
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
    qrmi.task_status.return_value = TaskStatus.Failed

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

    app.run()

    assert app._succeeded is True
