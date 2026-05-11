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

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::anyhow;
use anyhow::{bail, Result};
use pasqal_local_api::{Client, ClientBuilder, JobStatus};
use std::collections::HashMap;
use std::env;

use async_trait::async_trait;

/// QRMI implementation for Pasqal Local
pub struct PasqalLocal {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
    pub(crate) job_uid: i32,
    pub(crate) job_id: String,
}

impl PasqalLocal {
    /// Constructs a QRMI to access Pasqal on prem QPU
    ///
    /// # Arguments
    ///
    /// * `backend_name` - The name of the backend/device to use
    ///
    /// # Environment variables
    /// * `QRMI_JOB_UID`: uid of the slurm job
    /// * `<backend_name>_QRMI_URL`: URL of the pasqd middleware (warden)
    ///
    pub fn new(backend_name: &str) -> Result<Self> {
        let url_var = format!("{backend_name}_QRMI_URL");
        let url =
            env::var(&url_var).map_err(|_| anyhow!("{url_var} environment variable is not set"))?;
        let job_uid: i32 = env::var("QRMI_JOB_UID")
            .ok()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap();
        let job_id: String = env::var("QRMI_JOB_ID").ok().unwrap();
        Ok(Self {
            api_client: ClientBuilder::new(url).build().unwrap(),
            backend_name: backend_name.to_string(),
            job_uid,
            job_id,
        })
    }
}

#[async_trait]
impl QuantumResource for PasqalLocal {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::PasqalLocal)
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        match self.api_client.get_accessible().await {
            Ok(accessible) => Ok(accessible.is_accessible),
            Err(err) => Err(err),
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        match self
            .api_client
            .create_session(self.job_uid, &self.job_id)
            .await
        {
            Ok(session) => Ok(session.id),
            Err(err) => Err(err),
        }
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        let token_var = format!("{}_QRMI_JOB_ACQUISITION_TOKEN", self.backend_name);
        let session_id = env::var(&token_var)
            .map_err(|_| anyhow!("{token_var} environment variable is not set"))?;
        match self.api_client.revoke_session(&session_id).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let token_var = format!("{}_QRMI_JOB_ACQUISITION_TOKEN", self.backend_name);
        let session_id = env::var(&token_var)
            .map_err(|_| anyhow!("{token_var} environment variable is not set"))?;

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

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        match self.api_client.get_task_logs(task_id).await {
            Ok(resp) => Ok(resp.logs),
            Err(err) => Err(err),
        }
    }

    async fn target(&mut self) -> Result<Target> {
        match self.api_client.get_device_specs().await {
            Ok(resp) => Ok(Target { value: resp.specs }),
            Err(err) => Err(err),
        }
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}
