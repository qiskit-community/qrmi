// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
use std::os::raw::c_int;

/// @brief C API invocation was succeeded.
pub const QRMI_SUCCESS: c_int = 0;
/// @brief C API invocation was failed.
pub const QRMI_ERROR: c_int = -1;
