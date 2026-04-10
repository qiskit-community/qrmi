"""Tests for QRMI Pasqal SamplerV2 integration."""

from qiskit.circuit import QuantumCircuit
from qiskit.primitives import PrimitiveResult
from qiskit.providers import JobStatus
from pulser import DigitalAnalogDevice

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

    def task_stop(self, _job_id):
        """No-op stop."""
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


def test_get_device_falls_back_to_dad_without_specs():
    """Return DigitalAnalogDevice when emulator does not expose device specs."""

    class _NoSpecsQRMI:
        @staticmethod
        def target():
            """Raise QRMI device-specs-not-available error."""
            raise RuntimeError(
                "Status: 404 Not Found, Fail "
                '{"status":"fail","message":"Not found.","code":"CD1202","data":'
                '{"description":"Device specs are not available for emulators."}}'
            )

    assert get_device(_NoSpecsQRMI()) is DigitalAnalogDevice
