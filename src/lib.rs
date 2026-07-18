// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025, 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

pub mod alice_bob;
pub(crate) mod common;
pub(crate) mod consts;
pub mod ibm;
pub mod iqm;
pub mod pasqal;
pub mod resource_provider;
pub use resource_provider::create_provider;
pub use resource_provider::ResourceProvider;

mod cext;
pub mod models;
#[cfg(feature = "pyo3")]
pub mod pyext;

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use anyhow::Result;
use async_trait::async_trait;
