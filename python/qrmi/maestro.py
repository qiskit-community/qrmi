# (C) Copyright Qoro Quantum 2026
# Licensed under the Apache License, Version 2.0.
"""Client-side helpers for Maestro's native request API.

These helpers serialize data only. Maestro servers and workers do not run Python.
"""

import json

_LEGACY_FIELDS = {
    "input",
    "job_type",
    "qubits",
    "simulator_type",
    "simulation_method",
    "observables",
    "config",
}
_FOREIGN_FIELDS = {
    "job_spec",
    "program_id",
    "parameters",
    "job_runs",
    "sequence",
    "human_qir",
    "input_params",
    "iqmjson",
    "use_timeslot",
    "tag",
}
_NATIVE_ENVELOPE_FIELDS = _LEGACY_FIELDS - {"observables"}


def _native_document(request):
    if isinstance(request, str):
        request = json.loads(request)
    if not isinstance(request, dict) or request.get("schema_version") != 2:
        raise ValueError("Maestro request must be an object with schema_version=2")
    if request.keys() & (_FOREIGN_FIELDS | _NATIVE_ENVELOPE_FIELDS | {"request"}):
        raise ValueError("Native Maestro request cannot mix payload envelopes")
    return request


def request_payload(request):
    """Create a Maestro payload without depending on build-specific enum IDs."""
    from qrmi import Payload

    request = _native_document(request)
    return Payload.MaestroLocal(
        input=json.dumps(request, allow_nan=False),
        job_type="request",
        qubits=0,
        simulator_type=0,
        simulation_method=0,
        observables="",
        config="{}",
    )


def payload_from_input(task_input):
    """Accept a native request, a request wrapper, or a legacy runner payload."""
    from qrmi import Payload

    if not isinstance(task_input, dict):
        raise ValueError("Maestro payload must be an object")
    if task_input.keys() & _FOREIGN_FIELDS:
        raise ValueError("Maestro payload cannot contain another backend's fields")
    if "request" in task_input:
        if set(task_input) != {"request"}:
            raise ValueError("Wrapped Maestro requests accept only the request field")
        return request_payload(task_input["request"])
    if "schema_version" in task_input:
        return request_payload(task_input)
    if task_input.keys() - _LEGACY_FIELDS:
        raise ValueError("Unknown Maestro payload field")
    if not isinstance(task_input.get("job_type"), str):
        raise ValueError("Maestro job_type must be a string")
    kind = task_input["job_type"].lower()
    if kind == "request":
        if "input" not in task_input:
            raise ValueError("Missing Maestro request field: input")
        return request_payload(task_input["input"])
    if kind not in {"execute", "estimate"}:
        raise ValueError("Unknown Maestro job_type")
    required = ("input", "qubits", "simulator_type", "simulation_method")
    missing = [field for field in required if field not in task_input]
    if missing:
        raise ValueError("Missing legacy Maestro fields: " + ", ".join(missing))
    if not isinstance(task_input["input"], str):
        raise ValueError("Legacy Maestro jobs require string input")
    for field in ("qubits", "simulator_type", "simulation_method"):
        value = task_input[field]
        minimum = 1 if field == "qubits" else 0
        if (
            not isinstance(value, int)
            or isinstance(value, bool)
            or not minimum <= value <= 4294967295
        ):
            raise ValueError(
                f"Legacy Maestro {field} must be an integer in {minimum}..4294967295"
            )
    config = task_input.get("config")
    if config is None:
        config = {}
    if not isinstance(config, (dict, str)):
        raise ValueError("Maestro config must be an object or serialized object")
    return Payload.MaestroLocal(
        input=task_input["input"],
        job_type=task_input["job_type"],
        qubits=task_input["qubits"],
        simulator_type=task_input["simulator_type"],
        simulation_method=task_input["simulation_method"],
        observables=task_input.get("observables", ""),
        config=(
            config if isinstance(config, str) else json.dumps(config, allow_nan=False)
        ),
    )
