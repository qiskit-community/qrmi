//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Models used by Direct Access API Client for Rust

mod backend_configuration;
mod backend_properties;
mod backends;

pub(crate) mod auth;
pub(crate) mod errors;
pub(crate) mod jobs;
pub(crate) mod version;
pub(crate) mod versions;

pub use self::backend_configuration::{
    BackendConfiguration, GateConfig, ProcessorType, TimingConstraints,
};
pub use self::backend_properties::{BackendProperties, Gate, Nduv};
pub use self::backends::{Backend, BackendStatus, Backends};
pub use self::errors::{Error, ErrorResponse, ExtendedErrorResponse, Target};
pub use self::jobs::{Job, JobStatus, Jobs, LogLevel, ProgramId, Storage, StorageOption, Usage};
