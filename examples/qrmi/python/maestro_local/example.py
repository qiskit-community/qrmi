# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright IBM, Qoro 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""An example of Maestro Local QRMI python-bindings"""

import argparse
import os
import time

from qrmi import (
    Payload,
    QuantumResource,
    ResourceType,
    TaskStatus,
    UnsupportedFunctionError,
)

parser = argparse.ArgumentParser(description="An example of Maestro Local QRMI")
parser.add_argument("--backend", required=True, help="Backend name (device identifier)")
parser.add_argument("input", help="QASM input file")
parser.add_argument(
    "--job-type", default="execute", help="Job type ('execute' or 'estimate')"
)
parser.add_argument("--qubits", type=int, default=5, help="Number of qubits")
parser.add_argument(
    "--simulator-type", type=int, default=0, help="Simulator type (0 = aer, 1 = qcsim)"
)
parser.add_argument(
    "--simulation-method",
    type=int,
    default=0,
    help="Simulation method (0 = statevector, 1 = matrix_product_state)",
)
parser.add_argument(
    "--observables",
    default="",
    help='Observables (pauli strings separated by ";"), required for "estimate" job type',
)
parser.add_argument("--config", default="{}", help="Task configuration, in JSON format")
args = parser.parse_args()

with open(args.input, encoding="utf-8") as f:
    qasm_input = f.read()

# instantiate a QRMI
qrmi = QuantumResource(args.backend, ResourceType.MaestroLocal)
print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")

# Check if QR is accessible
is_avail = qrmi.is_accessible()
print(f"Maestro Local QR is {'' if is_avail else 'not '}accessible")

# Get a session
session = qrmi.acquire()
os.environ[f"{args.backend}_QRMI_JOB_ACQUISITION_TOKEN"] = session
print("Maestro Local session ID:", session)

print(qrmi.metadata())

# Get target
try:
    target = qrmi.target()
    print(f"QR Target {target.value}")
except UnsupportedFunctionError:
    print("Target information is unavailable for Maestro Local")

task_id = qrmi.task_start(
    Payload.MaestroLocal(
        input=qasm_input,
        job_type=args.job_type,
        qubits=args.qubits,
        simulator_type=args.simulator_type,
        simulation_method=args.simulation_method,
        observables=args.observables,
        config=args.config,
    )
)
print(f"Task ID: {task_id}")

# Wait for completion
while True:
    status = qrmi.task_status(task_id)
    if status == TaskStatus.Completed:
        print("Task completed")
        print(f"Results: {qrmi.task_result(task_id).value}")
        break
    if status in (TaskStatus.Failed, TaskStatus.Cancelled):
        print(f"Task ended with status {status}")
        try:
            print(qrmi.task_logs(task_id))
        except UnsupportedFunctionError:
            print("Task logs are unavailable for Maestro Local")
        break
    print(f"Task status {status}, waiting 1s")
    time.sleep(1)

qrmi.task_stop(task_id)

qrmi.release(session)
