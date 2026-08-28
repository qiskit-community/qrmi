# python/tests/unit/primitives/iqm/test_iqm_job.py

from unittest.mock import MagicMock, patch
from uuid import uuid4

import pytest
from qiskit import QuantumCircuit
from qiskit.providers import JobStatus, Options

from qrmi.qiskit_iqm.iqm_provider import (
    IQMJobCustom,
    QRMIBackend,
    IQMProvider,
)
from qrmi import TaskStatus, ResourceType

# ---------------------------------------------------------------------------
# Testing IQMJobCustom
# ---------------------------------------------------------------------------


def test_submit_not_supported():
    backend = MagicMock()
    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=100,
    )

    with pytest.raises(
        NotImplementedError,
        match="Job is submitted automatically",
    ):
        job.submit()


@pytest.mark.parametrize(
    ("task_status", "expected"),
    [
        (TaskStatus.Queued, JobStatus.QUEUED),
        (TaskStatus.Running, JobStatus.RUNNING),
        (TaskStatus.Completed, JobStatus.DONE),
        (TaskStatus.Failed, JobStatus.ERROR),
        (TaskStatus.Cancelled, JobStatus.CANCELLED),
    ],
)
def test_status(task_status, expected):
    qrmi = MagicMock()
    qrmi.task_status.return_value = task_status

    backend = MagicMock()
    backend.qrmi = qrmi

    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=10,
    )

    assert job.status() == expected


@patch("qrmi.qiskit_iqm.iqm_provider._format_results")
@patch("qrmi.qiskit_iqm.iqm_provider.json.loads")
def test_result_completed(
    mock_json_loads,
    mock_format_results,
):
    measurements = {"m": [["0"], ["1"]]}

    mock_json_loads.return_value = {
        "measurements": measurements,
    }

    counts = {"0": 1, "1": 1}

    mock_format_results.return_value = [
        (
            "test_circuit",
            ["0", "1"],
            counts,
        )
    ]

    qrmi = MagicMock()
    qrmi.task_status.return_value = TaskStatus.Completed

    payload = MagicMock()
    payload.value = '{"measurements": {}}'

    qrmi.task_result.return_value = payload

    backend = MagicMock()
    backend.qrmi = qrmi
    backend.name = "iqm_backend"

    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=2,
    )

    result = job.result()

    assert result.success

    qrmi.task_result.assert_called_once()
    mock_format_results.assert_called_once()


@patch("qrmi.qiskit_iqm.iqm_provider._format_results")
@patch("qrmi.qiskit_iqm.iqm_provider.json.loads")
def test_result_uses_cache(
    mock_json_loads,
    mock_format_results,
):
    mock_json_loads.return_value = {"measurements": {}}

    mock_format_results.return_value = [("circ", ["0"], {"0": 1})]

    qrmi = MagicMock()
    qrmi.task_status.return_value = TaskStatus.Completed

    payload = MagicMock()
    payload.value = "{}"

    qrmi.task_result.return_value = payload

    backend = MagicMock()
    backend.qrmi = qrmi
    backend.name = "backend"

    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=1,
    )

    job.result()
    job.result()

    qrmi.task_result.assert_called_once()
    mock_format_results.assert_called_once()


@patch("qrmi.qiskit_iqm.iqm_provider.time.sleep")
def test_result_timeout(mock_sleep):
    qrmi = MagicMock()
    qrmi.task_status.return_value = TaskStatus.Running

    backend = MagicMock()
    backend.qrmi = qrmi

    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=1,
    )

    with pytest.raises(TimeoutError):
        job.result(
            timeout=0.01,
            poll_interval=0,
        )


def test_result_failed_job():
    qrmi = MagicMock()
    qrmi.task_status.return_value = TaskStatus.Failed

    backend = MagicMock()
    backend.qrmi = qrmi
    backend.name = "backend"

    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=1,
    )

    result = job.result()

    assert not result.success
    assert result.results == []


@patch("qrmi.qiskit_iqm.iqm_provider._format_results")
@patch("qrmi.qiskit_iqm.iqm_provider.json.loads")
def test_result_includes_metadata(
    mock_json_loads,
    mock_format_results,
):
    mock_json_loads.return_value = {
        "measurements",
    }


def test_cancel_warns():
    backend = MagicMock()

    job = IQMJobCustom(
        backend=backend,
        job_id=uuid4(),
        circuits=MagicMock(),
        shots=1,
    )

    with pytest.warns(UserWarning, match="cancel\\(\\) is not supported"):
        assert job.cancel() is False


# ---------------------------------------------------------------------------
# Testing QRMIBackend
# ---------------------------------------------------------------------------


@pytest.fixture
def qrmi_backend():
    backend = QRMIBackend.__new__(QRMIBackend)

    backend._idx_to_qb = {0: "QB1"}
    backend._use_default_calibration_set = False
    backend._calibration_set_id = uuid4()
    backend._max_circuits = None
    backend.target_json = {"dynamic_quantum_architecture": {}}

    return backend


def test_default_options():
    opts = QRMIBackend._default_options()

    assert isinstance(opts, Options)


def test_max_circuits_property(qrmi_backend):
    assert qrmi_backend.max_circuits is None

    qrmi_backend.max_circuits = 25

    assert qrmi_backend.max_circuits == 25


@patch("qrmi.qiskit_iqm.iqm_provider.IQMJobCustom")
def test_run_submits_job(mock_job):
    backend = MagicMock(spec=QRMIBackend)

    run_request = MagicMock()
    run_request.model_dump_json.return_value = '{"test": true}'
    run_request.circuits = []
    run_request.shots = 100

    backend.create_run_request.return_value = run_request

    qrmi = MagicMock()
    qrmi.task_start.return_value = "job-id"

    backend.qrmi = qrmi

    QRMIBackend.run(backend, QuantumCircuit(1))

    qrmi.task_start.assert_called_once()
    mock_job.assert_called_once()


def test_create_run_request_empty_list(qrmi_backend):
    with pytest.raises(
        ValueError,
        match="Empty list of circuits",
    ):
        qrmi_backend.create_run_request([])


def test_create_run_request_callback_called(qrmi_backend):
    circuit = QuantumCircuit(1)

    callback = MagicMock()

    with (
        patch.object(qrmi_backend, "_serialize_circuit"),
        patch("qrmi.qiskit_iqm.iqm_provider._build_run_request") as build_request,
    ):
        build_request.return_value = MagicMock()

        qrmi_backend.create_run_request(
            circuit,
            circuit_callback=callback,
        )

    callback.assert_called_once()


def test_create_run_request_unknown_option_warning(qrmi_backend):
    circuit = QuantumCircuit(1)

    with (
        pytest.warns(UserWarning, match="Unknown backend option"),
        patch.object(qrmi_backend, "_serialize_circuit"),
        patch("qrmi.qiskit_iqm.iqm_provider._build_run_request") as build_request,
    ):
        build_request.return_value = MagicMock()

        qrmi_backend.create_run_request(
            circuit,
            random_option=True,
        )


def test_create_run_request_deprecated_option_warning(qrmi_backend):
    circuit = QuantumCircuit(1)

    with (
        pytest.warns(DeprecationWarning),
        patch.object(qrmi_backend, "_serialize_circuit"),
        patch("qrmi.qiskit_iqm.iqm_provider._build_run_request") as build_request,
    ):
        build_request.return_value = MagicMock()

        qrmi_backend.create_run_request(
            circuit,
            heralding_mode="none",
        )


def test_calibration_change_warning(qrmi_backend):
    circuit = QuantumCircuit(1)

    qrmi_backend._use_default_calibration_set = True
    qrmi_backend._calibration_set_id = uuid4()

    dqa = MagicMock()
    dqa.calibration_set_id = uuid4()

    with (
        pytest.warns(UserWarning, match="calibration set has changed"),
        patch.object(qrmi_backend, "_serialize_circuit"),
        patch(
            "qrmi.qiskit_iqm.iqm_provider.DynamicQuantumArchitecture.model_validate",
            return_value=dqa,
        ),
        patch("qrmi.qiskit_iqm.iqm_provider._build_run_request") as build_request,
    ):
        build_request.return_value = MagicMock()

        qrmi_backend.create_run_request(circuit)


@patch("qrmi.qiskit_iqm.iqm_provider._build_run_request")
def test_create_run_request_wraps_validation_error(
    mock_build,
    qrmi_backend,
):
    from iqm.iqm_client import CircuitValidationError

    circuit = QuantumCircuit(1)

    mock_build.side_effect = CircuitValidationError("Invalid circuit")

    with (
        patch.object(qrmi_backend, "_serialize_circuit"),
        pytest.raises(
            CircuitValidationError,
            match="Make sure circuits were transpiled",
        ),
    ):
        qrmi_backend.create_run_request(circuit)


def test_serialize_circuit_uses_default_mapping(qrmi_backend):
    circuit = QuantumCircuit(1)

    with patch.object(
        qrmi_backend,
        "_serialize_circuit",
        return_value="serialized",
    ) as mock_serialise:
        result = qrmi_backend.serialize_circuit(circuit)

    assert result == "serialized"

    mock_serialise.assert_called_once_with(
        circuit,
        qrmi_backend._idx_to_qb,
    )


@patch("qrmi.qiskit_iqm.iqm_provider.Circuit")
@patch("qrmi.qiskit_iqm.iqm_provider.to_json_dict")
@patch("qrmi.qiskit_iqm.iqm_provider.serialize_instructions")
def test_serialize_circuit_success(
    mock_serialize,
    mock_json,
    mock_circuit_cls,
    qrmi_backend,
):
    circuit = QuantumCircuit(1, name="test")

    mock_serialize.return_value = ["instr"]
    mock_json.return_value = {"foo": "bar"}

    qrmi_backend._serialize_circuit(
        circuit,
        {0: "QB1"},
    )

    mock_circuit_cls.assert_called_once()


@patch("qrmi.qiskit_iqm.iqm_provider.Circuit")
@patch("qrmi.qiskit_iqm.iqm_provider.to_json_dict")
@patch("qrmi.qiskit_iqm.iqm_provider.serialize_instructions")
def test_serialize_circuit_invalid_metadata(
    mock_serialize,
    mock_json,
    mock_circuit_cls,
    qrmi_backend,
):
    circuit = QuantumCircuit(1, name="test")
    circuit.metadata = {}

    mock_serialize.return_value = ["instr"]
    mock_json.side_effect = ValueError

    with pytest.warns(
        UserWarning,
        match="Metadata of circuit test was dropped",
    ):
        qrmi_backend._serialize_circuit(
            circuit,
            {0: "QB1"},
        )

    _, kwargs = mock_circuit_cls.call_args
    assert kwargs["metadata"] is None


# ---------------------------------------------------------------------------
# Testing IQMProvider
# ---------------------------------------------------------------------------


@patch("qrmi.qiskit_iqm.iqm_provider.get_job_qpu_resources_and_types")
def test_init_filters_iqm_resources(mock_resources):
    mock_resources.return_value = (
        [
            "iqm:garnet",
            "ibm:brisbane",
            "iqm:deneb",
        ],
        [
            "iqm-server",
            "qiskit-runtime-service",
            "iqm-server",
        ],
    )

    provider = IQMProvider()

    assert provider._iqm_resources == [
        "iqm:garnet",
        "iqm:deneb",
    ]


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_default(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = ["iqm:garnet"]

    provider.get_backend()

    mock_resource.assert_called_once_with(
        "iqm_garnet",
        ResourceType.IQMServer,
    )

    mock_backend.assert_called_once()


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_named_backend(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = [
        "iqm:garnet",
        "iqm:deneb",
    ]

    provider.get_backend(name="iqm:deneb")

    mock_resource.assert_called_once_with(
        "iqm_deneb",
        ResourceType.IQMServer,
    )

    mock_backend.assert_called_once()


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_invalid_name_warns(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = ["iqm:garnet"]

    with pytest.warns(
        UserWarning,
        match="is not available",
    ):
        provider.get_backend(name="does-not-exist")

    mock_resource.assert_called_once_with(
        "iqm_garnet",
        ResourceType.IQMServer,
    )


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_with_calibration_set(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = ["iqm:garnet"]

    calset_id = uuid4()

    provider.get_backend(
        calibration_set_id=calset_id,
    )

    mock_resource.assert_called_once_with(
        f"iqm_garnet,{calset_id}",
        ResourceType.IQMServer,
    )


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_forwards_calibration_set(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = ["iqm:garnet"]

    resource = MagicMock()
    mock_resource.return_value = resource

    calset_id = uuid4()

    provider.get_backend(
        calibration_set_id=calset_id,
    )

    mock_backend.assert_called_once_with(
        resource,
        calibration_set_id=calset_id,
        use_metrics=False,
    )


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_forwards_use_metrics(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = ["iqm:garnet"]

    resource = MagicMock()
    mock_resource.return_value = resource

    provider.get_backend(use_metrics=True)

    mock_backend.assert_called_once_with(
        resource,
        calibration_set_id=None,
        use_metrics=True,
    )


@patch("qrmi.qiskit_iqm.iqm_provider.QRMIBackend")
@patch("qrmi.qiskit_iqm.iqm_provider.QuantumResource")
def test_get_backend_returns_backend(
    mock_resource,
    mock_backend,
):
    provider = IQMProvider.__new__(IQMProvider)
    provider._iqm_resources = ["iqm:garnet"]

    backend = MagicMock()
    mock_backend.return_value = backend

    result = provider.get_backend()

    assert result is backend
