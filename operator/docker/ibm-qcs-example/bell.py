# This code is part of Qiskit.
# (C) Copyright IBM 2026.
# Licensed under the Apache License, Version 2.0.
# See LICENSE.txt in the repository root.

"""Run a Bell circuit using an operator-managed IBM acquisition."""

import json
import os

from qiskit import QuantumCircuit
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qrmi import QuantumResource, ResourceType
from qrmi.primitives.ibm import SamplerV2, get_target


def main():
    device = os.environ["QRMI_JOB_QPU_RESOURCES"]
    token = os.environ[f"{device}_QRMI_JOB_ACQUISITION_TOKEN"]
    if not token:
        raise RuntimeError("Operator acquisition token is empty")

    resource = QuantumResource(device, ResourceType.IBMQiskitRuntimeService)
    if resource.metadata().get("session_id") != token:
        raise RuntimeError("Resource did not reuse the operator acquisition")

    circuit = QuantumCircuit(2)
    circuit.h(0)
    circuit.cx(0, 1)
    circuit.measure_all()

    target = get_target(resource)
    pm = generate_preset_pass_manager(optimization_level=1, target=target)
    isa_circuit = pm.run(circuit)

    sampler = SamplerV2(resource, options={"default_shots": 128})
    job = sampler.run([isa_circuit])
    print(f"Quantum job ID: {job.job_id()}", flush=True)

    result = job.result()
    counts = result[0].data.meas.get_counts()
    print(json.dumps({"backend": device, "counts": counts}), flush=True)
    if sum(counts.values()) != 128:
        raise RuntimeError("Unexpected measurement count")


if __name__ == "__main__":
    main()
