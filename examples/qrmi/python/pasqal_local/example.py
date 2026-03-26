# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2025 IBM, Pasqal. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""An example of Pasqal Local QRMI python-bindings"""

import argparse
import os
import time

from qrmi import Payload, QuantumResource, ResourceType, TaskStatus

parser = argparse.ArgumentParser(description="An example of Pasqal Local QRMI")
parser.add_argument("--backend", required=True, help="Backend name (device identifier)")
parser.add_argument("input", help="Pulser sequence")
args = parser.parse_args()

with open(args.input, encoding="utf-8") as f:
    serialized_sequence = f.read()

# instantiate a QRMI
qrmi = QuantumResource(args.backend, ResourceType.PasqalLocal)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

# Check if QR it's accessible
is_avail = qrmi.is_accessible()
print("Pasqal Local QR is %s accessible" % "not" if not is_avail else "")

# Get a session
session = qrmi.acquire()
os.environ[f"{args.backend}_QRMI_JOB_ACQUISITION_TOKEN"] = session
print("Pasqal Local session ID:", session)

# Get target
target = qrmi.target()
print("QR Target %s" % target.value)

# nit:start_task would be nicer probably
task_id = qrmi.task_start(
    Payload.PasqalCloud(sequence=serialized_sequence, job_runs=1000)
)
print("Task ID: %s" % task_id)

# Get its status
print("Status after creation %s" % qrmi.task_status(task_id))

# Quickly stop it
qrmi.task_stop(task_id)

# Get status, it should be stopped
print("Status after cancelation %s" % qrmi.task_status(task_id))

# Send send another task
new_task_id = qrmi.task_start(
    Payload.PasqalCloud(sequence=serialized_sequence, job_runs=100)
)
print("New Task ID: %s" % new_task_id)

# Wait for completion
while True:
    status = qrmi.task_status(new_task_id)
    if status == TaskStatus.Completed:
        print("Task completed")
        time.sleep(0.5)
        break
    elif status == TaskStatus.Failed:
        print("Task failed")
        break
    else:
        print("Task status %s, waiting 1s" % status)
        time.sleep(1)

# Get the results
print("Results: %s" % qrmi.task_result(new_task_id).value)
