import json

from qiskit import QuantumCircuit
from qiskit.circuit import Parameter

from qrmi.primitives.base_sampler import QRMIBaseSamplerV2
from qrmi.primitives.runtime_job_v2 import RuntimeJobV2


class _FakeQRMI:
    def __init__(self):
        """Create a minimal QRMI stub."""
        self.payload = None

    def task_start(self, payload):
        """Track payload and return job id."""
        self.payload = payload
        return "job-123"

    def task_stop(self, _job_id):
        """No-op stop."""
        return None


def test_run_uses_default_shots():
    """Verify that run() uses the default shot count."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc])

    payload = json.loads(qrmi.payload.input)

    assert payload["shots"] == 1024


def test_run_uses_explicit_shots():
    """Verify that run() uses the supplied shot count."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc], shots=2048)

    payload = json.loads(qrmi.payload.input)

    assert payload["shots"] == 2048


def test_run_includes_run_options():
    """Verify that the sampler's configured run options are included in the payload."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(
        qrmi,
        options={"run_options": {"test": True}},
    )

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc])

    payload = json.loads(qrmi.payload.input)

    assert payload["options"] == {"test": True}


def test_run_submits_sampler_payload():
    """Verify that the payload's program ID is submitted."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc])

    assert qrmi.payload.program_id == "sampler"


def test_run_returns_runtime_job():
    """Verify that run() returns a a RuntimeJobV2 instance for the submitted task."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    job = sampler.run([qc])

    assert isinstance(job, RuntimeJobV2)


def test_run_serializes_pub_without_parameters():
    """Verify that a circuit pub is serialised when no parameters are supplied."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc], shots=100)

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"][0]

    assert len(pub) == 3
    assert pub[1] is None
    assert pub[2] == 100


def test_run_serializes_pub_without_parameters_no_shots():
    """Verify that a circuit pub using no shots is serialised when no parameters are supplied."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(
        qrmi,
        options={"default_shots": None},
    )

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc])

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"]

    assert len(pub) == 1


def test_run_serializes_pub_with_parameters():
    """Verify that the circuit parameter values are serialised when parameters are supplied."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    test_param = Parameter("test_param")

    qc = QuantumCircuit(1)
    qc.rx(test_param, 0)
    qc.measure_all()

    sampler.run([(qc, [0.25])], shots=50)

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"][0]

    assert len(pub) == 3
    assert pub[1] == [0.25]
    assert pub[2] == 50


def test_run_serializes_pub_with_parameters_no_shots():
    """Verify that the circuit parameter values are serialised when parameters are using no shots."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(
        qrmi,
        options={"default_shots": None},
    )

    test_param = Parameter("test_param")

    qc = QuantumCircuit(1)
    qc.rx(test_param, 0)
    qc.measure_all()

    sampler.run([(qc, [0.25])])

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"][0]

    assert len(pub) == 2
    assert pub[1] == [0.25]


def test_run_serializes_multiple_pubs():
    """Verifies all supplied estimator pubs are serialised."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc1 = QuantumCircuit(1)
    qc2 = QuantumCircuit(1)

    sampler.run(
        [
            qc1,
            qc2,
        ]
    )

    payload = json.loads(qrmi.payload.input)

    assert len(payload["pubs"]) == 2


def test_run_starts_task():
    """Verify that a generated payload is submitted to the QRMI resource."""
    qrmi = _FakeQRMI()

    sampler = QRMIBaseSamplerV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    sampler.run([qc])

    assert qrmi.payload is not None
