import json
from unittest.mock import MagicMock, patch

import pytest
from pulser import DigitalAnalogDevice
from pulser.exceptions.serialization import DeserializeDeviceError

from qrmi.primitives.pasqal.target import (
    _parse_available_devices,
    get_device,
    get_target,
)


def test_parse_available_devices_success():
    """Verify that _parse_available_devices correctly parses valid JSON and deserializes devices."""
    qrmi = MagicMock()

    payload = [
        {
            "device_type": "mock_device",
            "specs": {"some": "spec"},
        }
    ]

    qrmi.target.return_value.value = json.dumps(payload)

    device = MagicMock()

    with patch(
        "qrmi.primitives.pasqal.target.deserialize_device",
        return_value=device,
    ) as mock_deserialize:
        result = _parse_available_devices(qrmi)

    assert result == {"mock_device": device}
    mock_deserialize.assert_called_once_with({"some": "spec"})


def test_parse_available_devices_invalid_json():
    """Verify that _parse_available_devices handles invalid JSON and returns empty dictionary."""
    qrmi = MagicMock()
    qrmi.target.return_value.value = "not valid json"

    result = _parse_available_devices(qrmi)

    assert result == {}


def test_parse_available_devices_skips_failed_deserializations():
    """Verify that _parse_available_devices skips devices that fail to deserialize and logs the error."""
    qrmi = MagicMock()

    payload = [
        {
            "device_type": "invalid_device",
            "specs": {"invalid": True},
        },
        {
            "device_type": "valid_device",
            "specs": {"valid": True},
        },
    ]

    qrmi.target.return_value.value = json.dumps(payload)

    valid_device = MagicMock()

    with patch(
        "qrmi.primitives.pasqal.target.deserialize_device",
        side_effect=[
            DeserializeDeviceError("invalid"),
            valid_device,
        ],
    ):
        result = _parse_available_devices(qrmi)

    assert result == {"valid_device": valid_device}


def test_parse_available_devices_empty_list():
    """Verify that _parse_available_devices returns an empty dictionary when the input is an empty list."""
    qrmi = MagicMock()
    qrmi.target.return_value.value = "[]"

    result = _parse_available_devices(qrmi)

    assert result == {}


def test_get_device_returns_digital_analog_device_for_emulator():
    """Verify that get_device returns DigitalAnalogDevice when the resource ID indicates an emulator."""
    qrmi = MagicMock()
    qrmi.resource_id.return_value = "emu"

    result = get_device(qrmi)

    assert result is DigitalAnalogDevice


def test_get_device_returns_matching_cloud_device():
    """Verify that get_device returns the correct device when the resource ID matches a cloud device."""
    qrmi = MagicMock()
    qrmi.resource_id.return_value = "device_a"

    device_a = MagicMock()
    device_b = MagicMock()

    with patch(
        "qrmi.primitives.pasqal.target._parse_available_devices",
        return_value={
            "device_a": device_a,
            "device_b": device_b,
        },
    ):
        result = get_device(qrmi)

    assert result is device_a


def test_get_device_returns_first_device_for_local_resource():
    """Verify that get_device returns the first device when the resource ID does not match any cloud device."""
    qrmi = MagicMock()
    qrmi.resource_id.return_value = "local"

    first_device = MagicMock()
    second_device = MagicMock()

    with patch(
        "qrmi.primitives.pasqal.target._parse_available_devices",
        return_value={
            "device_a": first_device,
            "device_b": second_device,
        },
    ):
        result = get_device(qrmi)

    assert result is first_device


def test_get_device_raises_stop_iteration_when_no_devices_available():
    """Verify that get_device raises StopIteration when no devices are available."""
    qrmi = MagicMock()
    qrmi.resource_id.return_value = "physical"

    with patch(
        "qrmi.primitives.pasqal.target._parse_available_devices",
        return_value={},
    ):
        with pytest.raises(StopIteration):
            get_device(qrmi)


def test_get_target_not_implemented():
    """Verify that get_target raises NotImplementedError."""
    qrmi = MagicMock()

    with pytest.raises(NotImplementedError):
        get_target(qrmi)
