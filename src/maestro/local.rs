// This code is part of Qiskit.
//
// (C) Copyright IBM, Qoro Quantum 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::common::resolve_opt;
use crate::models::{
    Payload, ResourceStatus, ResourceStatusCode, ResourceType, Target, TaskResult, TaskStatus,
};
use crate::{QrmiError, QuantumResource, Result};
use anyhow::anyhow;
use async_trait::async_trait;
use maestro_local_api::maestro::request::{Client as RequestClient, Error as RequestError};
use maestro_local_api::maestro::session::{Session, Task, TaskType};
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::path::PathBuf;

/// QRMI implementation for Maestro Local.
///
/// Session configuration is read when the resource is constructed. An acquired
/// session is retained by this instance; exporting its token is only necessary
/// when another process or resource instance needs to use the same session.
pub struct MaestroLocal {
    backend_name: String,
    session_id: Option<u32>,
    request_client: RequestClient,
    // Maestro removes tasks when results are retrieved or cancellation succeeds.
    // Remember terminal states we have actually observed, scoped to the session.
    terminal_tasks: HashMap<(u32, u32), TaskStatus>,
}

/// The fields of a legacy execute/estimate submission, moved to a blocking thread.
struct LegacySubmission {
    session_id: u32,
    task_type: TaskType,
    input: String,
    observables: String,
    config: String,
    qubits: u32,
    simulator_type: u32,
    simulation_method: u32,
}

impl MaestroLocal {
    // The socket client is synchronous, and a cleanup reply may take a minute.
    // Keep that blocking work off async executor threads, including for callers
    // using a non-Tokio executor.
    async fn blocking<T: Send + 'static>(
        &self,
        operation: impl FnOnce(PathBuf) -> T + Send + 'static,
    ) -> Result<T> {
        let socket = self.request_client.socket_path().to_path_buf();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime
                .spawn_blocking(move || operation(socket))
                .await
                .map_err(|error| anyhow!("Maestro socket worker failed: {error}").into())
        } else {
            let (sender, receiver) = futures::channel::oneshot::channel();
            std::thread::spawn(move || {
                let _ = sender.send(operation(socket));
            });
            receiver
                .await
                .map_err(|error| anyhow!("Maestro socket worker failed: {error}").into())
        }
    }

    /// Runs one legacy task exchange on a blocking thread. These client calls
    /// are async in signature only: they perform synchronous socket I/O.
    async fn task_call<T, F>(
        &self,
        task: &Task,
        call: impl FnOnce(Task, PathBuf) -> F + Send + 'static,
    ) -> Result<T>
    where
        T: Send + 'static,
        F: Future<Output = T>,
    {
        let task = Task {
            id: task.id,
            session_id: task.session_id,
        };
        self.blocking(move |socket| futures::executor::block_on(call(task, socket)))
            .await
    }

    async fn task_exists(&self, task: &Task) -> Result<std::result::Result<bool, String>> {
        self.task_call(
            task,
            |task, socket| async move { task.exists_at(&socket).await },
        )
        .await
    }

    async fn native_api<T: Send + 'static>(
        &self,
        operation: impl FnOnce(RequestClient) -> std::result::Result<T, RequestError> + Send + 'static,
    ) -> Result<T> {
        let client = self.request_client.clone();
        let value = self.blocking(move |_| operation(client)).await?;
        value.map_err(|error| match error.code.as_str() {
            "invalid_input" => QrmiError::InvalidInput(error.message),
            "unsupported_api"
            | "unsupported_version"
            | "unsupported_capability"
            | "unsupported_operation" => QrmiError::UnsupportedFunction(error.to_string()),
            "invalid_config" | "session_not_found" => QrmiError::InvalidConfig(error.to_string()),
            "task_not_found" => QrmiError::TaskNotFound(error.message),
            _ => anyhow!("Maestro API: {error}").into(),
        })
    }

    /// Constructs a Maestro Local instance.
    ///
    /// Environment variables used:
    /// * `<backend_name>_QRMI_JOB_ACQUISITION_TOKEN`: optional existing session ID.
    /// * `QRMI_MAESTRO_SOCKET`: optional socket path (default `/run/maestro.sock`).
    pub fn new(backend_name: &str) -> Result<Self> {
        let name = format!("{backend_name}_QRMI_JOB_ACQUISITION_TOKEN");
        let session_id = match env::var(&name) {
            Ok(value) => Some(
                value
                    .parse::<u32>()
                    .map_err(|source| QrmiError::ParseError {
                        name,
                        value,
                        source: Box::new(source),
                    })?,
            ),
            Err(env::VarError::NotPresent) => None,
            Err(env::VarError::NotUnicode(_)) => {
                return Err(QrmiError::InvalidConfig(format!(
                    "{name} is not valid UTF-8"
                )));
            }
        };
        Ok(Self {
            backend_name: backend_name.to_string(),
            session_id,
            request_client: RequestClient::default(),
            terminal_tasks: HashMap::new(),
        })
    }

    /// Constructs a Maestro Local instance from an explicit configuration map.
    ///
    /// Accepts `QRMI_MAESTRO_SOCKET` (default `/run/maestro.sock`) and
    /// `QRMI_JOB_ACQUISITION_TOKEN` (optional existing session ID), without a
    /// backend-name prefix. Fully lowercase keys are accepted as a fallback.
    /// This constructor does not read or modify the process environment.
    pub fn from_config(backend_name: &str, config: HashMap<String, String>) -> Result<Self> {
        let name = "QRMI_JOB_ACQUISITION_TOKEN";
        let session_id = resolve_opt(name, Some(&config))
            .map(|value| {
                value
                    .parse::<u32>()
                    .map_err(|source| QrmiError::ParseError {
                        name: name.into(),
                        value,
                        source: Box::new(source),
                    })
            })
            .transpose()?;
        let socket = resolve_opt("QRMI_MAESTRO_SOCKET", Some(&config))
            .unwrap_or_else(|| "/run/maestro.sock".into());
        Ok(Self {
            backend_name: backend_name.into(),
            session_id,
            request_client: RequestClient::new(socket),
            terminal_tasks: HashMap::new(),
        })
    }

    fn get_session_id(&self) -> Result<u32> {
        self.session_id.ok_or_else(|| QrmiError::InvalidConfig(format!(
            "No session for {}. Call acquire() or set {}_QRMI_JOB_ACQUISITION_TOKEN before constructing the resource",
            self.backend_name, self.backend_name
        )))
    }

    fn task(&self, task_id: &str) -> Result<Task> {
        let id = task_id
            .parse::<u32>()
            .map_err(|_| QrmiError::InvalidInput(format!("Invalid task ID: {task_id}")))?;
        Ok(Task {
            id,
            session_id: self.get_session_id()?,
        })
    }

    async fn ensure_task_exists(&self, task: &Task) -> Result<()> {
        if self
            .task_exists(task)
            .await?
            .map_err(|e| anyhow!("Failed to check task {}: {e}", task.id))?
        {
            Ok(())
        } else {
            Err(QrmiError::TaskNotFound(task.id.to_string()))
        }
    }

    /// Preserve the original failure unless the server confirms that the task
    /// disappeared between the existence check and the requested operation.
    async fn task_response<T>(
        &self,
        task: &Task,
        operation: &str,
        response: std::result::Result<T, String>,
    ) -> Result<T> {
        match response {
            Ok(value) => Ok(value),
            Err(error) => {
                if matches!(self.task_exists(task).await, Ok(Ok(false))) {
                    return Err(QrmiError::TaskNotFound(task.id.to_string()));
                }
                Err(anyhow!("{operation} for task {}: {error}", task.id).into())
            }
        }
    }

    /// Whether the server's retained diagnostics end with a confirmed
    /// cancellation of this task. They outlive the task until its session ends.
    /// Older servers without diagnostics, unreachable servers and any other
    /// history all answer false.
    async fn cancellation_recorded(&self, task: &Task) -> bool {
        let (session_id, id) = (task.session_id, task.id);
        let Ok(logs) = self
            .native_api(move |client| client.logs(session_id, id))
            .await
        else {
            return false;
        };
        serde_json::from_str::<serde_json::Value>(&logs)
            .ok()
            .and_then(|value| {
                let last = value["events"].as_array()?.last()?.clone();
                Some(last["state"] == "cancelled")
            })
            .unwrap_or(false)
    }

    /// The server's view of a task that this instance has not seen finish.
    async fn server_task_status(&self, task: &Task) -> Result<TaskStatus> {
        self.ensure_task_exists(task).await?;
        let failed = self
            .task_call(
                task,
                |task, socket| async move { task.failed_at(&socket).await },
            )
            .await?;
        if self
            .task_response(task, "Failed to get failure status", failed)
            .await?
        {
            return Ok(TaskStatus::Failed);
        }
        let finished = self
            .task_call(task, |task, socket| async move {
                task.finished_at(&socket).await
            })
            .await?;
        if self
            .task_response(task, "Failed to get completion status", finished)
            .await?
        {
            // Failure can become visible between FAILED and FINISHED.
            // Once FINISHED is true, re-read failure before caching status.
            let failed = self
                .task_call(
                    task,
                    |task, socket| async move { task.failed_at(&socket).await },
                )
                .await?;
            return Ok(
                if self
                    .task_response(task, "Failed to get final failure status", failed)
                    .await?
                {
                    TaskStatus::Failed
                } else {
                    TaskStatus::Completed
                },
            );
        }
        let running = self
            .task_call(task, |task, socket| async move {
                task.running_at(&socket).await
            })
            .await?;
        Ok(
            if self
                .task_response(task, "Failed to get running status", running)
                .await?
            {
                TaskStatus::Running
            } else {
                TaskStatus::Queued
            },
        )
    }

    /// Creates, configures and executes a legacy task. Runs on a blocking thread.
    async fn submit_legacy(job: LegacySubmission, socket: PathBuf) -> Result<Task> {
        let socket = socket.as_path();
        let session = Session { id: job.session_id };
        let task = session
            .create_task_at(socket)
            .await
            .map_err(|e| anyhow!("Failed to create task: {e}"))?;
        let estimate = matches!(job.task_type, TaskType::ESTIMATE);
        let configured = async {
            Self::accepted(
                "set task type",
                task.set_type_at(job.task_type, socket).await,
            )?;
            if estimate {
                Self::accepted(
                    "set observables",
                    task.set_observables_as_string_at(job.observables, socket)
                        .await,
                )?;
            }
            Self::accepted("set qubits", task.set_qubits_at(job.qubits, socket).await)?;
            Self::accepted(
                "set simulator type",
                task.set_simulator_type_at(job.simulator_type, socket).await,
            )?;
            Self::accepted(
                "set simulation method",
                task.set_simulation_method_at(job.simulation_method, socket)
                    .await,
            )?;
            Self::accepted("set QASM", task.set_qasm_at(job.input, socket).await)?;
            Self::accepted(
                "set options JSON",
                task.set_options_json_at(job.config, socket).await,
            )?;
            Self::accepted("execute task", task.execute_at(socket).await)
        }
        .await;
        if let Err(error) = configured {
            // A failed submission otherwise loses the only handle to the task.
            if !matches!(task.cancel_at(socket).await, Ok(true)) {
                log::warn!(
                    "Failed to clean up Maestro task {} after rejected submission",
                    task.id
                );
            }
            // task_start errors other than Other promise that no task was
            // created. Cancellation does not undo creation, and may itself fail.
            return Err(anyhow!(
                "Maestro task {} was created but submission failed: {error}",
                task.id
            )
            .into());
        }
        Ok(task)
    }

    fn accepted(operation: &str, response: std::result::Result<bool, String>) -> Result<()> {
        match response {
            Ok(true) => Ok(()),
            Ok(false) => Err(QrmiError::InvalidInput(format!(
                "Maestro rejected {operation}"
            ))),
            Err(error) => Err(anyhow!("Failed to {operation}: {error}").into()),
        }
    }
}

#[async_trait]
impl QuantumResource for MaestroLocal {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::MaestroLocal)
    }

    async fn status(&mut self) -> Result<ResourceStatus> {
        let status = self.native_api(|client| client.status()).await?;
        Ok(ResourceStatus {
            status: if status.accepting_jobs {
                ResourceStatusCode::Online
            } else {
                ResourceStatusCode::Offline
            },
            status_reason: status.reason,
            healthy: status.healthy,
            busy: status.busy,
            pending_job_count: status.pending_job_count,
            capacity: None,
        })
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        use maestro_local_api::maestro_lib::Response;
        match self
            .blocking(|socket| maestro_local_api::maestro_lib::send_command_at(&socket, "PING\n"))
            .await?
        {
            Response::OK(accessible) => Ok(accessible),
            Response::ERROR(error) => Err(anyhow!("Error pinging Maestro Local: {error}").into()),
            response => Err(anyhow!("Unexpected Maestro Local ping response: {response:?}").into()),
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        if let Some(id) = self.session_id {
            if self
                .blocking(move |socket| {
                    futures::executor::block_on(Session::session_exists_at(id, &socket))
                })
                .await?
                .map_err(|e| anyhow!("Failed to check session {id}: {e}"))?
            {
                return Ok(id.to_string());
            }
            self.session_id = None;
            self.terminal_tasks.clear();
        }
        let session = self
            .blocking(|socket| Session::new_at(&socket))
            .await?
            .map_err(|e| anyhow!("Failed to acquire a new session: {e}"))?;
        self.session_id = Some(session.get_id());
        Ok(session.get_id().to_string())
    }

    async fn release(&mut self, id: &str) -> Result<()> {
        // The explicit acquisition token identifies the session being released.
        // An environment variable must never redirect this operation elsewhere.
        let session_id = id
            .parse::<u32>()
            .map_err(|_| QrmiError::InvalidInput(format!("Invalid session ID: {id}")))?;
        // Deletion waits for the server to stop the session's active worker.
        let deleted = self
            .blocking(move |socket| {
                futures::executor::block_on(Session { id: session_id }.delete_at(&socket))
            })
            .await?;
        match deleted {
            Ok(true) => {
                if self.session_id == Some(session_id) {
                    self.session_id = None;
                }
                self.terminal_tasks
                    .retain(|(session, _), _| *session != session_id);
                Ok(())
            }
            Ok(false) => Err(anyhow!("Maestro refused to delete session {id}").into()),
            Err(error) => Err(anyhow!("Error deleting session {id}: {error}").into()),
        }
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let Payload::MaestroLocal {
            input,
            job_type,
            qubits,
            simulator_type,
            simulation_method,
            observables,
            config,
        } = payload
        else {
            return Err(QrmiError::UnsupportedPayload(format!("{payload:?}")));
        };

        if job_type.eq_ignore_ascii_case("request") {
            let request: serde_json::Value = serde_json::from_str(&input)?;
            if !request.is_object() || request["schema_version"].as_u64() != Some(2) {
                return Err(QrmiError::InvalidInput(
                    "Maestro request requires a JSON object with schema_version=2".into(),
                ));
            }
            // All new configuration lives in the native document. These old
            // positional fields are intentionally unused in request mode.
            let session_id = self.get_session_id()?;
            let id = self
                .native_api(move |client| client.submit(session_id, &input))
                .await?;
            self.terminal_tasks.remove(&(session_id, id));
            return Ok(id.to_string());
        }

        // Reject malformed input before creating a task on the server.
        let task_type = if job_type.eq_ignore_ascii_case("execute") {
            TaskType::EXECUTE
        } else if job_type.eq_ignore_ascii_case("estimate") {
            if observables.trim().is_empty() {
                return Err(QrmiError::InvalidInput(
                    "Observables must be provided for ESTIMATE job type".into(),
                ));
            }
            TaskType::ESTIMATE
        } else {
            return Err(QrmiError::InvalidInput(format!(
                "Invalid job_type: {job_type}"
            )));
        };
        if input.trim().is_empty() || qubits == 0 {
            return Err(QrmiError::InvalidInput(
                "A nonempty QASM circuit and a positive qubit count are required".into(),
            ));
        }
        let options: serde_json::Value = serde_json::from_str(&config)?;
        if !options.is_object() {
            return Err(QrmiError::InvalidInput(
                "Maestro config must be a JSON object".into(),
            ));
        }

        let job = LegacySubmission {
            session_id: self.get_session_id()?,
            task_type,
            input,
            observables,
            config,
            qubits,
            simulator_type,
            simulation_method,
        };
        let task = self
            .blocking(move |socket| futures::executor::block_on(Self::submit_legacy(job, socket)))
            .await??;
        self.terminal_tasks.remove(&(task.session_id, task.id));
        Ok(task.id.to_string())
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let task = self.task(task_id)?;
        if matches!(
            self.task_status(task_id).await?,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            return Ok(());
        }
        // The server replies after the worker has stopped; the client allows
        // for its whole cleanup budget, so a slow success is not lost.
        let response = self
            .task_call(&task, |task, socket| async move {
                task.cancel_at(&socket).await
            })
            .await?;
        if matches!(response, Ok(true)) {
            self.terminal_tasks
                .insert((task.session_id, task.id), TaskStatus::Cancelled);
            return Ok(());
        }
        // A task may finish between the status check and the cancel request.
        // A negative acknowledgement while it remains active is a real failure.
        match self.task_status(task_id).await {
            Ok(TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled) => Ok(()),
            Err(error @ QrmiError::TaskNotFound(_)) => Err(error),
            _ => match response {
                Ok(false) => Err(anyhow!("Maestro refused to cancel active task {task_id}").into()),
                Err(error) => Err(anyhow!("Failed to cancel task {task_id}: {error}").into()),
                Ok(true) => unreachable!(),
            },
        }
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let task = self.task(task_id)?;
        let key = (task.session_id, task.id);
        if let Some(status) = self.terminal_tasks.get(&key) {
            return Ok(status.clone());
        }
        let status = match self.server_task_status(&task).await {
            // Cancellation removes the task. Retained diagnostics still confirm a
            // cancellation made by another client, or one whose acknowledgement
            // was lost, so runners see a terminal state. A missing task alone
            // proves neither cancellation nor failure and stays TaskNotFound.
            Err(QrmiError::TaskNotFound(id)) => {
                if !self.cancellation_recorded(&task).await {
                    return Err(QrmiError::TaskNotFound(id));
                }
                TaskStatus::Cancelled
            }
            status => status?,
        };
        if matches!(
            status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        ) {
            self.terminal_tasks.insert(key, status.clone());
        }
        Ok(status)
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let task = self.task(task_id)?;
        let status = self.task_status(task_id).await?;
        if status != TaskStatus::Completed {
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: format!("Task is not completed (current status: {status:?})"),
            });
        }
        // GET_RESULTS consumes the server-side task. Repeated result retrieval
        // reports TaskNotFound, while the known Completed status stays available.
        self.ensure_task_exists(&task).await?;
        let results = self
            .task_call(&task, |task, socket| async move {
                task.get_results_as_string_at(&socket).await
            })
            .await?;
        let value = self
            .task_response(&task, "Failed to get results", results)
            .await?;
        Ok(TaskResult { value })
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let task = self.task(task_id)?;
        self.native_api(move |client| client.logs(task.session_id, task.id))
            .await
    }

    async fn target(&mut self) -> Result<Target> {
        let value = self.native_api(|client| client.capabilities()).await?;
        Ok(Target { value })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        HashMap::from([("backend_name".to_string(), self.backend_name.clone())])
    }
}

#[cfg(test)]
#[path = "tests/local.rs"]
mod tests;
