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

import logging

import json
from json import JSONDecodeError
from pulser import DigitalAnalogDevice
from pulser.devices import Device
from pulser.abstract_repr import deserialize_device
from pulser.exceptions.serialization import DeserializeDeviceError
from qiskit.transpiler.target import Target

from qrmi import QuantumResource

logger = logging.getLogger(__name__)


def _parse_available_devices(qrmi: QuantumResource) -> dict[str, Device]:
    """Fetches the devices available through this connection."""

    devices = {}
    # Serialized data from the Pasqal QRMI matches the one from
    # the /api/v1/devices/public-specs cloud endpoint.
    devices_str = qrmi.target().value
    try:
        data = json.loads(devices_str)
    except JSONDecodeError:
        logger.exception(f"Failed to deserialize device information: {devices_str}")
        return devices
    for specs in data:
        name = specs["device_type"]
        try:
            dev = deserialize_device(specs["specs"])
        except DeserializeDeviceError:
            logger.exception(f"Failed to deserialize device: {name}")
            continue
        devices[name] = dev
    return devices


def get_device(qrmi: QuantumResource) -> Device:
    """Returns Pulser Device

    Args:
        qrmi: Pasqal QRMI resource object

    Returns:
        pulser.devices.Device: Pulser device
    """
    resource_id = qrmi.resource_id()
    if "emu" in resource_id.lower():
        return DigitalAnalogDevice
    devices = _parse_available_devices(qrmi)
    if resource_id in devices:
        # Expected cloud case
        specs = devices[resource_id]
    else:
        # Expected local case
        # Get first device specs
        specs = next(iter(devices.values()))
    return specs


def get_target(qrmi: QuantumResource) -> Target:
    """Returns Qiskit Target of Pasqal Device for use with Qiskit Pasqal Provider"""
    raise NotImplementedError
