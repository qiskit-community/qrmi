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

""" "generating OQTOPUS program input from Qiskit QuantumCircuit"""

import argparse
from qiskit import QuantumCircuit
from qiskit import qasm3

parser = argparse.ArgumentParser(
    description="A tool to generate OQTOPUS program input from Bell-state QuantumCircuit"
)
parser.add_argument("output", help="output file")
args = parser.parse_args()

# Create circuit: 2 qubits, 2 classical bits
qc = QuantumCircuit(2, 2)

# Apply Hadamard gate to qubit 0
qc.h(0)

# Apply CNOT gate (control=0, target=1)
qc.cx(0, 1)

qc.measure_all()

qasm_string = qasm3.dumps(qc)
with open(args.output, mode="w", encoding="utf-8") as f:
    f.write(qasm_string)
