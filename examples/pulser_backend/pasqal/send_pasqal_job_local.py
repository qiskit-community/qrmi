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
import os
from dotenv import load_dotenv
from pulser import Pulse, Register, Sequence
from pulser.backend.remote import JobParams, RemoteBackend
from pulser.json.abstract_repr.deserializer import (
    deserialize_device,
)
from qrmi.pulser_backend.backend import PulserQRMIConnection
from qrmi.pulser_backend.service import QRMIService

import logging

logging.basicConfig(
    level=logging.DEBUG,  # or INFO
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
)
logger = logging.getLogger()

logger.info(json.dumps(dict(os.environ)))

# Create QRMI
load_dotenv()
try:
    service = QRMIService()
except RuntimeError as e:
    logger.error(f"Something went wrong in the QRMI: {e}")
    exit(1)
resources = service.resources()
if len(resources) == 0:
    logger.error("No quantum resource is available.")
    exit(1)


# Randomly select QR
qrmi = resources[0]

target = qrmi.target()

qrmi_conn = PulserQRMIConnection(qrmi)
# Generate Pulser device
device = deserialize_device(target.value)
reg = Register(
    {
        "q0": (-2.5, -2.5),
        "q1": (2.5, -2.5),
        "q2": (-2.5, 2.5),
        "q3": (2.5, 2.5),
    }
).with_automatic_layout(device)
print(device)

seq = Sequence(reg, device)
seq.declare_channel("rydberg", "rydberg_global")

pulse1 = Pulse.ConstantPulse(100, 2, 2, 0)

seq.add(pulse1, "rydberg")
seq.measure("ground-rydberg")


backend = RemoteBackend(seq, qrmi_conn)
result = backend.run([JobParams(runs=500, variables=[])], wait=True)
print("results", result)
