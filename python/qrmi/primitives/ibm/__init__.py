# This code is part of Qiskit.
#
# (C) Copyright 2025-2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""IBM implementations of Primitive"""

from .sampler import SamplerV2
from .estimator import EstimatorV2
from .target import get_target
from .backend import get_backend
from typing import TYPE_CHECKING

# The executor depends on qiskit-ibm-runtime internals that first appear in
# 0.51.0. Load it lazily (PEP 562) so SamplerV2, EstimatorV2, get_target and
# get_backend keep working with older qiskit-ibm-runtime versions.
_EXECUTOR_NAMES = ("ExecutorJob", "ExecutorSampler", "ExecutorEstimator", "Executor")

if TYPE_CHECKING:
    from .executor import ExecutorJob, ExecutorSampler, ExecutorEstimator, Executor


def __getattr__(name):
    if name in _EXECUTOR_NAMES:
        try:
            from . import executor  # pylint: disable=import-outside-toplevel
        except ImportError as exc:
            raise ImportError(
                f"qrmi.primitives.ibm.{name} requires qiskit-ibm-runtime>=0.51.0. "
                "Install it with: pip install 'qrmi[ibm]'"
            ) from exc
        return getattr(executor, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__():
    return sorted([*globals(), *_EXECUTOR_NAMES])
