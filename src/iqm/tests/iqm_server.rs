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

use super::super::IQMServer;
use crate::models::ResourceType;
use crate::QuantumResource;
use iqm_server_api::apis::configuration;

#[tokio::test]
async fn resource_id_and_type_match_backend() {
    const BACKEND_NAME: &str = "sirius:mock";

    // Create a service instance with dummy configuration for testing
    // Note: The configuration values are not used in this test since we are only
    // testing the resource_id and resource_type methods.
    // hence their values don't matter
    let mut qrmi = IQMServer {
        config: configuration::Configuration::new(),
        backend_name: BACKEND_NAME.to_string(),
        acquisition_token: None,
    };

    let resource_id = qrmi
        .resource_id()
        .await
        .expect("resource_id should succeed");
    let resource_type = qrmi
        .resource_type()
        .await
        .expect("resource_type should succeed");

    assert_eq!(resource_id, BACKEND_NAME);
    assert_eq!(resource_type, ResourceType::IQMServer);
}
