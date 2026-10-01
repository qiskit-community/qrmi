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

parser = argparse.ArgumentParser(description="An example of OQTOPUS QRMI")
parser.add_argument("device_id", help="OQTOPUS device ID")
parser.add_argument("job_spec", help="Job spec")
parser.add_argument("job_type", help="Job type")
args = parser.parse_args()

load_dotenv()

qrmi = QuantumResource(args.device_id, ResourceType.OQTOPUS)
print(qrmi)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

print(json.dumps(qrmi.status().to_dict(), indent=2))

target_json = json.loads(qrmi.target().value)
print(json.dumps(target_json, indent=2))

with open(args.job_spec, encoding="utf-8") as f:
    job_spec = json.load(f)
    job_spec = json.dumps(job_spec, indent=2)
    payload = Payload.Oqtopus(job_spec=job_spec)
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
