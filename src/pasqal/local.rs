// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2026
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
use pasqal_local_api::{Client, ClientBuilder, JobStatus};
use std::collections::HashMap;
use std::env;


use async_trait::async_trait;

/// QRMI implementation for Pasqal Cloud
pub struct PasqalLocal {
    pub(crate) api_client: Client,
    pub(crate) job_uid: i32,
    pub(crate) job_id: String
}

impl PasqalLocal {
    /// Constructs a QRMI to access Pasqal on prem QPU
    ///
    /// # Environment variables
    /// /// * `QRMI_JOB_UID`: uid of the slurm job
    /// /// * `PASQAL_LOCAL_QRMI_URL`: URL of the pasqd middleware (warden)
    ///
    pub fn new() -> Result<Self> {
        let url =
            env::var(format!("PASQAL_LOCAL_QRMI_URL")).map_err(|_| {
                anyhow!("PASQAL_LOCAL_QRMI_URL environment variable is not set")
            })?;
        let job_uid: i32 = env::var(format!("QRMI_JOB_UID"))
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap();
        let job_id: String = env::var(format!("QRMI_JOB_ID"))
            .ok()
            .unwrap();
        Ok(Self {
            api_client: ClientBuilder::new(url).build().unwrap(),
            job_uid: job_uid,
            job_id: job_id
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
        match self.api_client.create_session(self.job_uid, &self.job_id).await {
            Ok(session) => Ok(session.id),
            Err(err) => Err(err), 
        }
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        let session_id = env::var("PASQAL_LOCAL_QRMI_JOB_ACQUISITION_TOKEN")
            .map_err(|_| {
                anyhow!("PASQAL_LOCAL_QRMI_JOB_ACQUISITION_TOKEN environment variable is not set")
            })?;
        match self.api_client.revoke_session(&session_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err), 
        }
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let session_id = env::var("PASQAL_LOCAL_QRMI_JOB_ACQUISITION_TOKEN")
            .map_err(|_| {
                anyhow!("PASQAL_LOCAL_QRMI_JOB_ACQUISITION_TOKEN environment variable is not set")
            })?;
        
        if let Payload::PasqalCloud { sequence, job_runs } = payload {
            match self
                .api_client
                .create_job(sequence, job_runs, &session_id)
                .await
            {
                Ok(job) => Ok(job.id.to_string()),
                Err(err) => Err(err),
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload))
        }
    }

    async fn task_stop(&mut self, _task_id: &str) -> Result<()> {
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        match self.api_client.get_job(task_id).await {
            Ok(job) => {
                let status = match job.status {
                    JobStatus::Pending => TaskStatus::Queued,
                    JobStatus::Running => TaskStatus::Running,
                    JobStatus::Done => TaskStatus::Completed,
                    JobStatus::Canceled => TaskStatus::Cancelled,
                    JobStatus::Error => TaskStatus::Failed,
                };
                return Ok(status);
            }
            Err(err) => Err(err),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        match self.api_client.get_job(task_id).await {
            Ok(job) => {
                let Some(results) = job.results else {
                    bail!("Results not available. Current status: {:?}.", job.status);
                };
                Ok(TaskResult { value: results })
            }
            Err(err) => Err(err),
        }
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
        let metadata: HashMap<String, String> = HashMap::new();
        metadata
    }
}
