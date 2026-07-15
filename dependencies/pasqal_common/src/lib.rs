//
// (C) Copyright Pasqal SAS 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Shared HTTP utilities for the Pasqal client libraries.
//!
//! This crate exists so the Pasqal Cloud and Pasqal Local clients can share a
//! single retry implementation rather than duplicating it.

pub mod retry;

pub use retry::{with_retry, DEFAULT_MAX_RETRIES};
