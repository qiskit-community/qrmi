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

#[cfg(feature = "pyo3")]
use {
    pyo3::prelude::*,
    pyo3_stub_gen::{define_stub_info_gatherer, derive::*},
};

/// Task statuses.
#[repr(C)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(eq, eq_int, hash, frozen),
    gen_stub_pyclass_enum
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Task is queued
    Queued,
    /// Task is running
    Running,
    /// Task was completed
    Completed,
    /// Task was failed
    Failed,
    /// Task was cancelled
    Cancelled,
}
#[cfg(feature = "pyo3")]
define_stub_info_gatherer!(stub_info);
