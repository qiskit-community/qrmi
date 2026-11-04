# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright IBM, Qoro 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Estimator-style example with Maestro Local QRMI: build a circuit and
observable with Qiskit, convert the circuit to QASM, and compute expectation
values locally."""

# pylint: disable=invalid-name
import json
import os
import time

from qiskit import QuantumCircuit, qasm2
from qiskit.quantum_info import SparsePauliOp
from qrmi import (
    Payload,
    QuantumResource,
    ResourceType,
    TaskStatus,
    UnsupportedFunctionError,
)

BACKEND_NAME = "MAESTRO_LOCAL"
QUBITS = 2

# Construct a resource directly for this standalone example. Slurm jobs can
# also discover Maestro through QRMIService using the QRMI_JOB_QPU_* settings.
qrmi = QuantumResource(BACKEND_NAME, ResourceType.MaestroLocal)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

is_avail = qrmi.is_accessible()
print(f"Maestro Local QR is {'' if is_avail else 'not '}accessible")
if not is_avail:
    raise RuntimeError("Maestro Local QR is not accessible")

# acquire() retains the session in this resource. Export the token only when
# sharing the session with another process or resource instance.
session = qrmi.acquire()
os.environ[f"{BACKEND_NAME}_QRMI_JOB_ACQUISITION_TOKEN"] = session
print("Maestro Local session ID:", session)

######################################################
#                Create Quantum Program              #
######################################################

# Build a Bell-state circuit with Qiskit. No measurements are added, since
# expectation values are computed directly from the resulting state.
circuit = QuantumCircuit(QUBITS)
circuit.h(0)
circuit.cx(0, 1)
print(circuit.draw(output="text"))

# Maestro Local expects the program as a QASM string.
qasm_input = qasm2.dumps(circuit)

# Observable(s) to estimate, as Pauli strings separated by ";".
observable = SparsePauliOp(["ZZ", "XX"])
print(f">>> Observable: {observable.paulis}")
observables = ";".join(str(pauli) for pauli in observable.paulis)

task_id = qrmi.task_start(
    Payload.MaestroLocal(
        input=qasm_input,
        job_type="estimate",
        qubits=QUBITS,
        simulator_type=0,  # aer
        simulation_method=0,  # statevector
        observables=observables,
        config="{}",
    )
)
print("Task ID:", task_id)

# Wait for completion
while True:
    status = qrmi.task_status(task_id)
    if status == TaskStatus.Completed:
        result = json.loads(qrmi.task_result(task_id).value)
        print("Expectation values:", result.get("expectation_values"))
        break
    if status in (TaskStatus.Failed, TaskStatus.Cancelled):
        print(f"Task ended with status {status}")
        try:
            print(qrmi.task_logs(task_id))
        except UnsupportedFunctionError:
            print("Task logs are unavailable for Maestro Local")
        break
    print(f"Task status {status}, waiting 1s")
    time.sleep(1)

qrmi.task_stop(task_id)
qrmi.release(session)
