# This code is part of Qiskit.
#
# (C) Copyright 2025 Pasqal, IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.


import json
import random

from dotenv import load_dotenv
from pulser_qrmi_backend.service import QRMIService
from qiskit.circuit import QuantumCircuit
from qiskit_pasqal_provider.providers.gate import HamiltonianGate, InterpolatePoints
from qrmi.primitives.pasqal.sampler import QPPSamplerV2

# Create QRMI
load_dotenv()
service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    raise ValueError("No quantum resource is available.")

# Randomly select QR
qrmi = resources[random.randrange(len(resources))]

######################################################
#                Create Quantum Program              #
######################################################

# We define the coordinates of the atoms, 6 in total.
coords = [[0, 0], [3, 5.2], [6, 0], [9, -5.2], [9, 5.2], [12, 0]]

# With a blockade radius of 8.7
blockade_radius = 8.7

# Calculate interaction strength between nearest-neighbours
interaction = 5420158.53 / blockade_radius**6

# Set up an adiabatic pulse,
# This pulse ramps from up 0 -> 4, stays constant, and ramps down again during the times
times = [0, 0.2, 0.8, 1]
ampl = InterpolatePoints(values=[0, 4, 4, 0], times=times)
det = InterpolatePoints(
    values=[-10, -10, interaction / 2, interaction / 2],
    times=times,
)
phase = 0.0

# analog gate
gate = HamiltonianGate(ampl, det, phase, coords, grid_transform="triangular")

# Qiskit circuit with analog gate
qc = QuantumCircuit(len(coords))
qc.append(gate, qc.qubits)

sampler = QPPSamplerV2(qrmi=qrmi)
res = sampler.run([qc])
print(json.loads(res[0])['counter'])
