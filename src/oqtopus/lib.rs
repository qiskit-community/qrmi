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

use crate::error::{required_env, QrmiError};
use crate::models::ResourceType;
use crate::{QuantumResource, Result};
use async_trait::async_trait;
use libloading::Library;

use log::{error, info, warn};

/// QRMI implementation for OQTOPUS Cloud
pub struct Oqtopus {
    pub(crate) device_id: String,
}

impl Oqtopus {
    /// Constructs a OQTOPUS cloud instance.
    ///
    /// Environment variables used:
    /// * QRMI_OQTOPUS_BASE_URL - Oqtopus cloud base URL
    /// * QRMI_OQTOPUS_API_TOKEN - Oqtopus cloud API token
    pub fn new(device_id: &str) -> Result<Self> {
        let endpoint = required_env(format!("{device_id}_QRMI_OQTOPUS_BASE_URL"))?;
        let api_token = required_env(format!("{device_id}_QRMI_OQTOPUS_API_TOKEN"))?;
        info!("{}, {}", endpoint, api_token);
        Ok(Self {
            device_id: device_id.to_string(),
        })
    }
}

// Implement the QuantumResource trait using the asynchronous wrappers.
#[async_trait]
impl QuantumResource for Oqtopus {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.device_id.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::OQTOPUS)
    }

    /// Asynchronously checks if a backend is accessible.
    async fn is_accessible(&mut self) -> Result<bool> {
        Ok(true)
    }
}
