# This code is part of Qiskit.
#
# (C) Copyright 2025, 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Tests for QuantumResource.resource_id() and resource_type().

All tests are self-contained: they do not connect to any remote API.
Where a real QuantumResource is constructed the required environment
variables are injected via ``monkeypatch`` so no credentials are needed.
"""

import base64

import pytest

from qrmi import QuantumResource, ResourceType

# ---------------------------------------------------------------------------
# QuantumResource — IBMQuantumSystem
# ---------------------------------------------------------------------------


@pytest.fixture(name="ibm_qs_env")
def fixture_ibm_qs_env(monkeypatch):
    """Inject the minimum env vars required to construct an IBMQuantumSystem resource."""
    rid = "test_eagle"
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QS_ENDPOINT", "https://localhost")
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QS_IAM_APIKEY", "dummy-key")
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QS_SERVICE_CRN", "dummy-crn")
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QS_IAM_ENDPOINT", "https://iam.localhost")
    return rid


def test_ibm_qs_resource_id(ibm_qs_env):
    """resource_id() returns the name passed to the constructor."""
    rid = ibm_qs_env
    qr = QuantumResource(rid, ResourceType.IBMQuantumSystem)
    assert qr.resource_id() == rid


def test_ibm_qs_resource_type(ibm_qs_env):
    """resource_type() returns ResourceType.IBMQuantumSystem."""
    qr = QuantumResource(ibm_qs_env, ResourceType.IBMQuantumSystem)
    assert qr.resource_type() == ResourceType.IBMQuantumSystem


# ---------------------------------------------------------------------------
# QuantumResource — IBMQiskitRuntimeService
# ---------------------------------------------------------------------------


@pytest.fixture(name="ibm_qrs_env")
def fixture_ibm_qrs_env(monkeypatch):
    """Inject the minimum env vars required to construct an IBMQiskitRuntimeService resource."""
    rid = "ibm_marrakesh"
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QRS_ENDPOINT", "https://localhost")
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QRS_IAM_ENDPOINT", "https://iam.localhost")
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QRS_IAM_APIKEY", "dummy-key")
    monkeypatch.setenv(f"{rid}_QRMI_IBM_QRS_SERVICE_CRN", "dummy-crn")
    return rid


def test_ibm_qrs_resource_id(ibm_qrs_env):
    """resource_id() returns the name passed to the constructor."""
    rid = ibm_qrs_env
    qr = QuantumResource(rid, ResourceType.IBMQiskitRuntimeService)
    assert qr.resource_id() == rid


def test_ibm_qrs_resource_type(ibm_qrs_env):
    """resource_type() returns ResourceType.IBMQiskitRuntimeService."""
    qr = QuantumResource(ibm_qrs_env, ResourceType.IBMQiskitRuntimeService)
    assert qr.resource_type() == ResourceType.IBMQiskitRuntimeService


# ---------------------------------------------------------------------------
# QuantumResource — PasqalCloud
# ---------------------------------------------------------------------------


@pytest.fixture(name="pasqal_cloud_env")
def fixture_pasqal_cloud_env(monkeypatch):
    """Inject the minimum env vars required to construct a PasqalCloud resource."""
    rid = "fresnel"
    monkeypatch.setenv(f"{rid}_QRMI_PASQAL_PROJECT_ID", "test-project-id")
    monkeypatch.setenv(f"{rid}_QRMI_PASQAL_USERNAME", "test@example.com")
    monkeypatch.setenv(f"{rid}_QRMI_PASQAL_PASSWORD", "dummy-pass")
    return rid


def test_pasqal_cloud_resource_id(pasqal_cloud_env):
    """resource_id() returns the name passed to the constructor."""
    rid = pasqal_cloud_env
    qr = QuantumResource(rid, ResourceType.PasqalCloud)
    assert qr.resource_id() == rid


def test_pasqal_cloud_resource_type(pasqal_cloud_env):
    """resource_type() returns ResourceType.PasqalCloud."""
    qr = QuantumResource(pasqal_cloud_env, ResourceType.PasqalCloud)
    assert qr.resource_type() == ResourceType.PasqalCloud


# ---------------------------------------------------------------------------
# QuantumResource — AliceBobFelis
# ---------------------------------------------------------------------------


@pytest.fixture(name="alice_bob_env")
def fixture_alice_bob_env(monkeypatch):
    """Inject the minimum env vars required to construct an AliceBobFelis resource.

    The API key must be a valid base64-encoded ``user:password`` string because
    the Rust constructor calls ``decode_api_key`` and panics on invalid input.
    """
    rid = "ab_emu_40q_physical_cats"
    api_key = base64.b64encode(b"user:dummy-password").decode()
    monkeypatch.setenv(f"{rid}_QRMI_AB_FELIS_API_KEY", api_key)
    monkeypatch.setenv(f"{rid}_QRMI_AB_FELIS_BASE_ENDPOINT", "https://localhost")
    return rid


def test_alice_bob_resource_id(alice_bob_env):
    """resource_id() returns the name passed to the constructor."""
    rid = alice_bob_env
    qr = QuantumResource(rid, ResourceType.AliceBobFelis)
    assert qr.resource_id() == rid


def test_alice_bob_resource_type(alice_bob_env):
    """resource_type() returns ResourceType.AliceBobFelis."""
    qr = QuantumResource(alice_bob_env, ResourceType.AliceBobFelis)
    assert qr.resource_type() == ResourceType.AliceBobFelis


# ---------------------------------------------------------------------------
# QuantumResource — IQMServer
# ---------------------------------------------------------------------------


@pytest.fixture(name="iqm_env")
def fixture_iqm_env(monkeypatch):
    """Inject the minimum env vars required to construct an IQMServer resource.

    The resource ID uses underscore notation (``iqm_device``) because the Rust
    constructor replaces the *last* underscore with a colon when setting the
    internal backend name (``iqm:device``).
    """
    rid = "iqm_device"
    monkeypatch.setenv(f"{rid}_QRMI_IQM_ISA_ENDPOINT", "https://localhost")
    monkeypatch.setenv(f"{rid}_QRMI_IQM_ISA_TOKEN", "dummy-token")
    return rid


def test_iqm_resource_id(iqm_env):
    """resource_id() returns the backend name with the last underscore replaced by a colon."""
    qr = QuantumResource(iqm_env, ResourceType.IQMServer)
    # Rust converts the last '_' in the resource ID to ':' (e.g. "iqm_device" → "iqm:device")
    assert qr.resource_id() == "iqm:device"


def test_iqm_resource_type(iqm_env):
    """resource_type() returns ResourceType.IQMServer."""
    qr = QuantumResource(iqm_env, ResourceType.IQMServer)
    assert qr.resource_type() == ResourceType.IQMServer
