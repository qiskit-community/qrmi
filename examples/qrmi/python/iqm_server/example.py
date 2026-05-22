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

"""An example of IQM Server QRMI python-bindings"""

import time
import json
import argparse
from dotenv import load_dotenv
from qrmi import QuantumResource, ResourceType, Payload, TaskStatus

parser = argparse.ArgumentParser(description="An example of IBM Quantum System QRMI")
parser.add_argument("qc_alias", help="QC alias name")
parser.add_argument("input", help="IQM JSON input file")
parser.add_argument("job_type", help="'circuit','run', or 'sweep'")
args = parser.parse_args()

load_dotenv()

qrmi = QuantumResource(args.qc_alias, ResourceType.IQMServer)
print(qrmi)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

print(qrmi.is_accessible())

lock = qrmi.acquire()
print(f"lock {lock}")

target_json = json.loads(qrmi.target().value)
print(json.dumps(target_json, indent=2))
print(qrmi.metadata())

with open(args.input, encoding="utf-8") as f:
    primitive_input = f.read()
    payload = Payload.IQMServer(
        iqmjson=primitive_input, job_type=args.job_type, use_timeslot=False, tag=None
    )
    job_id = qrmi.task_start(payload)
    print(f"Task started {job_id}")

    while True:
        status = qrmi.task_status(job_id)
        if status not in [TaskStatus.Running, TaskStatus.Queued]:
            break

        time.sleep(1)

    print(f"Task ended - {qrmi.task_status(job_id)}")
    print(qrmi.task_result(job_id).value)

    print(qrmi.task_logs(job_id))

    qrmi.task_stop(job_id)

qrmi.release(lock)
