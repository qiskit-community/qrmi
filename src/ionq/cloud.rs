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
use anyhow::{bail, Result};
use async_trait::async_trait;
use ionq_cloud_api::{Backend, Client, ClientBuilder, IonQJob, SessionData, SessionRequestData};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

pub struct IonQCloud {
    pub(crate) api_client: Client,
    pub(crate) backend: Backend,
    pub(crate) session_request_data: SessionRequestData, // QuantumResource trait is fixed but maybe we should add this somewhere else?
    pub(crate) session_data: Option<SessionData>,
    pub(crate) use_sessions: bool,
}

fn ionq_job_type() -> &'static str {
    // IonQ v0.4 "Create job" expects type ionq.circuit.v1.
    "ionq.circuit.v1"
}

fn parse_bracket_index(s: &str) -> Option<u32> {
    let l = s.find('[')?;
    let r = s[l + 1..].find(']')? + (l + 1);
    s[l + 1..r].trim().parse::<u32>().ok()
}

fn qasm_to_ionq_qis(qasm: &str) -> Result<Value> {
    // Minimal, pragmatic translator for simple examples.
    // - Extract qubit count from `qreg q[n];` (QASM2) or `qubit[n] q;` (QASM3).
    // - Translate a small set of gates into IonQ QIS gates.
    // - Ignore measurement statements (IonQ returns measurement probabilities for the final state).

    let mut n_qubits: Option<u32> = None;
    let mut circuit: Vec<Value> = Vec::new();

    for raw in qasm.lines() {
        let no_comment = raw.split("//").next().unwrap_or(raw);
        let line = no_comment.trim();
        if line.is_empty() {
            continue;
        }

        let lower = line.to_ascii_lowercase();

        // Headers / declarations
        if lower.starts_with("openqasm") || lower.starts_with("include ") {
            continue;
        }
        if lower.starts_with("qreg ") || lower.starts_with("qubit") {
            if let Some(n) = parse_bracket_index(line) {
                n_qubits = Some(n);
            }
            continue;
        }
        if lower.starts_with("creg ") || lower.starts_with("bit") {
            continue;
        }

        // Ignore measurements (QASM2 and QASM3 forms)
        if lower.starts_with("measure ") || lower.contains("= measure") {
            continue;
        }

        // Normalize: strip trailing ';'
        let stmt = line.trim_end_matches(';').trim();
        let mut it = stmt.split_whitespace();
        let gate = it.next().unwrap_or("");
        let rest = it.collect::<Vec<_>>().join(" ");

        match gate.to_ascii_lowercase().as_str() {
            "h" | "x" | "y" | "z" | "s" | "t" => {
                let tgt = parse_bracket_index(&rest).ok_or_else(|| {
                    anyhow::anyhow!("Failed to parse target from QASM stmt: '{stmt}'")
                })?;
                circuit.push(serde_json::json!({
                    "gate": gate.to_ascii_lowercase(),
                    "target": tgt,
                }));
            }

            "cx" | "cnot" => {
                let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                if parts.len() != 2 {
                    bail!("Unsupported CX/CNOT form (expected 'cx q[a],q[b];'): '{stmt}'");
                }
                let c = parse_bracket_index(parts[0])
                    .ok_or_else(|| anyhow::anyhow!("Failed to parse control from '{stmt}'"))?;
                let t = parse_bracket_index(parts[1])
                    .ok_or_else(|| anyhow::anyhow!("Failed to parse target from '{stmt}'"))?;
                circuit.push(serde_json::json!({
                    "gate": "cnot",
                    "control": c,
                    "target": t,
                }));
            }

            other => {
                bail!(
                    "Unsupported gate '{other}' in QASM for IonQ example. \
                     For end-to-end demo, stick to h/x/y/z/s/t/cx (or pass IonQ circuit JSON directly). \
                     Offending stmt: '{stmt}'"
                );
            }
        }
    }

    let qubits = n_qubits.unwrap_or(1);
    Ok(serde_json::json!({
        "qubits": qubits,
        "gateset": "qis",
        "circuit": circuit,
    }))
}

fn build_ionq_input(program: &str) -> Result<Value> {
    let s = program.trim_start();

    // If it's already JSON, accept it as an IonQ "input" object.
    if s.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<Value>(s) {
            if v.get("qubits").is_some() && v.get("circuit").is_some() {
                return Ok(v);
            }
        }
    }

    // Otherwise treat it as OpenQASM (2 or 3) and translate.
    if s.starts_with("OPENQASM") || s.to_ascii_uppercase().contains("OPENQASM") {
        return qasm_to_ionq_qis(program);
    }

    bail!("IonQCloud input must be IonQ circuit JSON (the 'input' object) or OpenQASM 2/3 text.");
}

fn map_ionq_status(s: &str) -> TaskStatus {
    let x = s.trim().to_ascii_lowercase();
    match x.as_str() {
        "completed" | "done" | "succeeded" | "success" => TaskStatus::Completed,
        "failed" | "error" => TaskStatus::Failed,
        "canceled" | "cancelled" => TaskStatus::Cancelled,
        "running" => TaskStatus::Running,
        // submitted/ready/queued/unknown
        _ => TaskStatus::Queued,
    }
}

fn extract_probabilities(raw: serde_json::Value) -> serde_json::Value {
    // Accept several response shapes:
    // 1) { "0": 0.5, "1": 0.5, ... }
    // 2) { "probabilities": { ... } }
    // 3) { "data": { "probabilities": { ... } } }
    if raw.get("probabilities").is_some() {
        return raw.get("probabilities").cloned().unwrap_or(raw);
    }
    if raw
        .get("results")
        .and_then(|r| r.get("probabilities"))
        .is_some()
    {
        return raw["results"]["probabilities"].clone();
    }

    if raw
        .get("data")
        .and_then(|d| d.get("probabilities"))
        .is_some()
    {
        return raw["data"]["probabilities"].clone();
    }
    if raw
        .get("data")
        .and_then(|d| d.get("results"))
        .and_then(|r| r.get("probabilities"))
        .is_some()
    {
        return raw["data"]["results"]["probabilities"].clone();
    }
    raw
}

impl IonQCloud {
    pub fn new(backend_name: &str) -> Result<Self> {
        let backend = match backend_name.parse::<Backend>() {
            Ok(b) => b,
            Err(_) => {
                let valid_devices = [
                    "simulator",
                    "qpu.harmony",
                    "qpu.aria-1",
                    "qpu.aria-2",
                    "qpu.forte-1",
                    "qpu.forte-enterprise-1",
                    "qpu.forte-enterprise-2",
                ];
                bail!(
                    "Backend '{}' is invalid. Valid backends: {}",
                    backend_name,
                    valid_devices.join(", ")
                );
            }
        };

        let var_name = "QRMI_IONQ_CLOUD_API_KEY";
        let api_key = env::var(var_name).unwrap_or_else(|_| {
            eprintln!("Warning: {var_name} is not set; proceeding with empty api key.");
            String::new()
        });

        let api_client = ClientBuilder::new(api_key).build()?;

        // Sessions are optional for job creation (session_id is optional).
        // For end-to-end simulator demo, don't use sessions.
        let use_sessions = backend != Backend::Simulator;

        Ok(Self {
            api_client,
            backend,
            session_request_data: SessionRequestData {
                backend: backend_name.to_string(),
                limits: None,
            },
            session_data: None,
            use_sessions,
        })
    }
}

#[async_trait]
impl QuantumResource for IonQCloud {
    async fn is_accessible(&mut self) -> Result<bool> {
        // TODO: except for simulator all devices seem to NOT be availabe:
        // curl "https://api.ionq.co/v0.4/backends/qpu.forte-1"
        // what is going on?
        match self.api_client.get_backend(self.backend.clone()).await {
            Ok(device) => Ok(device.status == "available"),
            Err(err) => bail!("Failed to get device: {}", err),
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        if !self.use_sessions {
            // Simulator path: no session management; return a dummy token.
            // (Callers can still follow the QRMI acquire/release pattern if they want.)
            return Ok("no-session".to_string());
        }

        let session = self
            .api_client
            .create_session(self.backend.clone(), &self.session_request_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire session: {e}"))?;
        let id = session.id.clone();
        self.session_data = Some(session);
        Ok(id)
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        if !self.use_sessions {
            // Simulator path: nothing to release.
            self.session_data = None;
            return Ok(());
        }

        self.api_client
            .end_session(_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to release session: {e}"))?;
        self.session_data = None;
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

        // Ensure backend matches the resource configuration (prevent surprises)
        let payload_backend = target.parse::<Backend>().map_err(|_| {
            anyhow::anyhow!("Payload target '{target}' is not a valid IonQ backend string")
        })?;
        if payload_backend != self.backend {
            bail!(
                "Payload target '{}' does not match IonQCloud backend '{}'",
                payload_backend,
                self.backend
            );
        }

        // Build IonQ v0.4 circuit input (QIS) from QASM or accept JSON input.
        let ionq_input = build_ionq_input(&input)?;

        // Optional sessions: only for non-simulator backends.
        if self.use_sessions && self.session_data.is_none() {
            let _ = self.acquire().await?;
        }
        let session_id = self.session_data.as_ref().map(|s| s.id.clone());

        let job_type = ionq_job_type();
        let name = format!("qrmi-ionq-{}", Uuid::new_v4());

        let job: IonQJob = self
            .api_client
            .create_job(
                self.backend.clone(),
                job_type,
                shots,
                &name,
                session_id.as_deref(),
                None,
                None,
                ionq_input,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create IonQ job: {e}"))?;

        Ok(job.id)
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let _ = self
            .api_client
            .cancel_job(task_id.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to cancel IonQ job {task_id}: {e}"))?;
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let job = self
            .api_client
            .get_job(task_id.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get IonQ job {task_id}: {e}"))?;
        Ok(map_ionq_status(&job.status))
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let job = self
            .api_client
            .get_job(task_id.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get IonQ job {task_id}: {e}"))?;

        let st = map_ionq_status(&job.status);
        if st != TaskStatus::Completed {
            bail!("IonQ job {task_id} is not completed yet (status: {:?})", st);
        }

        let raw_probs = self
            .api_client
            .get_job_probabilities(task_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch probabilities for {task_id}: {e}"))?;

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

    async fn task_logs(&mut self, _task_id: &str) -> Result<String> {
        // IonQ doesn’t expose a “stream logs” API in the same style as IBM Runtime.
        // Provide a useful fallback: dump current job object.
        let job = self.api_client.get_job(_task_id.to_string()).await?;
        Ok(format!(
            "IonQ job logs fallback (job details):\n{}",
            serde_json::to_string_pretty(&job)?
        ))
    }

    async fn target(&mut self) -> Result<Target> {
        let backend = self
            .api_client
            .get_backend(self.backend.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get IonQ backend details: {e}"))?;
        Ok(Target {
            value: serde_json::to_string(&backend)?,
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend.to_string());
        // TODO: add
        //metadata.insert("session_request_data".to_string(), self.session_request_data.to_string());
        //metadata.insert("session_data".to_string(), self.session_data.to_string());
        metadata
    }
}
