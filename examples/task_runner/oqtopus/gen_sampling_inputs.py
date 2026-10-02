#
# (C) Copyright 2026 IBM. All Rights Reserved.
# (C) Copyright 2026 The University of Osaka. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

""" "generating OQTOPUS program input from Qiskit QuantumCircuit"""

import json
import argparse
from qiskit import QuantumCircuit
from qiskit import qasm3
import dataclasses
from oqtopus_client.services.job_spec import OqtopusJobSpec

parser = argparse.ArgumentParser(
    description="A tool to generate OQTOPUS program input from Bell-state QuantumCircuit"
)
parser.add_argument("device_id", help="device ID")
args = parser.parse_args()

# Create circuit: 2 qubits, 2 classical bits
qc = QuantumCircuit(2, 2)

# Apply Hadamard gate to qubit 0
qc.h(0)

# Apply CNOT gate (control=0, target=1)
qc.cx(0, 1)

qc.measure_all()

qasm_str = qasm3.dumps(qc)

spec = OqtopusJobSpec(
    device_id=args.device_id,
    job_type="sampling",
    program=[qasm_str],
    name="Bell State Sampling",
    description="Bell state sampling example",
)

d = dataclasses.asdict(spec)
# device_id will be added by QRMI
d.pop("device_id", None)

with open(f"sampling_input_{args.device_id}.json", mode="w", encoding="utf-8") as f:
    json.dump(d, f, indent=2)
