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
parser.add_argument("program", help="program input")
parser.add_argument("job_type", help="job type")
args = parser.parse_args()

load_dotenv()

qrmi = QuantumResource(args.device_id, ResourceType.OQTOPUS)
print(qrmi)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

print(qrmi.is_accessible())

target_json = json.loads(qrmi.target().value)
print(json.dumps(target_json, indent=2))

with open(args.program, encoding="utf-8") as f:
    program = f.read()
    payload = Payload.Oqtopus(
        name="Bell State Sampling",
        description="Bell state sampling example",
        job_type=args.job_type,
        program=program,
        shots=1000,
        transpiler_info=None,
        simulator_info=None,
        mitigation_info=None,
    )
    job_id = qrmi.task_start(payload)
    print(f"Task started {job_id}")

    while True:
        status = qrmi.task_status(job_id)
        if status not in [TaskStatus.Running, TaskStatus.Queued]:
            break

        time.sleep(1)

    print(f"Task ended - {qrmi.task_status(job_id)}")
    result_json = json.loads(qrmi.task_result(job_id).value)
    print(json.dumps(result_json, indent=2))
