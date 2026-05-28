"""A tool to generate input for Alice & Bob example"""
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

from qsharp import TargetProfile
from qsharp.interop.qiskit import QSharpBackend
from qiskit import QuantumCircuit

# Define Circuit
circuit = QuantumCircuit(1, 1)
circuit.delay(100, unit="us") # Seems to not be taken into account by the QIR conversion
circuit.measure(0, 0)

backend = QSharpBackend()
print(backend.qir(circuit, target_profile=TargetProfile.Adaptive_RI))
