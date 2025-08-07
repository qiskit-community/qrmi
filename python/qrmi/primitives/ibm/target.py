# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2024, 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Qiskit Target creation"""

# pylint: disable=invalid-name
import json
from qiskit.transpiler.target import Target
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target
from qiskit_ibm_runtime.models import BackendProperties, BackendConfiguration
from qrmi import QuantumResource


def get_target(qrmi: QuantumResource) -> Target:
    """Returns Qiskit transpiler target

    Args:
        qrmi: IBM QRMI object

    Returns:
        qiskit.transpiler.target.Target: Qiskit Transpiler target
    """
    target = qrmi.target()
    target = json.loads(target.value)
    backend_config = BackendConfiguration.from_dict(target["configuration"])
    backend_props = BackendProperties.from_dict(target["properties"])
    return convert_to_target(backend_config, backend_props)
