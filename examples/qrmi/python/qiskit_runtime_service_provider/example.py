# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# Copyright (C) 2025 UKRI-STFC (Hartree Centre), IBM
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""An example of IBM Qiskit Runtime Service QRMI python-bindings"""

import os
import time
import json
import argparse
from dotenv import load_dotenv
from qrmi import IBMQiskitRuntimeServiceProvider

parser = argparse.ArgumentParser(
    description="An example of IBM Qiskit Runtime Service QRMI"
)
args = parser.parse_args()

load_dotenv()


provider = IBMQiskitRuntimeServiceProvider()
resources = provider.backends("num_qubits=27&name=ibm*")
for qrmi in resources:
    print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")
    print(qrmi.is_accessible())
