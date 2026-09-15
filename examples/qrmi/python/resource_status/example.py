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

"""An example of QRMI status()"""

import json
import argparse
from logging import getLogger, basicConfig, DEBUG
from dotenv import load_dotenv
# pylint: disable=no-name-in-module
from qrmi import (
    QuantumResource,
    ResourceType,
    ResourceStatusCode,
)

basicConfig(level=DEBUG, format="{asctime} [{levelname}] {message}", style="{")

logger = getLogger(__name__)

parser = argparse.ArgumentParser(
    description="Display the current status of the specified resource"
)
parser.add_argument("resource_type", help="Resource type")
parser.add_argument("resource_id", help="Resource ID")
args = parser.parse_args()

_resource_type_map = {
    "ibm-quantum-system": ResourceType.IBMQuantumSystem,
    "qiskit-runtime-service": ResourceType.IBMQiskitRuntimeService,
    "ibm-quantum-compute-service": ResourceType.IBMQuantumComputeService,
    "pasqal-cloud": ResourceType.PasqalCloud,
    "pasqal-local": ResourceType.PasqalLocal,
    "alice-bob-felis": ResourceType.AliceBobFelis,
    "iqm-server": ResourceType.IQMServer,
}

load_dotenv()

qrmi = QuantumResource(args.resource_id, _resource_type_map.get(args.resource_type))
logger.info(
    "Selected resource: id=%s type=%s", qrmi.resource_id(), str(qrmi.resource_type())
)

if qrmi.is_accessible():
    logger.info("accessible")
else:
    logger.info("unaccessible")

status = qrmi.status()
match status.status:
    case ResourceStatusCode.Online:
        logger.info("online")
    case ResourceStatusCode.Offline:
        logger.info("offline")
    case ResourceStatusCode.Paused:
        logger.info("paused")
    case ResourceStatusCode.Busy:
        logger.info("busy")

logger.info("reason: %s", status.status_reason)

if status.pending_job_count is None:
    logger.warning("this resource does not report pending_job_count")
else:
    logger.info("pending job count: %d", status.pending_job_count)

if status.healthy is None:
    logger.warning("this resource not report healthy")
else:
    logger.info("%s", "healthy" if status.healthy is True else "unhealthy")

if status.capacity:
    logger.info("available slots: %d", status.capacity.available_slots)
    logger.info("max slots: %d", status.capacity.max_slots)
else:
    logger.warning("this resource not report capacity")

print(json.dumps(status.to_dict(), indent=2))
