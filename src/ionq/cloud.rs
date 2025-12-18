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

/// QRMI implementation for IonQ Cloud

use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use ionq_cloud_api::{Client, ClientBuilder, DeviceType};
use std::collections::HashMap;
use std::env;

pub struct IonQCloud {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

impl IonQCloud {
    /// Constructs a QRMI to access IonQ Cloud Service
    ///
    /// # Environment variables
    ///
    /// * `QRMI_IONQ_CLOUD_API_KEY`: IONQ API KEY
    ///
    pub fn new(backend_name: &str) -> Result<Self> {

        let var_name = "QRMI_IONQ_CLOUD_API_KEY".to_string();
        let api_key = env::var(&var_name).unwrap_or_else(|_| {
            // Do not need api key to query backends info
            eprintln!("Warning: {var_name} is not set; proceeding with empty api key.");
            String::new()
        });

        Ok(Self {
            api_client: ClientBuilder::new(api_key).build().unwrap(),
            backend_name: backend_name.to_string(),
        })
    }
}

#[async_trait]
impl QuantumResource for IonQCloud {
    async fn is_accessible(&mut self) -> Result<bool> {
        let device_type = match self.backend_name.parse::<DeviceType>() {
            Ok(dt) => dt,
            Err(_) => {
                let valid_devices = vec!["simulator", "qpu.harmony", "qpu.aria-1", "qpu.aria-2", "qpu.forte-1", "qpu.forte-enterprise-1", "qpu.forte-enterprise-2"];
                let err = format!(
                    "Device '{}' is invalid. Valid devices: {}",
                    self.backend_name,
                    valid_devices.join(", ")
                );
                bail!(err);
            }
        };

        // TODO: except for simulator all devices seem to not be availabe:
        // curl "https://api.ionq.co/v0.4/backends/qpu.forte-1"
        // what is going on?
        match self.api_client.get_device(device_type).await {
            Ok(device) => Ok(device.status == "available"),
            Err(err) => bail!("Failed to get device: {}", err),
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        todo!()
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        todo!()
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        todo!()
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        todo!()
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
         todo!()
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
         todo!()
    }

    async fn task_logs(&mut self, _task_id: &str) -> Result<String> {
        todo!()
    }

    async fn target(&mut self) -> Result<Target> {
        todo!()
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}