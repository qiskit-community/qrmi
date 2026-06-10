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

import pulser
from dotenv import load_dotenv
from pulser import Pulse, Sequence
from pulser.backend.remote import JobParams
from pulser.register import Register

from qrmi.pulser.connection import PulserQRMIConnection
from qrmi.pulser.service import QRMIService

# Create QRMI
load_dotenv()
service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    raise RuntimeError("No quantum resource is available.")

# Select QR
for res in resources:
    print(f"Available resource: id={res.resource_id()} type={str(res.resource_type())}")

# For this example, we select the first resource
qrmi = resources[0]

qrmi_conn = PulserQRMIConnection(qrmi)

# Generate Pulser device.
# Emulator targets may not expose device specs so we fall back to DigitalAnalogDevice.
# For a real program, you may want to manually fetch the device specs and construct the corresponding Pulser device.
# This can, for example, be done by using the PasqalCloud package.
try:
    avail_devices = qrmi_conn.fetch_available_devices()
    for dev in avail_devices.keys():
        print("Found device specs for: ", dev)
    name, device = next(iter(avail_devices.items()))
    print(f"Chose device specs: '{name}' from the QRMI connection.")
except RuntimeError:
    device = pulser.DigitalAnalogDevice
    print(
        "Could not find device from the QRMI connection. Defaulted to 'DigitalAnaloDevice'"
    )

reg = Register(
    {
        "q0": (-2.5, -2.5),
        "q1": (2.5, -2.5),
        "q2": (-2.5, 2.5),
        "q3": (2.5, 2.5),
    }
).with_automatic_layout(device)

seq = Sequence(reg, device)
seq.declare_channel("rydberg", "rydberg_global")

pulse1 = Pulse.ConstantPulse(100, 2, 2, 0)

seq.add(pulse1, "rydberg")
seq.measure("ground-rydberg")

backend = pulser.QPUBackend(seq, qrmi_conn)
remote_results = backend.run([JobParams(runs=500, variables=[])], wait=True)

print(f"Logs: ", qrmi_conn.get_batch_logs(batch_id=remote_results.batch_id))
print(f"Results: {remote_results.results}")