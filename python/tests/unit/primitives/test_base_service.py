"""Tests for QRMI Service for IBM QRMI integration."""

from qrmi import QRMIService as CoreQRMIService
from qrmi.primitives.service import QRMIService


def test_service_reexports_core_service():
    """Verify"""
    assert QRMIService is CoreQRMIService
