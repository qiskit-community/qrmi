import json
from unittest.mock import MagicMock, patch

from qrmi.primitives.ibm.target import get_target


class _FakeTarget:
    def __init__(self, value):
        self.value = value


class _FakeQRMI:
    def target(self):
        """Simulate a QRMI target response with a JSON payload."""
        return _FakeTarget(
            json.dumps(
                {
                    "configuration": {"backend_name": "fake"},
                    "properties": {"qubits": []},
                }
            )
        )


@patch("qrmi.primitives.ibm.target.convert_to_target")
@patch("qrmi.primitives.ibm.target.BackendProperties.from_dict")
@patch("qrmi.primitives.ibm.target.BackendConfiguration.from_dict")
def test_get_target_converts_backend_data(
    mock_configuration,
    mock_properties,
    mock_convert,
):
    """Convert QRMI target data into a Qiskit Target."""

    config = MagicMock()
    props = MagicMock()
    target = MagicMock()

    mock_configuration.return_value = config
    mock_properties.return_value = props
    mock_convert.return_value = target

    result = get_target(_FakeQRMI())

    mock_configuration.assert_called_once_with({"backend_name": "fake"})

    mock_properties.assert_called_once_with({"qubits": []})

    mock_convert.assert_called_once_with(config, props)

    assert result is target


def test_get_target_requests_target_from_qrmi():
    """Retrieve target information from the QRMI resource."""

    qrmi = MagicMock()

    qrmi.target.return_value = _FakeTarget(
        json.dumps(
            {
                "configuration": {},
                "properties": {},
            }
        )
    )

    with (
        patch("qrmi.primitives.ibm.target.BackendConfiguration.from_dict"),
        patch("qrmi.primitives.ibm.target.BackendProperties.from_dict"),
        patch("qrmi.primitives.ibm.target.convert_to_target"),
    ):
        get_target(qrmi)

    qrmi.target.assert_called_once_with()
