//
// (C) Copyright IonQ 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! # ionq_cloud_client
//!
//! This is a Rust client to interact with IonQ Cloud Services using the API.
//!

mod client;
mod models;

pub use client::{Client, ClientBuilder, IonQBackend, IonQJob, SessionData, SessionRequestData};
pub use models::Backend;
pub use models::JobStatus;
