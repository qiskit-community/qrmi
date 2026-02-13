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

//! # pasqal_cloud_client
//!
//! This is a Rust client to interact with Pasqal Cloud Services using the API.
//!

mod client;
mod models;

pub use client::DEFAULT_AUTH_ENDPOINT;
pub use client::{AuthError, Client, ClientBuilder};
pub use models::DeviceType;
pub use models::JobStatus;
