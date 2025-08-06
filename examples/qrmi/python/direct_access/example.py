# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""An example of IBM Direct Access QRMI python-bindings"""

import os
import time
import json
import argparse
from dotenv import load_dotenv
from qrmi import QuantumResource, ResourceType, Payload, TaskStatus

parser = argparse.ArgumentParser(description="An example of IBM Direct Access QRMI")
parser.add_argument("backend", help="backend name")
parser.add_argument("input", help="primitive input file")
parser.add_argument("program_id", help="'estimator' or 'sampler'")
args = parser.parse_args()

load_dotenv()

qrmi = QuantumResource(args.backend, ResourceType.IBMDirectAccess)
print(qrmi)

print(qrmi.is_accessible())

lock = qrmi.acquire()
print(f"lock {lock}")

target_json = json.loads(qrmi.target().value)
print(json.dumps(target_json, indent=2))
print(qrmi.metadata())

with open(args.input, encoding="utf-8") as f:
    primitive_input = f.read()
    payload = Payload.QiskitPrimitive(input=primitive_input, program_id=args.program_id)
    job_id = qrmi.task_start(payload)
    print(f"Task started {job_id}")

    while True:
        status = qrmi.task_status(job_id)
        if status not in [TaskStatus.Running, TaskStatus.Queued]:
            break

        time.sleep(1)

    print(f"Task ended - {qrmi.task_status(job_id)}")
    print(qrmi.task_result(job_id).value)

    qrmi.task_stop(job_id)

qrmi.release(lock)
