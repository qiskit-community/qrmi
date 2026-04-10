"""Tests for pulser connection integration."""

import json

import pulser
import pytest
from pulser.backend.remote import RemoteResultsError
from pulser.result import SampledResult

from qrmi import TaskStatus
from qrmi.pulser.connection import PulserQRMIConnection


class _TaskResult:
    def __init__(self, value):
        """Store raw task result payload."""
        self.value = value


class _FakeQRMI:
    def __init__(self, status: TaskStatus = TaskStatus.Completed):
        """Create a minimal QRMI stub."""
        self._status = status
        self.started: list[str] = []
        self.stopped: list[str] = []

    def task_start(self, _payload):
        """Track submitted payloads and return task id."""
        job_id = f"job-{len(self.started) + 1}"
        self.started.append(job_id)
        return job_id

    def task_status(self, _job_id):
        """Return configured task status."""
        return self._status

    @staticmethod
    def target():
        """Return target payload as abstract device representation."""
        return _TaskResult(pulser.MockDevice.to_abstract_repr())

    @staticmethod
    def task_result(_job_id):
        """Return a successful Pasqal-style counter payload."""
        return _TaskResult('{"counter":{"0":3}}')

    def task_stop(self, job_id):
        """Track stopped jobs."""
        self.stopped.append(job_id)


def _build_sequence() -> pulser.Sequence:
    register = pulser.Register.from_coordinates([(0.0, 0.0)])
    sequence = pulser.Sequence(register, pulser.MockDevice)
    sequence.declare_channel("rydberg", "rydberg_global")
    sequence.add(pulser.Pulse.ConstantPulse(100, 1.0, 0.0, 0.0), "rydberg")
    sequence.measure("ground-rydberg")
    return sequence


def test_supports_open_batch_is_false() -> None:
    """Return False for open batch support."""
    connection = PulserQRMIConnection(qrmi=_FakeQRMI())  # type: ignore[arg-type]
    assert connection.supports_open_batch() is False


def test_init_without_qrmi_uses_single_resource(monkeypatch) -> None:
    """Use the only scheduled QRMI resource when no explicit resource is given."""

    fake_qrmi = _FakeQRMI()

    class _FakeService:
        @staticmethod
        def resources():
            """Return one resource."""
            return [fake_qrmi]

    monkeypatch.setattr("qrmi.pulser.connection.QRMIService", _FakeService)
    connection = PulserQRMIConnection()
    assert connection._qrmi is fake_qrmi


def test_init_without_qrmi_raises_when_no_resource(monkeypatch) -> None:
    """Raise if no accessible resource exists in the current job context."""

    class _FakeService:
        @staticmethod
        def resources():
            """Return no resources."""
            return []

    monkeypatch.setattr("qrmi.pulser.connection.QRMIService", _FakeService)
    with pytest.raises(RuntimeError, match="No accessible QRMI resource found"):
        PulserQRMIConnection()


def test_init_without_qrmi_raises_for_many_resources(monkeypatch) -> None:
    """Raise if multiple resources are scheduled and no explicit one is selected."""

    class _FakeResource(_FakeQRMI):
        def __init__(self, resource_id: str) -> None:
            super().__init__()
            self._resource_id = resource_id

        def resource_id(self) -> str:
            """Return the fake resource id."""
            return self._resource_id

    class _FakeService:
        @staticmethod
        def resources():
            """Return multiple resources."""
            return [_FakeResource("EMU_FREE"), _FakeResource("PASQAL_LOCAL")]

    monkeypatch.setattr("qrmi.pulser.connection.QRMIService", _FakeService)
    with pytest.raises(ValueError, match="Multiple QRMI resources are available"):
        PulserQRMIConnection()


def test_submit_wait_false_returns_remote_results() -> None:
    """Return a remote-results handler with QRMI task IDs."""
    connection = PulserQRMIConnection(qrmi=_FakeQRMI())  # type: ignore[arg-type]

    remote_results = connection.submit(
        _build_sequence(),
        wait=False,
        job_params=[{"runs": 5}],
    )

    assert remote_results.batch_id == "job-1"
    assert remote_results.job_ids == ["job-1"]


def test_submit_wait_true_returns_legacy_payload_shape() -> None:
    """Return legacy raw payloads when wait=True."""
    connection = PulserQRMIConnection(qrmi=_FakeQRMI())  # type: ignore[arg-type]

    result = connection.submit(
        _build_sequence(),
        wait=True,
        job_params=[{"runs": 5}],
    )

    assert isinstance(result, list)
    assert json.loads(result[0])["counter"] == {"0": 3}


def test_remote_results_return_sampled_result() -> None:
    """Return sampled results from a completed QRMI task."""
    connection = PulserQRMIConnection(qrmi=_FakeQRMI())  # type: ignore[arg-type]
    remote_results = connection.submit(
        _build_sequence(),
        wait=False,
        job_params=[{"runs": 5}],
    )

    results = remote_results.results

    assert len(results) == 1
    assert isinstance(results[0], SampledResult)
    assert results[0].bitstring_counts == {"0": 3}


def test_remote_results_raise_when_task_is_running() -> None:
    """Raise when requesting results for non-completed QRMI tasks."""
    connection = PulserQRMIConnection(qrmi=_FakeQRMI(status=TaskStatus.Running))  # type: ignore[arg-type]
    remote_results = connection.submit(
        _build_sequence(),
        wait=False,
        job_params=[{"runs": 5}],
    )

    with pytest.raises(RemoteResultsError):
        _ = remote_results.results


def test_get_available_results_ignores_bad_payload() -> None:
    """Return no available results when completed payload is malformed."""

    class _BadResultQRMI(_FakeQRMI):
        @staticmethod
        def task_result(_job_id):
            """Return malformed payload."""
            return _TaskResult("not-json")

    connection = PulserQRMIConnection(qrmi=_BadResultQRMI())  # type: ignore[arg-type]
    remote_results = connection.submit(
        _build_sequence(),
        wait=False,
        job_params=[{"runs": 5}],
    )

    assert remote_results.get_available_results() == {}
