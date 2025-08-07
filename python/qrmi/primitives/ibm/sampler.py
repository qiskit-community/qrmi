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

"""SamplerV2 Primitive implementation with IBM QRMI"""
from typing import Union
from qrmi import QuantumResource
from qrmi.primitives import QRMIBaseSamplerV2


class SamplerV2(QRMIBaseSamplerV2):
    """SamplerV2 for QRMI"""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        super().__init__(qrmi, options=options)
