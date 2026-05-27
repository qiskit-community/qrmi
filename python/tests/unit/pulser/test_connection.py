"""Tests for pulser connection integration."""

import json

import pulser
import pytest
from pulser.backend.remote import RemoteResultsError
from pulser.result import SampledResult
from qrmi import ResourceType, TaskStatus
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

    def resource_type(self) -> ResourceType:
        """Return a Pasqal resource type"""
        return ResourceType.PasqalCloud

    def task_status(self, _job_id):
        """Return configured task status."""
        return self._status

    @staticmethod
    def target():
        """Return target payload as abstract device representation."""
        return _TaskResult(
            json.dumps(
                [
                    {
                        "device_type": "DUMMY",
                        "specs": pulser.MockDevice.to_abstract_repr(),
                    }
                ]
            )
        )

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
    assert len(remote_results.results) == 1
    assert remote_results.results[0].bitstring_counts == {"0": 3}


def test_submit_wait_true_returns_remote_results() -> None:
    """Return remote-results payloads when wait=True."""
    connection = PulserQRMIConnection(qrmi=_FakeQRMI())  # type: ignore[arg-type]

    remote_results = connection.submit(
        _build_sequence(),
        wait=True,
        job_params=[{"runs": 5}],
    )

    assert remote_results.batch_id == "job-1"
    assert remote_results.job_ids == ["job-1"]
    assert len(remote_results.results) == 1
    assert remote_results.results[0].bitstring_counts == {"0": 3}


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


def test_wrong_device_type() -> None:
    """Test that the QRMI connection raises a ValueError when the device type is not Pasqal."""

    class _BadDeviceTypeQRMI(_FakeQRMI):
        def resource_type(self):
            """Return a non Pasqal resource."""
            return ResourceType.IBMQuantumSystem

    with pytest.raises(TypeError):
        PulserQRMIConnection(_BadDeviceTypeQRMI)


def test_fetch_available_devices() -> None:
    """Test the parsing from the qrmi.target interface to the Connexion.fetch_available_devices method"""

    connection = PulserQRMIConnection(qrmi=_FakeQRMI())  # type: ignore[arg-type]
    devices = connection.fetch_available_devices()
    assert len(devices) == 1
    assert "DUMMY" in devices
    assert isinstance(devices["DUMMY"], pulser.devices.VirtualDevice)


def test_get_batch_status_running_any_job_is_running() -> None:
    """Return BatchStatus.RUNNING when at least one job has RUNNING status, even if others are PENDING or DONE."""

    class _MixedStatusQRMI(_FakeQRMI):
        _statuses = [TaskStatus.Running, TaskStatus.Queued, TaskStatus.Completed]
        _call_count = 0

        def task_status(self, _job_id):
            """Return mixed statuses across jobs."""
            status = self._statuses[self._call_count % len(self._statuses)]
            self._call_count += 1
            return status

    connection = PulserQRMIConnection(qrmi=_MixedStatusQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2|job-3"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.RUNNING


def test_get_batch_status_running_no_running_pending() -> None:
    """Return BatchStatus.RUNNING when no jobs are RUNNING but at least one job has PENDING status."""

    class _PendingStatusQRMI(_FakeQRMI):
        _statuses = [TaskStatus.Queued, TaskStatus.Completed]
        _call_count = 0

        def task_status(self, _job_id):
            """Return PENDING for first job, DONE for second."""
            status = self._statuses[self._call_count % len(self._statuses)]
            self._call_count += 1
            return status

    connection = PulserQRMIConnection(qrmi=_PendingStatusQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.RUNNING


def test_batch_status_canceled_all_jobs_are_canceled() -> None:
    """Return BatchStatus.CANCELED when all jobs have CANCELED status."""

    class _CanceledStatusQRMI(_FakeQRMI):
        def task_status(self, _job_id):
            """Return CANCELED for all jobs."""
            return TaskStatus.Cancelled

    connection = PulserQRMIConnection(qrmi=_CanceledStatusQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2|job-3"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.CANCELED


def test_batch_status_not_canceled_one_job_canceled() -> None:
    """Return BatchStatus.DONE (not CANCELED) when only one job has CANCELED status and others are DONE."""

    class _PartialCanceledStatusQRMI(_FakeQRMI):
        _statuses = [TaskStatus.Cancelled, TaskStatus.Completed, TaskStatus.Completed]
        _call_count = 0

        def task_status(self, _job_id):
            """Return CANCELED for first job, DONE for the rest."""
            status = self._statuses[self._call_count % len(self._statuses)]
            self._call_count += 1
            return status

    connection = PulserQRMIConnection(qrmi=_PartialCanceledStatusQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2|job-3"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result != BatchStatus.CANCELED
    assert result == BatchStatus.DONE


def test_batch_status_error_when_all_jobs_error_status() -> None:
    """Return BatchStatus.ERROR when all jobs have ERROR status."""

    class _ErrorStatusQRMI(_FakeQRMI):
        def task_status(self, _job_id):
            """Return ERROR for all jobs."""
            return TaskStatus.Failed

    connection = PulserQRMIConnection(qrmi=_ErrorStatusQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2|job-3"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.ERROR


def test_batch_status_done_all_jobs_completed() -> None:
    """Return BatchStatus.DONE when all jobs have completed successfully."""

    class _AllCompletedQRMI(_FakeQRMI):
        def task_status(self, _job_id):
            """Return COMPLETED for all jobs."""
            return TaskStatus.Completed

    connection = PulserQRMIConnection(qrmi=_AllCompletedQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2|job-3"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.DONE


def test_batch_status_running_mix_running_error() -> None:
    """Return BatchStatus.RUNNING when a mix of RUNNING and ERROR statuses exist."""

    class _RunningAndErrorQRMI(_FakeQRMI):
        _statuses = [TaskStatus.Running, TaskStatus.Failed]
        _call_count = 0

        def task_status(self, _job_id):
            """Return alternating RUNNING and ERROR statuses."""
            status = self._statuses[self._call_count % len(self._statuses)]
            self._call_count += 1
            return status

    connection = PulserQRMIConnection(qrmi=_RunningAndErrorQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.RUNNING


def test_batch_status_done_mix_done_canceled_error() -> None:
    """
    Return BatchStatus.DONE when there is a mix of DONE, CANCELED, and ERROR statuses (not all CANCELED, not all ERROR).
    """

    class _MixedDoneCanceledErrorQRMI(_FakeQRMI):
        _statuses = [TaskStatus.Completed, TaskStatus.Cancelled, TaskStatus.Failed]
        _call_count = 0

        def task_status(self, _job_id):
            """Return DONE, CANCELED, and ERROR for successive jobs."""
            status = self._statuses[self._call_count % len(self._statuses)]
            self._call_count += 1
            return status

    connection = PulserQRMIConnection(qrmi=_MixedDoneCanceledErrorQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2|job-3"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.DONE


def test_batch_status_running_mix_pending_error() -> None:
    """Return BatchStatus.RUNNING when a mix of PENDING and ERROR statuses exist but none are RUNNING."""

    class _PendingAndErrorQRMI(_FakeQRMI):
        _statuses = [TaskStatus.Queued, TaskStatus.Failed]
        _call_count = 0

        def task_status(self, _job_id):
            """Return alternating PENDING and ERROR statuses."""
            status = self._statuses[self._call_count % len(self._statuses)]
            self._call_count += 1
            return status

    connection = PulserQRMIConnection(qrmi=_PendingAndErrorQRMI())  # type: ignore[arg-type]
    batch_id = "job-1|job-2"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.RUNNING


def test_batch_status_done_single_job_completed() -> None:
    """Return BatchStatus.DONE when a single job has a DONE status."""

    class _SingleDoneQRMI(_FakeQRMI):
        def task_status(self, _job_id):
            """Return COMPLETED for the single job."""
            return TaskStatus.Completed

    connection = PulserQRMIConnection(qrmi=_SingleDoneQRMI())  # type: ignore[arg-type]
    batch_id = "job-1"

    from pulser.backend.remote import BatchStatus

    result = connection._get_batch_status(batch_id)

    assert result == BatchStatus.DONE
