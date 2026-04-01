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

import pulser
import pulser.abstract_repr
from pulser import MockDevice
from pulser.devices import Device
from qiskit.transpiler.target import Target

from qrmi import QuantumResource


def get_device(qrmi: QuantumResource) -> Device:
    """Returns Pulser Device

    Args:
        qrmi: Pasqal Cloud QRMI object

    Returns:
        pulser.devices.Device: Pulser device
    """
    try:
        target = qrmi.target()
    except RuntimeError as err:
        if "Device specs are not available for emulators." in str(err):
            return MockDevice
        raise
    target_value = target.value if hasattr(target, "value") else target
    return pulser.abstract_repr.deserialize_device(target_value)


def get_target(qrmi: QuantumResource) -> Target:
    """Returns Qiskit Target of Pasqal Device for use with Qiskit Pasqal Provider"""
    raise NotImplementedError
