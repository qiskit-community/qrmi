from unittest.mock import MagicMock, patch

import pytest

from qrmi.primitives.service import QRMIService


def test_constructor_raises_on_plugin_error():
    """Verify service raises RuntimeError when a QRMI plugin error is reported."""

    with patch.dict(
        "os.environ",
        {"QRMI_PLUGIN_ERROR": "plugin failed"},
        clear=False,
    ):
        with pytest.raises(RuntimeError, match="plugin failed"):
            QRMIService()


@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_resources_empty_when_no_qpus(mock_get_qpus):
    """Verify service returns an empty resource list when no QPUs are discovered."""

    mock_get_qpus.return_value = ([], [])

    service = QRMIService()

    assert not service.resources()


@patch("qrmi.primitives.service.QuantumResource")
@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_accessible_ibm_resource_is_added(
    mock_get_qpus,
    mock_quantum_resource,
):
    """Verify service adds accessible IBM resources."""

    mock_get_qpus.return_value = (
        ["ibm-qpu"],
        ["ibm-quantum-system"],
    )

    resource = MagicMock()
    resource.is_accessible.return_value = True
    mock_quantum_resource.return_value = resource

    service = QRMIService()

    assert service.resources() == [resource]
    assert service.resource("ibm-qpu") is resource


@patch("qrmi.primitives.service.QuantumResource")
@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_inaccessible_resource_is_ignored(
    mock_get_qpus,
    mock_quantum_resource,
):
    """Verify service ignores inaccessible resources."""

    mock_get_qpus.return_value = (
        ["ibm-qpu"],
        ["ibm-quantum-system"],
    )

    resource = MagicMock()
    resource.is_accessible.return_value = False
    mock_quantum_resource.return_value = resource

    service = QRMIService()

    assert service.resources() == []
    assert service.resource("ibm-qpu") is None


@patch("qrmi.primitives.service.QuantumResource")
@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_unsupported_resource_type_is_ignored(
    mock_get_qpus,
    mock_quantum_resource,
):
    """Verify service ignores unsupported QRMI resource types."""

    mock_get_qpus.return_value = (
        ["unsupported"],
        ["unsupported-type"],
    )

    service = QRMIService()

    assert not service.resources()
    mock_quantum_resource.assert_not_called()


@patch("qrmi.primitives.service.QuantumResource")
@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_resource_returns_matching_resource(
    mock_get_qpus,
    mock_quantum_resource,
):
    """Verify service returns the resource matching a given identifier."""

    mock_get_qpus.return_value = (
        ["ibm-qpu"],
        ["ibm-quantum-system"],
    )

    resource = MagicMock()
    resource.is_accessible.return_value = True
    mock_quantum_resource.return_value = resource

    service = QRMIService()

    assert service.resource("ibm-qpu") is resource


@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_resource_returns_none_when_not_found(
    mock_get_qpus,
):
    """Verify service returns None when a matching resource is not found."""

    mock_get_qpus.return_value = ([], [])

    service = QRMIService()

    assert service.resource("missing") is None


@patch("qrmi.primitives.service.QuantumResource")
@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_resources_returns_all_accessible_resources(
    mock_get_qpus,
    mock_quantum_resource,
):
    """Verify service returns all accessible resources."""

    mock_get_qpus.return_value = (
        ["qpu1", "qpu2"],
        ["ibm-quantum-system", "pasqal-cloud"],
    )

    resource1 = MagicMock()
    resource1.is_accessible.return_value = True

    resource2 = MagicMock()
    resource2.is_accessible.return_value = True

    mock_quantum_resource.side_effect = [resource1, resource2]

    service = QRMIService()

    assert service.resources() == [resource1, resource2]


@pytest.mark.parametrize(
    "resource_type",
    [
        "ibm-quantum-system",
        "qiskit-runtime-service",
        "pasqal-cloud",
        "pasqal-local",
        "alice-bob-felis",
        "iqm-server",
    ],
)
@patch("qrmi.primitives.service.QuantumResource")
@patch("qrmi.primitives.service.get_job_qpu_resources_and_types")
def test_supported_resource_types_are_constructed(
    mock_get_qpus,
    mock_quantum_resource,
    resource_type,
):
    """Verify service constructs QRMI resources for supported resource types."""

    mock_get_qpus.return_value = (
        ["resource"],
        [resource_type],
    )

    resource = MagicMock()
    resource.is_accessible.return_value = True

    mock_quantum_resource.return_value = resource

    service = QRMIService()

    assert service.resources() == [resource]
