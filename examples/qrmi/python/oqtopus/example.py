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

"""An example of OQTOPUS QRMI python-bindings"""

import time
import json
import argparse
from dotenv import load_dotenv
from qrmi import QuantumResource, ResourceType, Payload, TaskStatus

parser = argparse.ArgumentParser(description="An example of IBM Quantum System QRMI")
parser.add_argument("device_id", help="OQTOPUS device ID")
args = parser.parse_args()

load_dotenv()

qrmi = QuantumResource(args.device_id, ResourceType.OQTOPUS)
print(qrmi)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

print(qrmi.is_accessible())
