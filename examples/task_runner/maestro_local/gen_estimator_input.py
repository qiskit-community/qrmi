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

"""Generates a QRMI task runner input file for an `estimate` job on Maestro Local,
built from a Qiskit circuit and observable."""

# pylint: disable=invalid-name
import argparse
import json

from qiskit import QuantumCircuit, qasm2
from qiskit.quantum_info import SparsePauliOp

parser = argparse.ArgumentParser(
    description="A tool to generate a Maestro Local task runner input for testing"
)
parser.add_argument(
    "--simulator-type", type=int, default=0, help="Simulator type (0 = aer, 1 = qcsim)"
)
parser.add_argument(
    "--simulation-method",
    type=int,
    default=0,
    help="Simulation method (0 = statevector, 1 = matrix_product_state)",
)
parser.add_argument(
    "-o", "--output", default="estimator_input.json", help="Output filename"
)
args = parser.parse_args()

QUBITS = 2

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

task_input = {
    "input": qasm_input,
    "job_type": "estimate",
    "qubits": QUBITS,
    "simulator_type": args.simulator_type,
    "simulation_method": args.simulation_method,
    "observables": observables,
    "config": "{}",
}

with open(args.output, "w", encoding="utf-8") as output_file:
    json.dump(task_input, output_file, indent=2)

print(f"Wrote {args.output}")
