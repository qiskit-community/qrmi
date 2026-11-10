# This code is part of Qiskit.
#
# (C) Copyright 2025, 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""EstimatorV2 Primitive implementation with IBM QRMI"""

from warnings import deprecated
from qrmi import QuantumResource
from qrmi.primitives import QRMIBaseEstimatorV2


@deprecated(
    "The EstimatorV2 class is deprecated as of qrmi 0.27.0 and will be removed no sooner "
    "than 3 months after the release date. `qrmi.primitives.ibm.ExecutorEstimator` "
    "provides a new Estimator implementation that runs all the pre- and post-processing "
    "in the client side.",
    category=FutureWarning,
)
class EstimatorV2(QRMIBaseEstimatorV2):
    """EstimatorV2 for QRMI"""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        super().__init__(qrmi, options=options)
