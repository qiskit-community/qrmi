# This code is part of Qiskit.
#
# Copyright (C) IBM 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""QRMIService example.

Unlike the other examples in ../, which each construct a single, specific
resource directly (e.g. QuantumResource("ibm_torino",
ResourceType.IBMQuantumSystem)), this example uses QRMIService, which
discovers *all* of the QPU resources assigned to the current job from the
environment -- the same environment variables a Slurm QRMI plugin would set
-- and exposes the ones that are currently accessible.
"""

import argparse
from dotenv import load_dotenv
from qrmi import QRMIService

parser = argparse.ArgumentParser(description="QRMIService Example")
parser.add_argument(
    "resource",
    nargs="?",
    default=None,
    help="Resource identifier to acquire, e.g. a backend name. If omitted, "
    "this just lists every accessible resource assigned to the job.",
)
args = parser.parse_args()

load_dotenv()

# Discovers and filters the job's QPU resources. See this example's README
# for the environment variables this reads.
service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    print("No accessible resources found for this job.")
    raise SystemExit(0)

print(f"Accessible resources ({len(resources)} found):")
for res in resources:
    print(f"  {res.resource_id():<30} type={str(res.resource_type())}")

if args.resource is None:
    raise SystemExit(0)

resource = service.resource(args.resource)
if resource is None:
    raise SystemExit(
        f"'{args.resource}' was not found among this job's accessible resources"
    )

print(f"\nAcquiring '{args.resource}'...")
lock = resource.acquire()
print(f"acquisition token = {lock}")

print(resource.metadata())
print(resource.target().value)

resource.release(lock)
print(f"Released '{args.resource}'.")
