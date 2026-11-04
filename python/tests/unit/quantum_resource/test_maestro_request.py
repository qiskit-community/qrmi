# (C) Copyright Qoro Quantum 2026
# Licensed under the Apache License, Version 2.0.
"""Test native request serialization and legacy payload compatibility."""

import json

import pytest

from qrmi.maestro import payload_from_input, request_payload


def test_request_payload_preserves_native_fields():
    """Keep noise, distribution, and multiline source intact in each envelope."""
    request = {
        "schema_version": 2,
        "operation": "execute",
        "circuit": {
            "source": "OPENQASM 2.0;\n// line comment\nqreg q[2];",
            "num_qubits": 2,
        },
        "simulator": {
            "backend": "distributed_gpu",
            "distribution": {"devices": [0, 1]},
        },
        "noise": {
            "channels": [{"kind": "bit_flip", "targets": [0], "probability": 0.1}]
        },
    }
    for task_input in (
        request,
        {"request": request},
        {"job_type": "request", "input": request},
    ):
        payload = payload_from_input(task_input)
        assert payload.job_type == "request"
        assert payload.qubits == 0
        assert json.loads(payload.input) == request


def test_legacy_runner_accepts_config_object_or_string():
    """Serialize object configurations without double-encoding JSON strings."""
    task = {
        "input": "OPENQASM 2.0; qreg q[1];",
        "job_type": "execute",
        "qubits": 1,
        "simulator_type": 0,
        "simulation_method": 0,
    }
    for config in ({"shots": 4}, '{"shots":4}'):
        payload = payload_from_input({**task, "config": config})
        assert json.loads(payload.config) == {"shots": 4}
        assert payload.input == task["input"]


@pytest.mark.parametrize(
    "document", [[], {}, {"schema_version": 1}, '{"schema_version":1}']
)
def test_request_payload_rejects_wrong_schema(document):
    """Reject missing and unsupported schema versions before submission."""
    with pytest.raises(ValueError):
        request_payload(document)


def test_request_payload_rejects_nonfinite_options():
    """Reject nonfinite values instead of emitting invalid JSON numbers."""
    with pytest.raises(ValueError):
        request_payload(
            {"schema_version": 2, "simulator": {"options": {"threshold": float("nan")}}}
        )
