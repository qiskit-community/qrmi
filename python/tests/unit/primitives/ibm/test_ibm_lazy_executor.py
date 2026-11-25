# This code is part of Qiskit.
#
# (C) Copyright 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""qrmi.primitives.ibm must import even when the executor cannot (issue #373)."""

import importlib
import sys

import pytest


@pytest.fixture(name="ibm_without_executor")
def fixture_ibm_without_executor(monkeypatch):
    """Reload qrmi.primitives.ibm with its executor submodule made unimportable."""
    for mod in ("qrmi.primitives.ibm", "qrmi.primitives.ibm.executor"):
        monkeypatch.delitem(sys.modules, mod, raising=False)
    # A None entry in sys.modules makes `import` raise ImportError.
    monkeypatch.setitem(sys.modules, "qrmi.primitives.ibm.executor", None)
    module = importlib.import_module("qrmi.primitives.ibm")
    yield module
    monkeypatch.delitem(sys.modules, "qrmi.primitives.ibm", raising=False)


def test_non_executor_symbols_still_import(ibm_without_executor):
    """SamplerV2, EstimatorV2, get_target and get_backend import without the executor."""
    for name in ("SamplerV2", "EstimatorV2", "get_target", "get_backend"):
        assert hasattr(ibm_without_executor, name)


@pytest.mark.parametrize(
    "name", ["ExecutorJob", "ExecutorSampler", "ExecutorEstimator", "Executor"]
)
def test_executor_symbols_raise_clear_error(ibm_without_executor, name):
    """Executor symbols raise an ImportError naming the required runtime version."""
    with pytest.raises(ImportError, match=r"qiskit-ibm-runtime>=0\.51\.0"):
        getattr(ibm_without_executor, name)


def test_unknown_attribute_raises_attribute_error(ibm_without_executor):
    """Unknown names still raise AttributeError."""
    with pytest.raises(AttributeError):
        getattr(ibm_without_executor, "DoesNotExist")
