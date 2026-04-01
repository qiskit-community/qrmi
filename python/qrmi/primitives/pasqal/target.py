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

import json
from typing import Any
import pulser
import pulser.abstract_repr
from pulser import MockDevice
from pulser.devices import Device
from qiskit.transpiler.target import Target

from qrmi import QuantumResource


def _is_missing_device_specs_error(err: RuntimeError) -> bool:
    """Return True when QRMI reports device specs are unavailable."""
    message = str(err)
    start = message.find("{")
    if start < 0:
        return False
    try:
        payload = json.loads(message[start:])
    except json.JSONDecodeError:
        return False
    return payload.get("code") == "CD1202"


def _normalize_target_payload(target: Any) -> str:
    """Return target payload as a JSON string.

    The target may be a QRMI wrapper object with `.value`, a JSON string, or a dict.
    """
    target_value = target.value if hasattr(target, "value") else target

    if isinstance(target_value, str):
        return target_value
    if isinstance(target_value, dict):
        return json.dumps(target_value)
    raise TypeError("Unsupported target payload type. Expected JSON string or dict.")


def get_device(qrmi: QuantumResource) -> Device:
    """Returns Pulser Device

    Args:
        qrmi: Pasqal QRMI resource object

    Returns:
        pulser.devices.Device: Pulser device
    """
    try:
        target = qrmi.target()
    except RuntimeError as err:
        if _is_missing_device_specs_error(err):
            return MockDevice
        raise
    target_payload = _normalize_target_payload(target)
    return pulser.abstract_repr.deserialize_device(target_payload)


def get_target(qrmi: QuantumResource) -> Target:
    """Returns Qiskit Target of Pasqal Device for use with Qiskit Pasqal Provider"""
    raise NotImplementedError
