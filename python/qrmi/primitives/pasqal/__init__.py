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

"""Pasqal implementations of Primitive."""

from .sampler import QPPSamplerV2, QRMIPasqalBackend, QRMIPasqalJob
from .target import get_device, get_target

SamplerV2 = QPPSamplerV2

__all__ = [
    "SamplerV2",
    "QPPSamplerV2",
    "QRMIPasqalBackend",
    "QRMIPasqalJob",
    "get_device",
    "get_target",
]
