"""Tests for QRMI Pasqal SamplerV2 integration."""

import json
from unittest.mock import MagicMock, patch

from qiskit.circuit import QuantumCircuit
from qiskit.primitives import PrimitiveResult
from qiskit.providers import JobStatus
from pulser import DigitalAnalogDevice, MockDevice

import pytest

from qrmi import TaskStatus
from qrmi.primitives.pasqal import sampler as pasqal_sampler
from qrmi.primitives.pasqal.sampler import (
    QPPSamplerV2,
    QRMIPasqalBackend,
    QRMIPasqalJob,
)
from qrmi.primitives.pasqal.target import get_device


class _TaskResult:
    def __init__(self, value):
        """Store raw task result payload."""
        self.value = value


class _Seq:
    def __init__(self):
        """Create sequence stub."""
        self.values = None

    def build(self, **values):
        """Capture build values and return self."""
        self.values = values
        return self

    def to_abstract_repr(self):
        """Return serialized sequence payload."""
        return "serialized-seq"


class _FakeQRMI:
    def __init__(self):
        """Create a minimal QRMI stub."""
        self.payloads = []
        self.task_stop_called = False

    def task_start(self, payload):
        """Track payload and return job id."""
        self.payloads.append(payload)
        return "job-1"

    def task_status(self, _job_id):
        """Return completed status for all jobs."""
        return TaskStatus.Completed

    def task_result(self, _job_id):
        """Return successful Pasqal-style counter payload."""
        return _TaskResult('{"counter": {"00": 3, "11": 1}}')

    @staticmethod
    def target():
        """Return target payload as abstract device representation."""
        return _TaskResult(
            json.dumps(
                [
                    {
                        "device_type": "DUMMY",
                        "specs": MockDevice.to_abstract_repr(),
                    }
                ]
            )
        )

    def task_stop(self, _job_id):
        """No-op stop."""
        self.task_stop_called = True
        return None

    def resource_id(self):
        """Return emulator-style resource identifier."""
        return "EMU_FREE"


def _patch_sequence_build(monkeypatch):
    seq = _Seq()
    monkeypatch.setattr(
        pasqal_sampler, "get_register_from_circuit", lambda _qc: object()
    )
    monkeypatch.setattr(
        pasqal_sampler,
        "gen_seq",
        lambda analog_register, device, circuit: seq,
    )
    return seq


def test_backend_run_returns_job_and_uses_target_device(monkeypatch):
    """Return a job object and use target device lookup."""
    qrmi = _FakeQRMI()
    seq = _patch_sequence_build(monkeypatch)
    device_calls = {"count": 0}

    def _get_device(_qrmi):
        device_calls["count"] += 1
        assert _qrmi is qrmi
        return object()

    monkeypatch.setattr(pasqal_sampler, "get_device", _get_device)
    backend = QRMIPasqalBackend(
        qrmi=qrmi,
        options={"default_shots": 77, "run_options": {"poll_interval_seconds": 0.0}},
    )
    job = backend.run(QuantumCircuit(1))

    assert isinstance(job, QRMIPasqalJob)
    assert device_calls["count"] == 1
    assert qrmi.payloads[0].sequence == "serialized-seq"
    assert qrmi.payloads[0].job_runs == 77
    assert seq.values is None


def test_job_result_returns_primitive_result():
    """Return a PrimitiveResult for completed QRMI jobs."""
    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    result = job.result()

    assert isinstance(result, PrimitiveResult)
    assert result[0].data.counts == {"00": 3, "11": 1}
    assert job.status() == JobStatus.DONE


def test_run_options_flow_to_job(monkeypatch):
    """Propagate run options to the created job."""
    qrmi = _FakeQRMI()
    _patch_sequence_build(monkeypatch)
    monkeypatch.setattr(pasqal_sampler, "get_device", lambda _qrmi: object())

    backend = QRMIPasqalBackend(
        qrmi=qrmi,
        options={
            "run_options": {
                "poll_interval_seconds": 0.25,
                "timeout_seconds": 5.0,
                "delete_job": True,
            }
        },
    )
    job = backend.run(QuantumCircuit(1), shots=12)

    assert job._poll_interval_seconds == 0.25
    assert job._timeout_seconds == 5.0
    assert job._delete_job is True
    assert qrmi.payloads[0].job_runs == 12


@patch("qrmi.primitives.pasqal.sampler.get_device")
@patch("qrmi.primitives.pasqal.sampler.get_register_from_circuit")
@patch("qrmi.primitives.pasqal.sampler.gen_seq")
def test_run_passes_values_to_sequence_build(
    mock_gen_seq,
    _mock_get_register,
    _mock_get_device,
):
    """Verify parameter values are passed to sequence.build()."""

    qrmi = MagicMock()
    qrmi.task_start.return_value = "job-123"
    qrmi.resource_id.return_value = "test-backend"

    sequence = MagicMock()
    built_sequence = MagicMock()

    mock_gen_seq.return_value = sequence
    sequence.build.return_value = built_sequence
    built_sequence.to_abstract_repr.return_value = "abstract-sequence"

    backend = QRMIPasqalBackend(qrmi)

    circuit = MagicMock()

    values = {
        "duration": 100,
        "detuning": 5.0,
    }

    backend.run(
        circuit,
        shots=10,
        values=values,
    )

    sequence.build.assert_called_once_with(
        duration=100,
        detuning=5.0,
    )


def test_qpp_sampler_v2_returns_job(monkeypatch):
    """Run provider SamplerV2 through QRMI backend and return a job."""
    qrmi = _FakeQRMI()
    _patch_sequence_build(monkeypatch)
    monkeypatch.setattr(pasqal_sampler, "get_device", lambda _qrmi: object())

    sampler = QPPSamplerV2(
        qrmi=qrmi,
        options={"run_options": {"poll_interval_seconds": 0.0}},
    )
    job = sampler.run([QuantumCircuit(1)], shots=9)

    assert isinstance(job, QRMIPasqalJob)
    assert job.result()[0].data.counts == {"00": 3, "11": 1}


def test_get_device_falls_back_to_dad_for_emul():
    """Return DigitalAnalogDevice when emulator does not expose device specs."""

    class _NoSpecsQRMI:
        @staticmethod
        def resource_id():
            """QRMI resource ID."""
            return "EMU_FREE"

    assert get_device(_NoSpecsQRMI()) is DigitalAnalogDevice


def test_get_device_match_resource_id():
    """Return the right Device when the resource ID matches an actual device i.e. from the cloud"""

    class _NoSpecsQRMI:
        @staticmethod
        def resource_id():
            """QRMI resource ID."""
            return "FRESNEL"

        @staticmethod
        def target():
            """Return target payload as abstract device representation."""
            target_return = [
                {
                    "device_type": "OTHER",
                    "specs": DigitalAnalogDevice.to_abstract_repr(),
                },
                {"device_type": "FRESNEL", "specs": MockDevice.to_abstract_repr()},
            ]

            return _TaskResult(json.dumps(target_return))

    assert get_device(_NoSpecsQRMI()) == MockDevice


def test_get_device_pasqal_local():
    """Return the right Device when the resource ID is PASQAL_LOCAL"""

    class _NoSpecsQRMI:
        @staticmethod
        def resource_id():
            """QRMI resource ID."""
            return "PASQAL_LOCAL"

        @staticmethod
        def target():
            """Return target payload as abstract device representation."""
            target_return = [
                {
                    "device_type": "FRESNEL_LOCAL",
                    "specs": MockDevice.to_abstract_repr(),
                },
            ]

            return _TaskResult(json.dumps(target_return))

    assert get_device(_NoSpecsQRMI()) == MockDevice


def test_normalize_pasqal_payload_with_string():
    """Return a dictionary when the payload is a JSON string."""
    payload_str = '{"counter": {"00": 3, "11": 1}}'
    normalized = pasqal_sampler._normalize_pasqal_payload(payload_str)
    assert isinstance(normalized, dict)
    assert normalized["counter"] == {"00": 3, "11": 1}


def test_normalize_pasqal_payload_with_dict():
    """Return the same dictionary when the payload is already a dict."""
    payload_dict = {"counter": {"00": 3, "11": 1}}
    normalized = pasqal_sampler._normalize_pasqal_payload(payload_dict)
    assert normalized is payload_dict


def test_normalize_pasqal_payload_with_invalid_type():
    """Raise TypeError when the payload is neither a string nor a dict."""
    with pytest.raises(TypeError):
        pasqal_sampler._normalize_pasqal_payload(42)


def test_normalize_pasqal_payload_with_invalid_json():
    """Raise TypeError when the payload is a string but not a valid JSON object."""
    with pytest.raises(TypeError):
        pasqal_sampler._normalize_pasqal_payload('["not", "a", "dict"]')


def test_extract_counts_returns_valid_counts():
    """Return a dictionary of counts when the counter payload is valid."""
    payload = {"counter": {"00": 3, "11": 1}}
    counts = pasqal_sampler._extract_counts(payload)
    assert counts == {"00": 3, "11": 1}


def test_extract_counts_raises_error_invalid_counter():
    """Raise RuntimeError when the counter payload is not a dict."""
    payload = {"counter": "not a dict"}
    with pytest.raises(RuntimeError):
        pasqal_sampler._extract_counts(payload)


def test_extract_counts_raises_error_no_valid_counts():
    """Raise RuntimeError when there are no valid counts in the counter payload."""
    payload = {"counter": {"00": "not a number", "11": None}}
    with pytest.raises(RuntimeError):
        pasqal_sampler._extract_counts(payload)


def test_cancel_calls_task_stop():
    """Verify that canceling a job calls the QRMI task_stop method."""
    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    job.cancel()
    assert qrmi.task_stop_called


def test_result_returns_cached_result_jobstatus_done():
    """Verify that result() returns the cached result when job status is DONE."""
    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    cached_result = MagicMock()

    job._result = cached_result
    job._last_status = JobStatus.DONE

    assert job.result() is cached_result


@patch("qrmi.primitives.pasqal.sampler._extract_counts")
@patch("qrmi.primitives.pasqal.sampler._normalize_pasqal_payload")
def test_result_returns_primitive_result(
    mock_normalize,
    mock_extract_counts,
):
    """Return a PrimitiveResult when the job completes."""

    qrmi = MagicMock()

    qrmi.task_result.return_value.value = "raw"

    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    job._last_status = JobStatus.DONE
    job.status = MagicMock(return_value=JobStatus.DONE)

    mock_normalize.return_value = {"data": "parsed"}
    mock_extract_counts.return_value = {"00": 100}

    result = job.result()

    assert isinstance(result, PrimitiveResult)

    qrmi.task_result.assert_called_once_with("job-1")


@patch("qrmi.primitives.pasqal.sampler.time.sleep")
@patch("qrmi.primitives.pasqal.sampler.time.time")
def test_result_raises_timeout(
    mock_time,
    _mock_sleep,
):
    """Raise TimeoutError when task execution exceeds timeout."""

    job = QRMIPasqalJob(
        qrmi=MagicMock(),
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    job._timeout_seconds = 5

    job.status = MagicMock(return_value=JobStatus.RUNNING)

    mock_time.side_effect = [0, 10]

    with pytest.raises(TimeoutError):
        job.result()


def test_result_raises_when_job_fails():
    """Raise RuntimeError when the job ends unsuccessfully."""

    job = QRMIPasqalJob(
        qrmi=MagicMock(),
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    job._last_status = JobStatus.ERROR

    job.status = MagicMock(return_value=JobStatus.ERROR)

    with pytest.raises(RuntimeError, match="ended with status"):
        job.result()


@patch("qrmi.primitives.pasqal.sampler.time.sleep")
@patch("qrmi.primitives.pasqal.sampler._extract_counts", return_value={"00": 100})
@patch(
    "qrmi.primitives.pasqal.sampler._normalize_pasqal_payload",
    return_value={"result": "parsed"},
)
def test_result_polls_until_done(
    _mock_normalize,
    _mock_extract_counts,
    mock_sleep,
):
    """Poll job status until a final state is reached."""

    qrmi = MagicMock()
    qrmi.task_result.return_value.value = "raw-result"

    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    statuses = [
        JobStatus.RUNNING,
        JobStatus.RUNNING,
        JobStatus.DONE,
    ]

    def fake_status():
        status = statuses.pop(0)
        job._last_status = status
        return status

    job.status = MagicMock(side_effect=fake_status)

    result = job.result()

    assert result.metadata["status"] == "DONE"
    assert job.status.call_count == 3
    qrmi.task_result.assert_called_once_with("job-1")
    assert mock_sleep.call_count == 2


def test_result_is_cached_after_fetch():
    """Cache decoded results after the first retrieval."""

    qrmi = MagicMock()

    qrmi.task_result.return_value.value = "raw"

    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    job.status = MagicMock(return_value=JobStatus.DONE)
    job._last_status = JobStatus.DONE

    with (
        patch(
            "qrmi.primitives.pasqal.sampler._normalize_pasqal_payload", return_value={}
        ),
        patch("qrmi.primitives.pasqal.sampler._extract_counts", return_value={}),
    ):
        job.result()
        job.result()

    qrmi.task_result.assert_called_once()


def test_result_populates_metadata():
    """Populate job metadata in the returned PrimitiveResult."""
    qrmi = MagicMock()

    qrmi.task_result.return_value.value = "raw"

    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    job.status = MagicMock(return_value=JobStatus.DONE)
    job._last_status = JobStatus.DONE

    with (
        patch(
            "qrmi.primitives.pasqal.sampler._normalize_pasqal_payload", return_value={}
        ),
        patch("qrmi.primitives.pasqal.sampler._extract_counts", return_value={}),
    ):
        result = job.result()

    assert result.metadata["status"] == "DONE"
    assert result.metadata["success"] is True
    assert result.metadata["backend_name"] == "EMU_FREE"
    assert result.metadata["job_id"] == "job-1"


def test_done_returns_true_for_completed_job():
    """Verify that done() returns True for completed jobs."""

    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    qrmi.task_status = lambda job_id: TaskStatus.Completed

    assert job.done() is True


def test_running_returns_true_for_running_job():
    """Verify that running() returns True for running jobs."""

    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    qrmi.task_status = lambda job_id: TaskStatus.Running

    assert job.running() is True


def test_cancelled_returns_true_for_cancelled_job():
    """Verify that cancelled() returns True for cancelled jobs."""

    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    qrmi.task_status = lambda job_id: TaskStatus.Cancelled

    assert job.cancelled() is True


def test_in_final_state_returns_true_for_completed_job():
    """Verify that in_final_state() returns True for completed jobs."""

    qrmi = _FakeQRMI()
    job = QRMIPasqalJob(
        qrmi=qrmi,
        job_id="job-1",
        backend_name="EMU_FREE",
        poll_interval_seconds=0.0,
    )

    qrmi.task_status = lambda job_id: TaskStatus.Completed

    assert job.in_final_state() is True
