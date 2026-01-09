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
use anyhow::anyhow;
use pasqal_local_api::{Client, ClientBuilder};
use std::collections::HashMap;
use uuid::Uuid;

use async_trait::async_trait;

/// QRMI implementation for Pasqal Cloud
pub struct PasqalLocal {
    pub(crate) api_client: Client,
}

impl PasqalLocal {
    /// Constructs a QRMI to access Pasqal on prem QPU
    ///
    /// # Environment variables
    ///
    ///
    pub fn new() -> Result<Self> {
        Ok(Self {
            api_client: ClientBuilder::new().build().unwrap(),
        })
    }
}

#[async_trait]
impl QuantumResource for PasqalLocal {
    async fn is_accessible(&mut self) -> Result<bool> {
        Ok(true)

        // let device_type = match self.backend_name.parse::<DeviceType>() {
        //     Ok(dt) => dt,
        //     Err(_) => {
        //         let valid_devices = vec!["FRESNEL", "FRESNEL_CAN1", "EMU_MPS", "EMU_FREE", "EMU_FRESNEL"];
        //         let err = format!(
        //             "Device '{}' is invalid. Valid devices: {}",
        //             self.backend_name,
        //             valid_devices.join(", ")
        //         );
        //         bail!(err);
        //     }
        // };

        // // The device may be down temporarily but jobs can still
        // // be submitted and queued through the cloud
        // // Thus we only check that the device is not retired 
        // match self.api_client.get_device(device_type).await {
        //     Ok(device) => Ok(device.availability == "ACTIVE"),
        //     Err(err) => bail!("Failed to get device: {}", err),
        // }
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
            match self
                .api_client
                .create_job(sequence)
                .await
            {
                Ok(job) => Ok(job.id.to_string()),
                Err(err) => Err(err),
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload))
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        // match self.api_client.get_batch(task_id).await {
        //     Ok(batch) => {
        //         let status = match batch.data.status {
        //             BatchStatus::Pending => TaskStatus::Queued,
        //             BatchStatus::Running => TaskStatus::Running,
        //             BatchStatus::Done => TaskStatus::Completed,
        //             BatchStatus::Canceled => TaskStatus::Cancelled,
        //             BatchStatus::TimedOut => TaskStatus::Failed,
        //             BatchStatus::Error => TaskStatus::Failed,
        //             BatchStatus::Paused => TaskStatus::Queued,
        //         };
        //         return Ok(status);
        //     }
        //     Err(err) => Err(err),
        // }
        Ok(TaskStatus::Completed)
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        // match self.api_client.get_batch_results(task_id).await {
        //     Ok(resp) => Ok(TaskResult { value: resp }),
        //     Err(_err) => Err(_err),
        // }
        Err(anyhow!("task_result not implemented yet"))
    }

    async fn task_logs(&mut self, _task_id: &str) -> Result<String> {
        Ok("There are no logs for this job.".to_string())
    }

    async fn target(&mut self) -> Result<Target> {
        Ok(Target {
            value: "placeholder".to_string()
        })
        // let device_type = match self.backend_name.parse::<DeviceType>() {
        //     Ok(dt) => dt,
        //     Err(_) => {
        //         let valid_devices = vec!["FRESNEL", "FRESNEL_CAN1", "EMU_MPS", "EMU_FREE", "EMU_FRESNEL"];
        //         let err = format!(
        //             "Device '{}' is invalid. Valid devices: {}",
        //             self.backend_name,
        //             valid_devices.join(", ")
        //         );
        //         panic!("{}", err);
        //     }
        // };

        // match self.api_client.get_device_specs(device_type).await {
        //     Ok(resp) => Ok(Target {
        //         value: resp.data.specs,
        //     }),
        //     Err(err) => Err(err),
        // }
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata
    }
}
