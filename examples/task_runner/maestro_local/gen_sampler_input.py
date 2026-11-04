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

"""Generates a QRMI task runner input file for an `execute` job on Maestro Local,
built from a Qiskit circuit."""

# pylint: disable=invalid-name
import argparse
import json

from qiskit import QuantumCircuit, qasm2

parser = argparse.ArgumentParser(
    description="A tool to generate a Maestro Local task runner input for testing"
)
parser.add_argument("--qubits", type=int, default=3, help="Number of qubits")
parser.add_argument("--shots", type=int, default=1000, help="Number of shots")
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
    "-o", "--output", default="sampler_input.json", help="Output filename"
)
args = parser.parse_args()

# Build a GHZ-state circuit with Qiskit.
circuit = QuantumCircuit(args.qubits, args.qubits)
circuit.h(0)
for qubit in range(1, args.qubits):
    circuit.cx(0, qubit)
circuit.measure(range(args.qubits), range(args.qubits))
print(circuit.draw(output="text"))

# Maestro Local expects the program as a QASM string.
qasm_input = qasm2.dumps(circuit)

task_input = {
    "input": qasm_input,
    "job_type": "execute",
    "qubits": args.qubits,
    "simulator_type": args.simulator_type,
    "simulation_method": args.simulation_method,
    "config": json.dumps({"shots": args.shots}),
}

with open(args.output, "w", encoding="utf-8") as output_file:
    json.dump(task_input, output_file, indent=2)

print(f"Wrote {args.output}")
