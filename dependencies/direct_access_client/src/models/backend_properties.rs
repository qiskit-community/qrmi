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

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

/// Recorded parameter as a name-date-unit-value
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Nduv {
    /// Date field
    pub date: String,
    /// Name field
    pub name: String,
    /// Nduv unit
    pub unit: String,
    /// The value of the Nduv
    pub value: f64,
}

/// Gate
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Gate {
    /// The gates name
    pub gate: String,
    /// List of Nduv objects for the name-date-unit-value for the gate
    pub parameters: Vec<Nduv>,
    /// A list of integers representing qubits
    pub qubits: Vec<u64>,
}

/// Qiskit device backend properties as Rust struct
///
/// # Example
///
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use direct_access_api::{AuthMethod, ClientBuilder};
///
///     let client = ClientBuilder::new("http://localhost:8080")
///         .with_auth(AuthMethod::IbmCloudIam {
///             apikey: "your_iam_apikey".to_string(),
///             service_crn: "your_service_crn".to_string(),
///             iam_endpoint_url: "iam_endpoint_url".to_string(),
///         })
///         .build()
///         .unwrap();
///     let props = client
///         .get_backend_properties::<direct_access_api::models::BackendProperties>("ibm_brisbane")
///         .await?;
///     println!("{:#?}", props);
///     Ok(())
/// }
/// ```
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct BackendProperties {
    /// Backend name
    pub backend_name: String,

    /// Backend version in the form X.X.X
    pub backend_version: String,

    /// Last date/time that a property was updated.
    pub last_update_date: String,

    /// System qubit parameters
    pub qubits: Vec<Vec<Nduv>>,

    /// System gate parameters
    pub gates: Vec<Gate>,

    /// General system parameters
    pub general: Vec<Nduv>,
}
