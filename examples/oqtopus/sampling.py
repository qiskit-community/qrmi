#
# (C) Copyright 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""An example of OQTOPUS QRMI python-bindings"""

import time
import json
from qiskit import QuantumCircuit
from qiskit import qasm3
from qrmi import Payload, TaskStatus, QRMIService # pylint: disable=no-name-in-module

service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    raise ValueError("No quantum resource is available.")

qrmi = resources[0]
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

if qrmi.is_accessible() is False:
    raise RuntimeError("Quantum resource is not accessible")

target_json = json.loads(qrmi.target().value)
print(json.dumps(target_json, indent=2))

# Create circuit: 2 qubits, 2 classical bits
qc = QuantumCircuit(2, 2)

# Apply Hadamard gate to qubit 0
qc.h(0)

# Apply CNOT gate (control=0, target=1)
qc.cx(0, 1)

qc.measure_all()

qasm_string = qasm3.dumps(qc)
print(qasm_string)
payload = Payload.Oqtopus(
    name="Bell State Sampling",
    description="Bell state sampling example",
    job_type="sampling",
    program=qasm_string,
    shots=1000,
    transpiler_info=None,
    simulator_info=None,
    mitigation_info=None,
)
job_id = qrmi.task_start(payload)
print(f"Task started {job_id}")

while True:
    status = qrmi.task_status(job_id)
    if status not in [TaskStatus.Running, TaskStatus.Queued]:
        break

    time.sleep(1)

print(f"Task ended - {qrmi.task_status(job_id)}")
result_json = json.loads(qrmi.task_result(job_id).value)
print(json.dumps(result_json, indent=2))
