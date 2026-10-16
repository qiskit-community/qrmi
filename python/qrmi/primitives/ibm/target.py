# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2024-2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Qiskit Target creation"""

# pylint: disable=no-name-in-module
import warnings
from qiskit.transpiler.target import Target
from qrmi import QuantumResource
from .backend import get_backend


def get_target(qrmi: QuantumResource) -> Target:
    """Returns Qiskit transpiler target

    Args:
        qrmi: IBM QRMI object

    Returns:
        qiskit.transpiler.target.Target: Qiskit Transpiler target
    """
    warnings.warn(
        "`get_target` function is deprecated as of qrmi 0.26.3, use get_backend().target instead.",
        DeprecationWarning,
        stacklevel=2,
    )

    backend = get_backend(qrmi)
    return backend.target
