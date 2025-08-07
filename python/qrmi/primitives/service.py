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

"""QRMI Service"""

import os
from logging import getLogger
from typing import List

from qrmi import QuantumResource, ResourceType

logger = getLogger("qrmi")


class QRMIService:
    """Class for interacting with the QRMI resources"""

    def __init__(self):
        qpus = os.environ["SLURM_JOB_QPU_RESOURCES"]
        logger.debug("qpus: %s", qpus)
        if len(qpus) == 0:
            qpus = []
        else:
            qpus = qpus.split(",")

        qpu_types = os.environ["SLURM_JOB_QPU_TYPES"]
        logger.debug("qpu types: %s", qpu_types)
        if len(qpu_types) == 0:
            qpu_types = []
        else:
            qpu_types = qpu_types.split(",")

        if len(qpus) != len(qpu_types):
            raise ValueError("Inconsistent specifications of QPU resources and types")

        self._qrmi_resources = {}
        for i, qpu in enumerate(qpus):
            qpu = qpu.strip()
            qrmi = None
            if qpu_types[i] == "direct-access":
                qrmi = QuantumResource(qpu, ResourceType.IBMDirectAccess)
            elif qpu_types[i] == "qiskit-runtime-service":
                qrmi = QuantumResource(qpu, ResourceType.IBMQiskitRuntimeService)
            elif qpu_types[i] == "pasqal-cloud":
                qrmi = QuantumResource(qpu, ResourceType.PasqalCloud)
            else:
                logger.warning(
                    "Unsupported resource type: %s specified for %s", qpu_types[i], qpu
                )

            if qrmi.is_accessible() is True:
                self._qrmi_resources[qpu] = qrmi
            else:
                logger.debug("%s is not accessible now. ignored.", qpu)

    def resources(self) -> List[QuantumResource]:
        """Return all accessible QRMI resources.

        Returns:
            List[QuantumResource]: QRMI resources
        """
        return list(self._qrmi_resources.values())

    def resource(self, resource_id: str) -> QuantumResource:
        """Return a single backend matching the specified resource identifier.

        Args:
            resource_id: A resource identifier, i.e. backend name for IBM Quantum.

        Returns:
            QuantumResource: QRMI resource if found, otherwise None.
        """
        return self._qrmi_resources.get(resource_id)
