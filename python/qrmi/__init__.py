# -*- coding: utf-8 -*-

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

"""QRMI python package"""

from importlib import import_module as _im

_core = _im(__name__ + "._core")  # -> qrmi._core  (Rust)

globals().update({k: v for k, v in _core.__dict__.items() if not k.startswith("_")})
