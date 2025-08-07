// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{bail, Result};
use pasqal_cloud_api::{BatchStatus, Client, ClientBuilder, DeviceType};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

use async_trait::async_trait;

/// QRMI implementation for Pasqal Cloud
pub struct PasqalCloud {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

impl PasqalCloud {
    /// Constructs a QRMI to access Pasqal Cloud Service
    ///
    /// # Environment variables
    ///
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID`: Pasqal Cloud Project ID to access the QPU
    /// * `<backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN`: Pasqal Cloud Auth Token
    ///
    /// Let's hardcode the rest for now
    pub fn new(backend_name: &str) -> Self {
        // Check to see if the environment variables required to run this program are set.
        let project_id = env::var(format!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID"))
            .unwrap_or_else(|_| panic!("{backend_name}_QRMI_PASQAL_CLOUD_PROJECT_ID"));
        let auth_token = env::var(format!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_TOKEN"))
            .unwrap_or_else(|_| panic!("{backend_name}_QRMI_PASQAL_CLOUD_AUTH_TOKEN"));
        Self {
            api_client: ClientBuilder::new(auth_token, project_id).build().unwrap(),
            backend_name: backend_name.to_string(),
        }
    }
}

impl Default for PasqalCloud {
    fn default() -> Self {
        Self::new("")
    }
}
#[async_trait]
impl QuantumResource for PasqalCloud {
    async fn is_accessible(&mut self) -> bool {
        let fresnel = DeviceType::Fresnel.to_string();
        if self.backend_name != fresnel {
            let err = format!(
                "Device {} is invalid. Only {} device can receive jobs.",
                self.backend_name, fresnel,
            );
            panic!("{}", err);
        };
        match self.api_client.get_device(DeviceType::Fresnel).await {
            Ok(device) => device.data.status == "UP",
            Err(_err) => false,
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        // TBD on cloud side for POC
        // Pasqal Cloud does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // TBD on cloud side for POC
        // Pasqal Cloud does not support session concept, so simply ignores
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        if let Payload::PasqalCloud { sequence, job_runs } = payload {
            // TODO: Make configurable (get emulator from qrmi)
            match self
                .api_client
                .create_batch(sequence, job_runs, DeviceType::EmuFree)
                .await
            {
                Ok(batch) => Ok(batch.data.id),
                Err(err) => Err(err),
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload))
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        match self.api_client.cancel_batch(task_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        // TODO: Change for Fresnel after testing
        match self.api_client.get_batch(task_id).await {
            Ok(batch) => {
                let status = match batch.data.status {
                    BatchStatus::Pending => TaskStatus::Queued,
                    BatchStatus::Running => TaskStatus::Running,
                    BatchStatus::Done => TaskStatus::Completed,
                    BatchStatus::Canceled => TaskStatus::Cancelled,
                    BatchStatus::TimedOut => TaskStatus::Failed,
                    BatchStatus::Error => TaskStatus::Failed,
                    BatchStatus::Paused => TaskStatus::Queued,
                };
                return Ok(status);
            }
            Err(err) => Err(err),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        match self.api_client.get_batch_results(task_id).await {
            Ok(resp) => Ok(TaskResult { value: resp }),
            Err(_err) => Err(_err),
        }
    }

    async fn target(&mut self) -> Result<Target> {
        let fresnel = DeviceType::Fresnel.to_string();
        if self.backend_name != fresnel {
            let err = format!(
                "Device {} is invalid. Only {} device can receive jobs.",
                self.backend_name, fresnel
            );
            panic!("{}", err);
        };
        match self.api_client.get_device_specs(DeviceType::Fresnel).await {
            Ok(resp) => Ok(Target {
                value: resp.data.specs,
            }),
            Err(_err) => Err(_err),
        }
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let metadata: HashMap<String, String> = HashMap::new();
        metadata
    }
}
