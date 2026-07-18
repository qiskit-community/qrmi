// This code is part of Qiskit.
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

//! Trait for vendor-level quantum resource providers.

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
///     use qrmi::models::Config;
///     use qrmi::resource_provider::{ResourceProvider, create_provider};
///
///     let config = Config::load("/path/to/qrmi_config.json")?;
///     let resource_def = &config.resource_map["ibm_inst1"];
///     let provider = create_provider(&resource_def.r#type, &resource_def.environment)?;
///
///     // List all resources
///     let resources = provider.resources(None).await?;
///     for mut r in resources {
///         println!("{}", r.resource_id().await?);
///     }
///
///     // Get the least busy resource
///     if let Some(mut r) = provider.least_busy(None).await? {
///         println!("least busy: {}", r.resource_id().await?);
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait ResourceProvider: Send + Sync {
    /// Returns a list of available quantum resources, optionally filtered.
    ///
    /// Results are sorted by `queue_length` ascending (least busy first).
    ///
    /// # Arguments
    ///
    /// * `filters` - A vendor-specific filter string, or `None` for no filtering.
    ///   The format and supported filter keys are vendor-defined.
    ///   Example: `Some("num_qubits=127&name=ibm_*&status=online")`
    async fn resources(
        &self,
        filters: Option<String>,
    ) -> Result<Vec<Box<dyn QuantumResource + Send + Sync>>>;

    /// Returns the least busy available quantum resource, optionally filtered.
    ///
    /// This is equivalent to calling [`resources`] and taking the first element.
    /// Returns `None` if no resources match the given filters.
    ///
    /// # Arguments
    ///
    /// * `filters` - Same filter string as [`resources`], or `None` for no filtering.
    async fn least_busy(
        &self,
        filters: Option<String>,
    ) -> Result<Option<Box<dyn QuantumResource + Send + Sync>>> {
        Ok(self.resources(filters).await?.into_iter().next())
    }
}
