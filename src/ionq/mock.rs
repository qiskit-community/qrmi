// This code is part of Qiskit.
//
// (C) Copyright IBM, IonQ 2025
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
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// In-memory job record used by the mock IonQ backend.
#[derive(Debug, Clone)]
struct MockJob {
    status: TaskStatus,
    result: Option<TaskResult>,
    logs: Vec<String>,
    payload_summary: String,
}

/// Simple in-process mock for an IonQ device.
///
/// This implementation never talks to the real IonQ APIs. It’s intended
/// for testing QRMI wiring (config, C/Python bindings, etc.) without
/// needing credentials or network access.
#[derive(Debug)]
pub struct IonQMock {
    backend_name: String,
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    online: bool,
    jobs: HashMap<String, MockJob>,
}

impl IonQMock {
    /// Construct a new mock backend with the given logical name.
    ///
    /// The `backend_name` is just metadata that gets surfaced in
    /// `metadata()` and in the synthetic result JSON.
    pub fn new(backend_name: &str) -> Result<Self> {
        Ok(Self {
            backend_name: backend_name.to_string(),
            inner: Mutex::new(Inner {
                online: true,
                jobs: HashMap::new(),
            }),
        })
    }

    /// Optionally flip the "online" bit for tests.
    pub fn set_online(&self, online: bool) {
        let mut inner = self.inner.lock().unwrap();
        inner.online = online;
    }
}

#[async_trait]
impl QuantumResource for IonQMock {
    async fn is_accessible(&mut self) -> Result<bool> {
        let inner = self.inner.lock().unwrap();
        Ok(inner.online)
    }

    async fn acquire(&mut self) -> Result<String> {
        // No real locking in the mock – just return a random token.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // Nothing to release in the mock implementation.
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let mut inner = self.inner.lock().unwrap();

        let job_id = format!("IONQ_MOCK_{}", Uuid::new_v4());

        // Synthesize a result depending on the payload.
        let (summary, result) = match &payload {
            Payload::IonQCloud {
                input,
                target,
                shots,
            } => {
                let preview: String = input.chars().take(128).collect();
                let summary = format!(
                    "IonQ mock job on target='{target}', shots={shots}, input_preview='{}'",
                    preview
                );

                let result_json = json!({
                    "backend": self.backend_name,
                    "job_id": job_id,
                    "mock": true,
                    "provider": "ionq",
                    "target": target,
                    "shots": shots,
                    "input_preview": preview,
                });

                (summary, TaskResult { value: result_json.to_string() })
            }

            other => {
                // This backend is meant primarily for IonQCloud payloads;
                // everything else is still accepted but flagged.
                let summary = format!(
                    "Unsupported payload variant for IonQMock: {:?}",
                    other
                );

                let result_json = json!({
                    "backend": self.backend_name,
                    "job_id": job_id,
                    "mock": true,
                    "warning": "unsupported payload variant for IonQ mock backend",
                });

                (summary, TaskResult { value: result_json.to_string() })
            }
        };

        let job = MockJob {
            status: TaskStatus::Completed,
            result: Some(result),
            logs: vec![
                format!(
                    "job {job_id} started on mock backend '{}'",
                    self.backend_name
                ),
                "job completed immediately by mock backend".to_string(),
            ],
            payload_summary: summary,
        };

        inner.jobs.insert(job_id.clone(), job);

        Ok(job_id)
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();

        if let Some(job) = inner.jobs.get_mut(task_id) {
            // Even though the job already "completed" immediately, we
            // let the caller mark it as cancelled for testing.
            job.status = TaskStatus::Cancelled;
            job.logs
                .push("job marked as cancelled by client request".to_string());
        }

        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let inner = self.inner.lock().unwrap();

        if let Some(job) = inner.jobs.get(task_id) {
            Ok(job.status.clone())
        } else {
            bail!("unknown job id for IonQMock: {task_id}");
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let inner = self.inner.lock().unwrap();

        if let Some(job) = inner.jobs.get(task_id) {
            if let Some(result) = &job.result {
                Ok(result.clone())
            } else {
                bail!("job {task_id} has no result yet");
            }
        } else {
            bail!("unknown job id for IonQMock: {task_id}");
        }
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let inner = self.inner.lock().unwrap();

        if let Some(job) = inner.jobs.get(task_id) {
            let mut logs = job.logs.clone();
            logs.push(format!("payload_summary: {}", job.payload_summary));
            Ok(logs.join("\n"))
        } else {
            bail!("unknown job id for IonQMock: {task_id}");
        }
    }

    async fn target(&mut self) -> Result<Target> {
        // Fabricate a minimal "device description" that you can parse
        // in higher layers if needed.
        let value = json!({
            "name": self.backend_name,
            "provider": "ionq",
            "type": "mock",
            "num_qubits": 29,
            "mock": true,
        })
        .to_string();

        Ok(Target { value })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("backend_name".to_string(), self.backend_name.clone());
        m.insert("provider".to_string(), "ionq".to_string());
        m.insert("kind".to_string(), "mock".to_string());
        m
    }
}
