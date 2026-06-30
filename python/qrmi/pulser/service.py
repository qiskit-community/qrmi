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

from qrmi import QuantumResource, ResourceType, get_job_qpu_resources_and_types

logger = getLogger("qrmi")


class QRMIService:
    """Class for interacting with the QRMI resources"""

    def __init__(self):
        # If resource acquisition failed in QRMI plugin,
        # the plugin may expose the error reason via environment variable.
        plugin_error = os.environ.get("QRMI_PLUGIN_ERROR")
        if plugin_error is not None:
            raise RuntimeError(plugin_error)

        qpus, qpu_types = get_job_qpu_resources_and_types()
        logger.debug("qpus: %s", qpus)
        logger.debug("qpu types: %s", qpu_types)

        self._qrmi_resources = {}
        for i, qpu in enumerate(qpus):
            qpu = qpu.strip()
            resource = None
            if qpu_types[i] == "qiskit-runtime-service":
                resource = QuantumResource(qpu, ResourceType.IBMQiskitRuntimeService)
            elif qpu_types[i] == "pasqal-cloud":
                resource = QuantumResource(qpu, ResourceType.PasqalCloud)
            elif qpu_types[i] == "pasqal-local":
                resource = QuantumResource(qpu, ResourceType.PasqalLocal)
            else:
                logger.warning(
                    "Unsupported resource type: %s specified for %s", qpu_types[i], qpu
                )
                continue

            try:
                if resource.is_accessible():
                    self._qrmi_resources[qpu] = resource
                else:
                    logger.debug("%s is not accessible now. ignored.", qpu)
            except RuntimeError as err:
                raise RuntimeError(f"{qpu} is not accessible. {err}") from err

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
