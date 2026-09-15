"""Unit tests for the Pulser service."""

from unittest.mock import MagicMock, call, patch

import pytest

from qrmi.pulser.service import QRMIService
from qrmi import ResourceType


class TestQRMIService:
    """Tests for the Pulser service."""

    @patch("qrmi.pulser.service.os.environ.get")
    def test_plugin_error_raises_runtime_error(self, mock_get):
        """Test plugin errors are propagated."""
        mock_get.return_value = "Failed to acquire resources"

        with pytest.raises(RuntimeError, match="Failed to acquire resources"):
            QRMIService()

    @patch("qrmi.pulser.service.QuantumResource")
    @patch("qrmi.pulser.service.get_job_qpu_resources_and_types")
    @patch("qrmi.pulser.service.os.environ.get")
    def test_accessible_resources_are_added(
        self,
        mock_environ,
        mock_get_resources,
        mock_resource,
    ):
        """Test accessible resources are stored."""
        mock_environ.return_value = None
        mock_get_resources.return_value = (
            ["ibm_backend"],
            ["qiskit-runtime-service"],
        )

        resource_instance = MagicMock()
        resource_instance.is_accessible.return_value = True
        mock_resource.return_value = resource_instance

        service = QRMIService()

        mock_resource.assert_called_once_with(
            "ibm_backend",
            ResourceType.IBMQiskitRuntimeService,
        )

        assert service.resources() == [resource_instance]
        assert service.resource("ibm_backend") == resource_instance

    @patch("qrmi.pulser.service.QuantumResource")
    @patch("qrmi.pulser.service.get_job_qpu_resources_and_types")
    @patch("qrmi.pulser.service.os.environ.get")
    def test_inaccessible_resources_are_ignored(
        self,
        mock_environ,
        mock_get_resources,
        mock_resource,
    ):
        """Test inaccessible resources are not stored."""
        mock_environ.return_value = None
        mock_get_resources.return_value = (
            ["ibm_backend"],
            ["qiskit-runtime-service"],
        )

        resource_instance = MagicMock()
        resource_instance.is_accessible.return_value = False
        mock_resource.return_value = resource_instance

        service = QRMIService()

        assert not service.resources()
        assert service.resource("ibm_backend") is None

    @patch("qrmi.pulser.service.logger")
    @patch("qrmi.pulser.service.QuantumResource")
    @patch("qrmi.pulser.service.get_job_qpu_resources_and_types")
    @patch("qrmi.pulser.service.os.environ.get")
    def test_unsupported_resource_type_is_skipped(
        self,
        mock_environ,
        mock_get_resources,
        mock_resource,
        mock_logger,
    ):
        """Test unsupported resource types are ignored."""
        mock_environ.return_value = None
        mock_get_resources.return_value = (
            ["resource1"],
            ["unsupported-type"],
        )

        service = QRMIService()

        mock_resource.assert_not_called()

        mock_logger.warning.assert_called_once_with(
            "Unsupported resource type: %s specified for %s",
            "unsupported-type",
            "resource1",
        )

        assert not service.resources()

    @patch("qrmi.pulser.service.QuantumResource")
    @patch("qrmi.pulser.service.get_job_qpu_resources_and_types")
    @patch("qrmi.pulser.service.os.environ.get")
    def test_runtime_error_from_is_accessible_is_rethrown(
        self,
        mock_environ,
        mock_get_resources,
        mock_resource,
    ):
        """Test accessibility errors are rethrown with resource context."""
        mock_environ.return_value = None
        mock_get_resources.return_value = (
            ["ibm_backend"],
            ["qiskit-runtime-service"],
        )

        resource_instance = MagicMock()
        resource_instance.is_accessible.side_effect = RuntimeError("Connection failed")
        mock_resource.return_value = resource_instance

        with pytest.raises(
            RuntimeError,
            match="ibm_backend is not accessible. Connection failed",
        ):
            QRMIService()

    @patch("qrmi.pulser.service.QuantumResource")
    @patch("qrmi.pulser.service.get_job_qpu_resources_and_types")
    @patch("qrmi.pulser.service.os.environ.get")
    def test_all_supported_resource_types(
        self,
        mock_environ,
        mock_get_resources,
        mock_resource,
    ):
        """Test mapping of QRMI resource types."""
        mock_environ.return_value = None

        mock_get_resources.return_value = (
            ["q1", "q2", "q3", "q4"],
            [
                "qiskit-runtime-service",
                "ibm-quantum-compute-service",
                "pasqal-cloud",
                "pasqal-local",
            ],
        )

        resource_instance = MagicMock()
        resource_instance.is_accessible.return_value = True
        mock_resource.return_value = resource_instance

        QRMIService()

        expected_calls = [
            call("q1", ResourceType.IBMQiskitRuntimeService),
            call("q2", ResourceType.IBMQuantumComputeService),
            call("q3", ResourceType.PasqalCloud),
            call("q4", ResourceType.PasqalLocal),
        ]

        assert mock_resource.call_args_list == expected_calls
