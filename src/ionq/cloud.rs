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

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use ionq_cloud_api::{Backend, Client, ClientBuilder, IonQJob, SessionRequestData};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

const IONQ_JOB_TYPE: &str = "ionq.circuit.v1";

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

    fn build_ionq_input(program: &str) -> Result<Value> {
        let s = program.trim_start();

        // If it's JSON and looks like an IonQ v0.4 "input" object, accept it.
        // IonQ v0.4 Create Job expects type=ionq.circuit.v1 and an input object
        // (QIS or native circuit).
        if s.starts_with('{') {
            let v: Value =
                serde_json::from_str(s).context("input looked like JSON but failed to parse")?;
            if v.get("qubits").is_some() && v.get("circuit").is_some() {
                return Ok(v);
            }
        }

        // Otherwise treat it as OpenQASM (2 or 3) and translate (minimal demo support).
        if s.to_ascii_uppercase().contains("OPENQASM") {
            return qasm_to_ionq_qis(program);
        }

        bail!(
            "IonQCloud input must be either:\n\
             - IonQ circuit JSON (the v0.4 `input` object with `qubits` + `circuit`), or\n\
             - OpenQASM 2/3 text (limited gate support in built-in demo translator)."
        );
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
    // Accept common shapes:
    // 1) { "0": 0.5, "1": 0.5 }
    // 2) { "probabilities": { ... } }
    // 3) { "results": { "probabilities": { ... } } }
    // 4) { "data": ... } variants
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

fn parse_bracket_index(s: &str) -> Option<u32> {
    let s = s.trim();
    let l = s.find('[')? + 1;
    let r = s[l..].find(']')? + l;
    s[l..r].trim().parse::<u32>().ok()
}

fn qasm_to_ionq_qis(qasm: &str) -> Result<Value> {
    // Minimal translator for examples:
    // - reads qubit count from `qreg q[n];` (QASM2) or `qubit[n] q;` (QASM3)
    // - supports: h/x/y/z/s/t and cx/cnot
    // - ignores measurement lines (IonQ returns final-state probabilities)
    let mut n_qubits: Option<u32> = None;
    let mut circuit: Vec<Value> = Vec::new();

    for raw in qasm.lines() {
        let line = raw.split("//").next().unwrap_or(raw).trim();
        if line.is_empty() {
            continue;
        }
        let stmt = line.trim_end_matches(';').trim();
        let lower = stmt.to_ascii_lowercase();

        // Headers / decls
        if lower.starts_with("openqasm") || lower.starts_with("include ") {
            continue;
        }
        if lower.starts_with("qreg ") || lower.starts_with("qubit") {
            if let Some(n) = parse_bracket_index(stmt) {
                n_qubits = Some(n);
            }
            continue;
        }
        if lower.starts_with("creg ") || lower.starts_with("bit") {
            continue;
        }

        // Ignore measurement statements (QASM2 and QASM3 forms)
        if lower.starts_with("measure ") || lower.contains("= measure") {
            continue;
        }

        // gate + operands
        let gate = stmt.split_whitespace().next().unwrap_or("");
        let rest = stmt[gate.len()..].trim();
        let gate_lc = gate.to_ascii_lowercase();

        match gate_lc.as_str() {
            "h" | "x" | "y" | "z" | "s" | "t" => {
                let target = parse_bracket_index(rest).with_context(|| {
                    format!("failed to parse single-qubit target in stmt: '{stmt}'")
                })?;
                circuit.push(serde_json::json!({
                    "gate": gate_lc,
                    "target": target,
                }));
            }
            "cx" | "cnot" => {
                let mut parts = rest.split(',').map(|s| s.trim());
                let c = parts
                    .next()
                    .and_then(parse_bracket_index)
                    .ok_or_else(|| anyhow::anyhow!("failed to parse control in stmt: '{stmt}'"))?;
                let t = parts
                    .next()
                    .and_then(parse_bracket_index)
                    .ok_or_else(|| anyhow::anyhow!("failed to parse target in stmt: '{stmt}'"))?;
                if parts.next().is_some() {
                    bail!("too many operands for cx/cnot in stmt: '{stmt}'");
                }
                circuit.push(serde_json::json!({
                    "gate": "cnot",
                    "control": c,
                    "target": t,
                }));
            }
            other => bail!(
                "Unsupported gate '{other}' in QASM for IonQ demo translator. \
                 Supported: h/x/y/z/s/t/cx (cnot). Offending stmt: '{stmt}'"
            ),
        }
    }

    let qubits = n_qubits.unwrap_or(1);
    Ok(serde_json::json!({
        "qubits": qubits,
        "gateset": "qis",
        "circuit": circuit,
    }))
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
        // POST /v0.4/sessions
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
        // POST /v0.4/sessions/{session_id}/end
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

        let ionq_input = Self::build_ionq_input(&input)?;

        let job_name = format!("qrmi-ionq-{}", Uuid::new_v4());
        let job: IonQJob = self
            .api_client
            .create_job(
                self.backend,
                IONQ_JOB_TYPE,
                shots,
                &job_name,
                self.session_id.as_deref(),
                None,
                None,
                ionq_input,
            )
            .await
            .context("create_job failed")?;

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
        // IonQ doesn’t expose “logs” like IBM Runtime; best-effort fallback.
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
