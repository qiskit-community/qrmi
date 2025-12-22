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

/// QRMI implementation for IonQ Cloud
use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use ionq_cloud_api::{Backend, Client, ClientBuilder, IonQJob, SessionRequestData};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

// Job types (IonQ API v0.4).
const JOB_TYPE_CIRCUIT: &str = "ionq.circuit.v1";
const JOB_TYPE_QASM2: &str = "ionq.qasm2.v1";
const JOB_TYPE_QASM3: &str = "ionq.qasm3.v1";
const JOB_TYPE_QIR: &str = "ionq.qir.v1";

pub struct IonQCloud {
    api_client: Client,
    backend: Backend,

    // Sessions are beta/optional in IonQ v0.4. We only use them if the caller
    // explicitly calls acquire(), and we pass the session_id through to create_job().
    session_id: Option<String>,
}

impl IonQCloud {
    pub fn new(backend_name: &str) -> Result<Self> {
        let backend: Backend = backend_name
            .parse()
            .with_context(|| format!("invalid IonQ backend '{backend_name}'"))?;

        let api_key = env::var("QRMI_IONQ_CLOUD_API_KEY").unwrap_or_default();
        let api_client = ClientBuilder::new(api_key).build()?;

        Ok(Self {
            api_client,
            backend,
            session_id: None,
        })
    }

    fn backend_str(&self) -> String {
        self.backend.to_string()
    }

    fn default_noise(&self) -> Option<Value> {
        // Noise model makes sense for the simulator; leave it out for QPUs.
        if self.backend == Backend::Simulator {
            Some(serde_json::json!({ "model": "ideal" }))
        } else {
            None
        }
    }

    fn looks_like_schema_rejection(err: &anyhow::Error) -> bool {
        // ionq_cloud_api errors look like: "Status: 400, Fail { ... }"
        let msg = err.to_string();
        msg.contains("Status: 400") || msg.contains("Status: 422")
    }

    fn build_job_body(
        job_type: &str,
        name: &str,
        shots: i32,
        target_field: &str, // "target" or "backend"
        target_value: &str, // e.g. "simulator"
        input: Value,       // either {data: "..."} or circuit JSON
        session_id: Option<&str>,
        noise: Option<Value>,
    ) -> Result<Value> {
        if shots <= 0 {
            bail!("shots must be > 0");
        }

        let mut m: Map<String, Value> = Map::new();
        m.insert("type".into(), Value::String(job_type.to_string()));
        m.insert("name".into(), Value::String(name.to_string()));
        m.insert(
            "shots".into(),
            Value::Number(serde_json::Number::from(shots as i64)),
        );
        m.insert(target_field.into(), Value::String(target_value.to_string()));
        m.insert("input".into(), input);

        if let Some(sid) = session_id {
            m.insert("session_id".into(), Value::String(sid.to_string()));
        }
        if let Some(noise) = noise {
            m.insert("noise".into(), noise);
        }

        Ok(Value::Object(m))
    }

    fn detect_job_type_and_input(program: &str) -> Result<(&'static str, Value)> {
        let s = program.trim_start();

        // If it's an IonQ circuit JSON *input object* (not a full job request), keep supporting it:
        // { "qubits": ..., "circuit": [...] }
        if s.starts_with('{') {
            if let Ok(v) = serde_json::from_str::<Value>(s) {
                if v.get("qubits").is_some() && v.get("circuit").is_some() {
                    return Ok((JOB_TYPE_CIRCUIT, v));
                }
            }
        }

        // Otherwise, treat it as source text and wrap it.
        // Use the header to decide between QASM2 vs QASM3; default to QIR.
        let first_line = s.lines().next().unwrap_or("").trim();
        let first_uc = first_line.to_ascii_uppercase();

        if first_uc.starts_with("OPENQASM 2") {
            return Ok((JOB_TYPE_QASM2, serde_json::json!({ "data": program })));
        }
        if first_uc.starts_with("OPENQASM 3") {
            return Ok((JOB_TYPE_QASM3, serde_json::json!({ "data": program })));
        }

        Ok((JOB_TYPE_QIR, serde_json::json!({ "data": program })))
    }

    async fn submit_qasm_like_with_target_fallback(
        &self,
        job_type: &str,
        name: &str,
        shots: i32,
        input: Value,
    ) -> Result<IonQJob> {
        let target_value = self.backend_str();
        let session_id = self.session_id.as_deref();
        let noise = self.default_noise();

        // 1) Preferred (your structure): "target"
        let body_target = Self::build_job_body(
            job_type,
            name,
            shots,
            "target",
            &target_value,
            input.clone(),
            session_id,
            noise.clone(),
        )?;

        match self.api_client.create_job_raw(body_target).await {
            Ok(job) => Ok(job),
            Err(e) => {
                // 2) Retry once with documented v0.4 key name: "backend"
                if Self::looks_like_schema_rejection(&e) {
                    let body_backend = Self::build_job_body(
                        job_type,
                        name,
                        shots,
                        "backend",
                        &target_value,
                        input,
                        session_id,
                        noise,
                    )?;
                    return self.api_client.create_job_raw(body_backend).await;
                }
                Err(e)
            }
        }
    }

    async fn submit_circuit_job(&self, name: &str, shots: i32, input: Value) -> Result<IonQJob> {
        let backend_value = self.backend_str();
        let session_id = self.session_id.as_deref();
        let noise = self.default_noise();

        // For the documented circuit job type, IonQ v0.4 docs show "backend".
        let body = Self::build_job_body(
            JOB_TYPE_CIRCUIT,
            name,
            shots,
            "backend",
            &backend_value,
            input,
            session_id,
            noise,
        )?;

        self.api_client.create_job_raw(body).await
    }
}

fn map_ionq_status(s: &str) -> TaskStatus {
    match s.trim().to_ascii_lowercase().as_str() {
        "completed" => TaskStatus::Completed,
        "failed" => TaskStatus::Failed,
        "canceled" | "cancelled" => TaskStatus::Cancelled,
        "started" | "running" => TaskStatus::Running,
        // submitted / ready / queued / unknown
        _ => TaskStatus::Queued,
    }
}

fn extract_probabilities(raw: Value) -> Value {
    const PTRS: [&str; 4] = [
        "/probabilities",
        "/results/probabilities",
        "/data/probabilities",
        "/data/results/probabilities",
    ];
    for p in PTRS {
        if let Some(v) = raw.pointer(p) {
            return v.clone();
        }
    }
    raw
}

#[async_trait]
impl QuantumResource for IonQCloud {
    async fn is_accessible(&mut self) -> Result<bool> {
        let device = self
            .api_client
            .get_backend(self.backend)
            .await
            .context("get_backend failed")?;
        Ok(device.status == "available")
    }

    async fn acquire(&mut self) -> Result<String> {
        // Sessions are beta; only use if caller explicitly wants them.
        let req = SessionRequestData {
            backend: self.backend.to_string(),
            limits: None,
        };

        let session = self
            .api_client
            .create_session(&req)
            .await
            .context("IonQ create_session failed (sessions are beta/limited availability)")?;

        self.session_id = Some(session.id.clone());
        Ok(session.id)
    }

    async fn release(&mut self, id: &str) -> Result<()> {
        self.api_client
            .end_session(id)
            .await
            .context("IonQ end_session failed")?;
        self.session_id = None;
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let Payload::IonQCloud {
            input,
            target,
            shots,
        } = payload
        else {
            bail!("IonQCloud backend only supports Payload::IonQCloud");
        };

        // Ensure payload target matches this resource.
        let payload_backend: Backend = target
            .parse()
            .with_context(|| format!("payload target '{target}' is not a valid IonQ backend"))?;
        if payload_backend != self.backend {
            bail!(
                "payload target '{}' does not match IonQCloud backend '{}'",
                payload_backend,
                self.backend
            );
        }

        let job_name = format!("qrmi-ionq-{}", Uuid::new_v4());

        // If the payload `input` is already a full IonQ create-job JSON body
        // (i.e. it contains "type" and "input"), submit it "as a job" after normalizing
        // a couple fields to match the QRMI request.
        let trimmed = input.trim_start();
        if trimmed.starts_with('{') {
            if let Ok(Value::Object(mut obj)) = serde_json::from_str::<Value>(trimmed) {
                if obj.contains_key("type") && obj.contains_key("input") {
                    // Normalize name/shots
                    obj.entry("name".to_string())
                        .or_insert(Value::String(job_name.clone()));
                    obj.insert(
                        "shots".to_string(),
                        Value::Number(serde_json::Number::from(shots as i64)),
                    );

                    // Ensure target/backend is present and consistent
                    let b = self.backend_str();
                    let has_target = obj.get("target").and_then(|v| v.as_str()).is_some();
                    let has_backend = obj.get("backend").and_then(|v| v.as_str()).is_some();

                    if has_target {
                        obj.insert("target".to_string(), Value::String(b.clone()));
                    } else if has_backend {
                        obj.insert("backend".to_string(), Value::String(b.clone()));
                    } else {
                        // Default to documented key for v0.4.
                        obj.insert("backend".to_string(), Value::String(b.clone()));
                    }

                    // Optional session/noise defaults if omitted
                    if let Some(sid) = self.session_id.as_deref() {
                        obj.entry("session_id".to_string())
                            .or_insert(Value::String(sid.to_string()));
                    }
                    if let Some(noise) = self.default_noise() {
                        obj.entry("noise".to_string()).or_insert(noise);
                    }

                    let job = self.api_client.create_job_raw(Value::Object(obj)).await?;
                    return Ok(job.id);
                }
            }
        }

        // Normal path: wrap raw QASM/QIR (or IonQ circuit JSON input object)
        let (job_type, job_input) = Self::detect_job_type_and_input(&input)?;

        let job: IonQJob = if job_type == JOB_TYPE_CIRCUIT {
            self.submit_circuit_job(&job_name, shots, job_input).await?
        } else {
            // QASM/QIR: submit using your preferred "target" structure first,
            // and retry once with "backend" if the API rejects it.
            self.submit_qasm_like_with_target_fallback(job_type, &job_name, shots, job_input)
                .await?
        };

        Ok(job.id)
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        self.api_client
            .cancel_job(task_id.to_string())
            .await
            .with_context(|| format!("cancel_job failed for {task_id}"))?;
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let job = self
            .api_client
            .get_job(task_id.to_string())
            .await
            .with_context(|| format!("get_job failed for {task_id}"))?;
        Ok(map_ionq_status(&job.status))
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let job = self
            .api_client
            .get_job(task_id.to_string())
            .await
            .with_context(|| format!("get_job failed for {task_id}"))?;

        let st = map_ionq_status(&job.status);
        if st != TaskStatus::Completed {
            bail!("IonQ job {task_id} not completed yet (status: {st:?})");
        }

        let raw_probs = self
            .api_client
            .get_job_probabilities(task_id)
            .await
            .with_context(|| format!("get_job_probabilities failed for {task_id}"))?;

        let probs = extract_probabilities(raw_probs);

        let result_json = serde_json::json!({
            "provider": "ionq",
            "backend": self.backend.to_string(),
            "job_id": task_id,
            "status": job.status,
            "probabilities": probs,
        });

        Ok(TaskResult {
            value: result_json.to_string(),
        })
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        // IonQ doesn't expose "logs" like IBM Runtime; best-effort fallback.
        let job = self.api_client.get_job(task_id.to_string()).await?;
        Ok(format!(
            "IonQ job logs fallback (job details):\n{}",
            serde_json::to_string_pretty(&job)?
        ))
    }

    async fn target(&mut self) -> Result<Target> {
        let backend = self
            .api_client
            .get_backend(self.backend)
            .await
            .context("get_backend failed")?;
        Ok(Target {
            value: serde_json::to_string(&backend)?,
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("backend_name".to_string(), self.backend.to_string());
        if let Some(sid) = &self.session_id {
            m.insert("session_id".to_string(), sid.clone());
        }
        m
    }
}
