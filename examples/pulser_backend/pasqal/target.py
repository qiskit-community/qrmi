# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2025 Pasqal, IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Pulser Device creation"""

# pylint: disable=invalid-name
import random

import pulser
import pulser.abstract_repr
from dotenv import load_dotenv
from pulser.devices import Device
from qrmi.pulser_backend.service import QRMIService

from qrmi import QuantumResource


def get_device(qrmi: QuantumResource) -> Device:
    """Returns Pulser Device

    Args:
        qrmi: Pasqal Cloud QRMI object

    Returns:
        pulser.devices.Device: Pulser device
    """
    target = qrmi.target()
    return pulser.abstract_repr.deserialize_device(target.value)

if __name__ == "__main__":
    import random

    from dotenv import load_dotenv
    from qrmi_primitives import QRMIService

    # Create QRMI
    load_dotenv()
    service = QRMIService()

    resources = service.resources()
    if len(resources) == 0:
        raise ValueError("No quantum resource is available.")

    # Randomly select QR
    qrmi_connection = resources[random.randrange(len(resources))]
    print(f"Device: {get_device(qrmi_connection)}")
