//
// (C) Copyright IBM 2026
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

/// Resource capacity.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(eq, get_all, skip_from_py_object),
    gen_stub_pyclass
)]
pub struct ResourceCapacity {
    /// Number of slots currently available to be acquired.
    pub available_slots: u64,
    /// Maximum number of slots the resource supports.
    pub max_slots: u64,
}

/// Resource statuses.
#[repr(C)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(eq, eq_int, hash, frozen, skip_from_py_object),
    gen_stub_pyclass_enum
)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceStatusCode {
    /// Online and available.
    Online,
    /// Offline and unavailable.
    Offline,
    /// Online but not currently accepting/running jobs due to a
    /// maintenance-type event (e.g. calibration),
    Paused,
    /// Online and available, but occupied by other users' jobs or with
    /// a large pending queue, so a submitted job will not run immediately
    Busy,
}

impl ResourceStatusCode {
    /// Returns a lowercase, human-readable representation
    /// ("online", "offline", "paused", "busy").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Paused => "paused",
            Self::Busy => "busy",
        }
    }
}

/// cbindgen:ignore
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[repr(C)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(eq, get_all, skip_from_py_object),
    gen_stub_pyclass
)]
pub struct ResourceStatus {
    /// The resource's current status.
    pub status: ResourceStatusCode,
    /// Vendor-specific status reason, such as "maintenance" or
    /// "calibration". `None` when the vendor does not report a reason.
    pub status_reason: Option<String>,
    /// Whether the quantum computer is healthy. `None` when the vendor
    /// does not report health information.
    pub healthy: Option<bool>,
    /// The resource's capacity. `None` when the vendor does not report
    /// capacity information.
    pub capacity: Option<ResourceCapacity>,
    /// The number of jobs pending in the queue. `None` when the resource
    /// has no queue, or the vendor does not report queue information.
    pub pending_job_count: Option<u64>,
}

impl ResourceStatus {
    /// Returns whether a job can currently be submitted to the resource.
    /// Matches the previous behavior of `is_accessible()`
    /// (true when Online or Busy).
    pub fn is_accessible(&self) -> bool {
        matches!(
            self.status,
            ResourceStatusCode::Online | ResourceStatusCode::Busy
        )
    }
}

// Python-facing methods. Thin wrappers around the plain inherent impl
// above, plus Python-convenience helpers (e.g. to_dict()).
#[cfg(feature = "pyo3")]
#[gen_stub_pymethods]
#[pymethods]
impl ResourceStatus {
    /// Returns whether a job can currently be submitted to the resource
    /// (``True`` when the status is ``ONLINE`` or ``BUSY``).
    #[pyo3(name = "is_accessible")]
    fn py_is_accessible(&self) -> bool {
        self.is_accessible()
    }

    /// Returns this status as a plain, JSON-serializable dict.
    /// Equivalent to calling ``json.dumps(status.to_dict())``.
    ///
    /// Field values mirror this struct's ``serde::Serialize``
    /// implementation: ``status`` is a lowercase string (e.g.
    /// ``"online"``), and fields the vendor does not report
    /// (``status_reason``, ``healthy``, ``capacity``,
    /// ``pending_job_count``) are ``None``.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        use pyo3::exceptions::PyValueError;
        pythonize::pythonize(py, self).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

#[cfg(feature = "pyo3")]
define_stub_info_gatherer!(stub_info);
