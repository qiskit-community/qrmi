"""Tests for Estimator V2 base class for IBM QRMI integration."""

import json

from qiskit import QuantumCircuit
from qiskit.circuit import Parameter
from qiskit.quantum_info import SparsePauliOp


from qrmi.primitives.base_estimator import QRMIBaseEstimatorV2
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


def test_run_uses_default_precision():
    """Verify that run() uses the default precision."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run([(qc, SparsePauliOp("Z"))])

    payload = json.loads(qrmi.payload.input)

    assert payload["precision"] == 0.015625


def test_run_uses_explicit_precision():
    """Verify that run() uses the supplied precision."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run(
        [(qc, SparsePauliOp("Z"))],
        precision=0.1,
    )

    payload = json.loads(qrmi.payload.input)

    assert payload["precision"] == 0.1


def test_run_includes_run_options():
    """Verify that the sampler's configured run options are included in the payload."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(
        qrmi,
        options={"run_options": {"test": True}},
    )

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run([(qc, SparsePauliOp("Z"))])

    payload = json.loads(qrmi.payload.input)

    assert payload["options"] == {"test": True}


def test_run_submits_estimator_payload():
    """Verify that the payload's program ID is submitted."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run([(qc, SparsePauliOp("Z"))])

    assert qrmi.payload.program_id == "estimator"


def test_run_returns_runtime_job():
    """Verify that run() returns a RuntimeJobV2 instance for the submitted task."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    job = estimator.run([(qc, SparsePauliOp("Z"))])

    assert isinstance(job, RuntimeJobV2)


def test_run_serializes_pub_without_parameters():
    """Verify that a circuit pub is serialised when no parameters are supplied."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run([(qc, SparsePauliOp("Z"))])

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"][0]

    assert len(pub) == 4
    assert pub[3] == 0.015625


def test_run_serializes_pub_without_parameters_no_precision():
    """Verify that a circuit pub using no precision is serialised when no parameters are supplied."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(
        qrmi,
        options={"default_precision": None},
    )

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run([(qc, SparsePauliOp("Z"))])

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"]

    assert len(pub) == 1


def test_run_serializes_pub_with_parameters():
    """Verify that the circuit parameter values are serialised when parameters are supplied."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    test_param = Parameter("test_param")

    qc = QuantumCircuit(1)
    qc.rx(test_param, 0)
    qc.measure_all()

    estimator.run(
        [
            (
                qc,
                SparsePauliOp("Z"),
                [0.5],
            )
        ]
    )

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"][0]

    assert len(pub) == 4
    assert pub[2] == [0.5]
    assert pub[3] == 0.015625


def test_run_serializes_pub_with_parameters_no_precision():
    """Verify that the circuit parameter values are serialised when parameters are using no precision."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(
        qrmi,
        options={"default_precision": None},
    )

    test_param = Parameter("test_param")

    qc = QuantumCircuit(1)
    qc.rx(test_param, 0)
    qc.measure_all()

    estimator.run(
        [
            (
                qc,
                SparsePauliOp("Z"),
                [0.5],
            )
        ]
    )

    payload = json.loads(qrmi.payload.input)

    pub = payload["pubs"][0]

    assert len(pub) == 3
    assert pub[2] == [0.5]


def test_run_serializes_multiple_pubs():
    """Verifies all supplied estimator pubs are serialised."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc1 = QuantumCircuit(1)
    qc2 = QuantumCircuit(1)

    estimator.run(
        [
            (qc1, SparsePauliOp("Z")),
            (qc2, SparsePauliOp("X")),
        ]
    )

    payload = json.loads(qrmi.payload.input)

    assert len(payload["pubs"]) == 2


def test_run_starts_task():
    """Verify that a generated payload is submitted to the QRMI resource."""
    qrmi = _FakeQRMI()

    estimator = QRMIBaseEstimatorV2(qrmi)

    qc = QuantumCircuit(1)
    qc.measure_all()

    estimator.run([(qc, SparsePauliOp("Z"))])

    assert qrmi.payload is not None
