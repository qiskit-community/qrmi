// This code is part of Qiskit.
//
// (C) Copyright IBM 2025-2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Trait for vendor-level quantum resource providers.

use crate::QuantumResource;
use anyhow::Result;
use async_trait::async_trait;

/// Defines an interface for vendors that can enumerate available quantum resources.
///
/// A `ResourceProvider` represents a vendor or service endpoint and is responsible
/// for discovering and returning the set of [`QuantumResource`] instances it exposes.
///
/// # Example
///
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use qrmi::ibm::IBMQiskitRuntimeServiceProvider;
///     use qrmi::resource_provider::ResourceProvider;
///
///     let provider = IBMQiskitRuntimeServiceProvider::new()?;
///     let resources = provider.backends("".to_string()).await?;
///     for mut r in resources {
///         println!("{}", r.resource_id().await?);
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// Returns a list of available quantum resources, optionally filtered.
    ///
    /// # Arguments
    ///
    /// * `filters` - A vendor-specific filter string. Pass an empty string for no filtering.
    ///   The format and supported filter keys are vendor-defined.
    async fn backends(&self, filters: String) -> Result<Vec<Box<dyn QuantumResource + Send + Sync>>>;
}
