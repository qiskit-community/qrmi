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

from dotenv import load_dotenv
from pulser import Pulse, Register, Sequence
from pulser.backend.remote import JobParams
from qrmi.pulser_backend.backend import PulserQRMIBackend, PulserQRMIConnection
from qrmi.pulser_backend.service import QRMIService
from target import get_device

# Create QRMI
load_dotenv()
service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    print("No quantum resource is available.")

# Randomly select QR
qrmi = resources[0]

qrmi_conn = PulserQRMIConnection(qrmi)

# Generate Pulser device
device = get_device(qrmi)

reg = Register(
    {
        "q0": (-2.5, -2.5),
        "q1": (2.5, -2.5),
        "q2": (-2.5, 2.5),
        "q3": (2.5, 2.5),
    }
)

seq = Sequence(reg, device)
seq.declare_channel("rydberg", "rydberg_global")

pulse1 = Pulse.ConstantPulse(100, 2, 2, 0)

seq.add(pulse1, "rydberg")
seq.measure("ground-rydberg")

backend = PulserQRMIBackend(seq, qrmi_conn)
result = backend.run([JobParams(runs=1000, variables=[])], wait=True)
print(f"Results: {json.loads(result[0])['counter']}")
