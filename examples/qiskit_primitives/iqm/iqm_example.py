# This code is part of Qiskit.
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

"""This file is an example of using Qiskit on IQM to run a simple but
non-trivial quantum circuit on an IQM quantum
computer. See the Qiskit on IQM user guide for instructions:
https://iqm-finland.github.io/qiskit-on-iqm/user_guide.html
"""

from qiskit import transpile, QuantumCircuit
from qrmi.qiskit_iqm import IQMProvider

provider = IQMProvider()
backend = provider.get_backend()
print(backend)
print(backend.num_qubits)

SHOTS = 1000

# Create Quantum Registers and Quantum Circuit
# https://github.com/iqm-finland/qiskit-on-iqm/blob/main/src/iqm/qiskit_iqm/examples/resonance_example.py
num_qb = min(backend.num_qubits, 5)  # use at most 5 qubits

qc = QuantumCircuit(num_qb)
qc.h(0)
for qb in range(1, num_qb):
    qc.cx(0, qb)
qc.barrier()
qc.measure_all()

qc_transpiled = transpile(qc, backend)
print(qc_transpiled.draw(output='text'))

job = backend.run(qc_transpiled, shots=SHOTS)
counts=job.result().get_counts()
print(counts)
