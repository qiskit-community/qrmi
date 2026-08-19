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

"""QRMI Service

``QRMIService`` is now implemented in Rust and exposed here via the
``qrmi._core`` extension module (re-exported from the top-level ``qrmi``
package), rather than being implemented in pure Python in this file.

This module is kept as a thin re-export so that
``from qrmi.primitives.service import QRMIService`` -- the import path used
by earlier releases, when this module contained the actual implementation
-- keeps working unchanged. New code should prefer importing from the
package instead: ``from qrmi.primitives import QRMIService`` or
``from qrmi import QRMIService``.
"""

from qrmi import QRMIService  # pylint: disable=no-name-in-module

__all__ = ["QRMIService"]
