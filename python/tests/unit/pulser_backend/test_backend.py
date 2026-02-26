"""Minimal tests for pulser backend integration."""

from qrmi.pulser_backend.backend import PulserQRMIConnection


def test_supports_open_batch_is_false() -> None:
    """Return False for open batch support."""
    connection = PulserQRMIConnection(qrmi=object())  # type: ignore[arg-type]
    assert connection.supports_open_batch() is False
