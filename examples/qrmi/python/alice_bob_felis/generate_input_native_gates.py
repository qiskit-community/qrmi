# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2026 Alice and Bob. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

import argparse
from dotenv import load_dotenv

from helpers import FelisQIRTranspiler
from qiskit import QuantumCircuit

load_dotenv()

parser = argparse.ArgumentParser(
    description="An example of a Quantum Resource from Alice and Bob's Felis API"
)
parser.add_argument("target", help="Felis target e.g. 'EMU:1Q:LESCANNE_2020'")

args = parser.parse_args()

# Define Circuit
circuit = QuantumCircuit(1, 1)
circuit.initialize('+')
circuit.measure_x(0, 0)

transpiler = FelisQIRTranspiler(args.target)
qir_circuit = transpiler.transpile(circuit)
print(qir_circuit)