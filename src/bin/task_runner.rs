//
// (C) Copyright IBM 2025-2026
// (C) Copyright UKRI-STFC (Hartree Centre) 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

#![allow(unused_imports)]
use eyre::{eyre, WrapErr};
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{thread, time};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use clap::builder::TypedValueParser as _;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

use qrmi::ibm::{IBMQiskitRuntimeService, IBMQuantumComputeService, IBMQuantumSystem};
use qrmi::maestro::MaestroLocal;
use qrmi::pasqal::PasqalCloud;
use qrmi::pasqal::PasqalLocal;
use qrmi::{models::Payload, models::TaskStatus, QuantumResource};

static IS_RUNNING: AtomicBool = AtomicBool::new(true);

const POLLING_INTERVAL: u64 = 1000;

fn qrmi_job_env(name: &str, legacy_name: &str) -> Result<String, eyre::Report> {
    env::var(name).or_else(|_| env::var(legacy_name)).map_err(|err| {
        eyre!(
            "The environment variable `{}` or legacy `{}` is not set and as such configuration could not be loaded. reason = {}",
            name,
            legacy_name,
            err
        )
    })
}

#[derive(Debug, Clone, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
/// Qiskit Primitive types
pub enum PrimitiveType {
    /// Estimator
    Estimator,
    /// Sampler
    Sampler,
}
impl PrimitiveType {
    fn as_str(&self) -> &str {
        match self {
            PrimitiveType::Estimator => "estimator",
            PrimitiveType::Sampler => "sampler",
        }
    }
}

/// qrmi_payload_v1、The input to QPU resource
#[derive(Serialize, Deserialize, Debug)]
pub struct QrmiInput {
    /// Number of times the pulser sequence is repeated. Required for pasqal-cloud QPU resource
    /// type.
    job_runs: Option<i32>,

    /// Parameters to inject into the primitive. Required for ibm-quantum-system,
    /// ibm-quantum-compute-service and
    /// qiskit-runtime-service(deprecated) QPU resources. Estimator schema:
    /// https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/estimator_v2_schema.json,
    /// Sampler schema:
    /// https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/sampler_v2_schema.json
    parameters: Option<serde_json::Value>,

    /// ID of the primitive to be executed. Required for ibm-quantum-system, ibm-quantum-compute-service and qiskit-runtime-service(deprecated)
    /// QPU resources.
    program_id: Option<PrimitiveType>,

    /// Pulser sequence for pasqal-cloud QPU resource. Required for pasqal-cloud QPU resource
    /// type.
    sequence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Subcommand)]
#[allow(dead_code)]
/// QRMI resource types
pub enum ResourceType {
    /// Maestro local native or legacy task
    MaestroLocal {
        input: String,
        job_type: String,
        qubits: u32,
        simulator_type: u32,
        simulation_method: u32,
        observables: String,
        config: String,
    },
    /// IBM Quantum System
    IBMQuantumSystem {
        /// Qiskit primitive input
        input: String,
        /// Qiskit primitive type
        program_id: PrimitiveType,
    },
    /// IBM Qiskit Runtime Service(deprecated)
    QiskitRuntimeService {
        /// Qiskit primitive input
        input: String,
        /// Qiskit primitive type
        program_id: PrimitiveType,
    },
    /// IBM Quantum Compute Service
    IBMQuantumComputeService {
        /// Qiskit primitive input
        input: String,
        /// Qiskit primitive type
        program_id: PrimitiveType,
    },
    /// Pasqal Cloud
    PasqalCloud {
        /// Pulser sequence
        sequence: String,
        /// Number of times the pulser sequence is repeated.
        job_runs: i32,
    },
    /// Pasqal Local
    PasqalLocal {
        /// Pulser sequence
        sequence: String,
        /// Number of times the pulser sequence is repeated.
        job_runs: i32,
    },
}
impl ResourceType {
    fn new(qpu_type: &str, args: Args) -> Result<Self, Box<dyn std::error::Error>> {
        let payload = match fs::read_to_string(&args.input) {
            Ok(v) => v,
            Err(err) => {
                return Err(eyre!("Failed to open {}. reason = {}", args.input, err).into());
            }
        };
        if qpu_type == "maestro-local" {
            let Some(Payload::MaestroLocal {
                input,
                job_type,
                qubits,
                simulator_type,
                simulation_method,
                observables,
                config,
            }) = MaestroLocal::payload_from_json(&payload)?
            else {
                unreachable!()
            };
            return Ok(Self::MaestroLocal {
                input,
                job_type,
                qubits,
                simulator_type,
                simulation_method,
                observables,
                config,
            });
        }
        let deserialized: QrmiInput = serde_json::from_str(&payload).unwrap();
        if qpu_type == "ibm-quantum-system" {
            let input = match &deserialized.parameters {
                Some(v) => v.to_string(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "parameters").into());
                }
            };
            let program_id = match &deserialized.program_id {
                Some(v) => v.clone(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "program_id").into());
                }
            };
            Ok(Self::IBMQuantumSystem { input, program_id })
        } else if qpu_type == "qiskit-runtime-service" {
            let input = match &deserialized.parameters {
                Some(v) => v.to_string(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "parameters").into());
                }
            };
            let program_id = match &deserialized.program_id {
                Some(v) => v.clone(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "program_id").into());
                }
            };
            Ok(Self::QiskitRuntimeService { input, program_id })
        } else if qpu_type == "ibm-quantum-compute-service" {
            let input = match &deserialized.parameters {
                Some(v) => v.to_string(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "parameters").into());
                }
            };
            let program_id = match &deserialized.program_id {
                Some(v) => v.clone(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "program_id").into());
                }
            };
            Ok(Self::IBMQuantumComputeService { input, program_id })
        } else if qpu_type == "pasqal-cloud" {
            let job_runs = match &deserialized.job_runs {
                Some(v) => v,
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "job_runs").into());
                }
            };
            let sequence = match &deserialized.sequence {
                Some(v) => v.to_string(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "sequence").into());
                }
            };
            Ok(Self::PasqalCloud {
                sequence,
                job_runs: *job_runs,
            })
        } else if qpu_type == "pasqal-local" {
            let job_runs = match &deserialized.job_runs {
                Some(v) => v,
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "job_runs").into());
                }
            };
            let sequence = match &deserialized.sequence {
                Some(v) => v.to_string(),
                None => {
                    return Err(eyre!("Missing property: {} in the payload.", "sequence").into());
                }
            };
            Ok(Self::PasqalLocal {
                sequence,
                job_runs: *job_runs,
            })
        } else {
            Err(
                eyre!(
                    "Resource type {} is not supported. [supported types: ibm-quantum-system, ibm-quantum-compute-service, qiskit-runtime-service(deprecated), pasqal-cloud]",
                    qpu_type,
                ).into()
            )
        }
    }
    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        match self {
            ResourceType::MaestroLocal { .. } => "maestro-local",
            ResourceType::IBMQuantumSystem { .. } => "ibm-quantum-system",
            ResourceType::QiskitRuntimeService { .. } => "qiskit-runtime-service",
            ResourceType::IBMQuantumComputeService { .. } => "ibm-quantum-compute-service",
            ResourceType::PasqalCloud { .. } => "pasqal-cloud",
            ResourceType::PasqalLocal { .. } => "pasqal-local",
        }
    }
    fn to_payload(&self) -> Option<Payload> {
        match self {
            ResourceType::MaestroLocal {
                input,
                job_type,
                qubits,
                simulator_type,
                simulation_method,
                observables,
                config,
            } => Some(Payload::MaestroLocal {
                input: input.clone(),
                job_type: job_type.clone(),
                qubits: *qubits,
                simulator_type: *simulator_type,
                simulation_method: *simulation_method,
                observables: observables.clone(),
                config: config.clone(),
            }),
            ResourceType::IBMQuantumSystem { input, program_id }
            | ResourceType::QiskitRuntimeService { input, program_id }
            | ResourceType::IBMQuantumComputeService { input, program_id } => {
                Some(Payload::QiskitPrimitive {
                    input: input.to_string(),
                    program_id: program_id.as_str().to_string(),
                })
            }
            ResourceType::PasqalCloud { sequence, job_runs } => Some(Payload::PasqalCloud {
                sequence: sequence.to_string(),
                job_runs: *job_runs,
            }),
            ResourceType::PasqalLocal { sequence, job_runs } => Some(Payload::PasqalCloud {
                sequence: sequence.to_string(),
                job_runs: *job_runs,
            }),
        }
    }
    fn create_qrmi(
        &self,
        qpu_name: &str,
    ) -> Result<Box<dyn QuantumResource>, Box<dyn std::error::Error>> {
        match self {
            ResourceType::MaestroLocal { .. } => {
                Ok(Box::new(MaestroLocal::new(qpu_name)?) as Box<dyn QuantumResource>)
            }
            ResourceType::IBMQuantumSystem { .. } => {
                Ok(Box::new(IBMQuantumSystem::new(qpu_name)?) as Box<dyn QuantumResource>)
            }
            ResourceType::QiskitRuntimeService { .. } => {
                Ok(Box::new(IBMQiskitRuntimeService::new(qpu_name)?) as Box<dyn QuantumResource>)
            }
            ResourceType::IBMQuantumComputeService { .. } => {
                Ok(Box::new(IBMQuantumComputeService::new(qpu_name)?) as Box<dyn QuantumResource>)
            }
            ResourceType::PasqalCloud { .. } => {
                Ok(Box::new(PasqalCloud::new(qpu_name)?) as Box<dyn QuantumResource>)
            }
            ResourceType::PasqalLocal { .. } => {
                Ok(Box::new(PasqalLocal::new(qpu_name)?) as Box<dyn QuantumResource>)
            }
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(version = "0.1.0")]
#[command(about = "qrmi_task_runner - Command to run a QRMI task")]
struct Args {
    /// QPU resource name.
    #[arg(value_name = "name")]
    qpu_name: String,

    /// Input to QPU resource.
    #[arg(value_name = "file")]
    input: String,

    /// Write output to <file> instead of stdout.
    #[arg(short, long, value_name = "file")]
    output: Option<String>,
}

// Handle signals, and cancel QPU job if SIGTERM is received.
async fn handle_signals(mut signals: Signals) {
    while let Some(signal) = signals.next().await {
        // To cancel a job, invoke scancel without --signal option. This will send
        // first a SIGCONT to all steps to eventually wake them up followed by a
        // SIGTERM, then wait the KillWait duration defined in the slurm.conf file
        // and finally if they have not terminated send a SIGKILL.
        match signal {
            SIGCONT | SIGTERM => {
                // cancel QPU job
                IS_RUNNING.store(false, Ordering::SeqCst);
            }
            // only registered sinals come
            _ => unreachable!(),
        }
    }
}

// Check to see if the specified file can be created, written and truncated.
// Exit this program immediately if failed.
fn check_file_argument(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .is_err()
    {
        return Err(eyre!("{} cannot be created.", path).into());
    }
    Ok(())
}

fn find_qpu_type(
    qpu_resources: Vec<&str>,
    qpu_types: Vec<&str>,
    qpu_name: String,
) -> Option<String> {
    if let Some(index) = qpu_resources.iter().position(|&r| r == qpu_name) {
        return Some(qpu_types[index].to_string());
    }
    None
}

// Convert SRUN_DEBUG environment value to RUST_LOG value
fn to_rust_loglevel(srun_debug: &str) -> &str {
    match srun_debug.parse::<i32>() {
        Ok(level) => match level {
            // --quiet
            2 => "error",
            // default
            3 => "info",
            // --verbose
            4 => "debug",
            // -vv or more
            n if n >= 5 => "debug",
            // default is Info as same as srun
            _ => "info",
        },
        Err(_) => "info",
    }
}

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Before executing a quantum job, check to see if the specified
    // file can be created, and inform to user if it cannot be written. This is
    // to prevent file writing errors after a long job execution.
    if let Some(ref output_file) = args.output {
        check_file_argument(output_file)?;
    }

    if let Ok(srun_debug) = env::var("SRUN_DEBUG") {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or(to_rust_loglevel(&srun_debug)),
        )
        .init();
    } else {
        // use default
        env_logger::init();
    }

    let delimiter = env::var("QRMI_LIST_DELIMITER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ",".to_string());

    let envvar_qpu_names = qrmi_job_env("QRMI_JOB_QPU_RESOURCES", "SLURM_JOB_QPU_RESOURCES")?;
    let qpu_names: Vec<&str> = envvar_qpu_names.split(&delimiter).collect();

    let envvar_qpu_types = qrmi_job_env("QRMI_JOB_QPU_TYPES", "SLURM_JOB_QPU_TYPES")?;
    let qpu_types: Vec<&str> = envvar_qpu_types.split(&delimiter).collect();

    let qpu_name = args.qpu_name.clone();
    let res_type: ResourceType;
    if let Some(qpu_type) = find_qpu_type(qpu_names, qpu_types, qpu_name.clone()) {
        res_type = ResourceType::new(&qpu_type, args.clone())?;
    } else {
        return Err(eyre!("{} is not specified in --qpu option", qpu_name).into());
    }

    let payload = res_type.to_payload().unwrap();
    let mut qrmi = res_type.create_qrmi(&qpu_name)?;

    // setup signal handler for slurm, and start it
    let signals = Signals::new([SIGTERM, SIGCONT])?;
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_signals(signals));

    let result = run_task(
        qrmi.as_mut(),
        payload,
        args.output.as_deref(),
        &IS_RUNNING,
        Duration::from_millis(POLLING_INTERVAL),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    )
    .await;

    // shutdown signal handler
    handle.close();
    signals_task.await?;

    if result.is_err() {
        // run_task has already written the error and the task diagnostics to stderr.
        std::process::exit(1);
    }
    Ok(())
}

type RunnerResult = Result<(), Box<dyn std::error::Error>>;

/// Submit a task, poll it until it finishes and write its result.
///
/// Succeeds only when the task completed and its result was written to
/// `output_file`, or to `output` when no file is given. Every other outcome is an
/// error: the task failed or was cancelled, `running` was cleared by a signal, or
/// the result could not be retrieved or written. Errors are written to
/// `diagnostics` together with the task's logs, which are retrieved before the
/// task is stopped. A failed log query is reported without replacing the
/// original error. A submitted task is always stopped.
async fn run_task(
    qrmi: &mut dyn QuantumResource,
    payload: Payload,
    output_file: Option<&str>,
    running: &AtomicBool,
    poll_interval: Duration,
    output: &mut dyn Write,
    diagnostics: &mut dyn Write,
) -> RunnerResult {
    let job_id = match qrmi.task_start(payload).await {
        Ok(job_id) => job_id,
        Err(err) => {
            let _ = writeln!(diagnostics, "Error: Failed to start task. reason = {err}");
            return Err(err.into());
        }
    };

    // Keep every exit after submission inside this block so that cleanup always runs.
    let result: RunnerResult = async {
        writeln!(output, "Task ID: {job_id}")?;
        // Poll the task status until it progresses to a final state such as TaskStatus::Completed.
        while running.load(Ordering::SeqCst) {
            match qrmi.task_status(&job_id).await {
                Ok(TaskStatus::Completed) => {
                    let result = qrmi.task_result(&job_id).await?;
                    if let Some(path) = output_file {
                        fs::write(path, result.value.as_bytes())?;
                        writeln!(output, "Wrote output to {path}.")?;
                    } else {
                        writeln!(output, "{}", result.value)?;
                    }
                    return Ok(());
                }
                Ok(status @ (TaskStatus::Failed | TaskStatus::Cancelled)) => {
                    return Err(eyre!("Task {job_id} {status:?}").into());
                }
                Ok(_) => {}
                Err(err) => {
                    let _ = writeln!(
                        diagnostics,
                        "Error: Failed to get task status. reason = {err}. Retrying."
                    );
                }
            }
            tokio::time::sleep(poll_interval).await;
        }
        // SIGTERM/SIGCONT stopped the polling loop before the task finished.
        Err(eyre!("Task {job_id} cancelled").into())
    }
    .await;

    if let Err(error) = &result {
        let _ = writeln!(diagnostics, "Error: {error}");
        match qrmi.task_logs(&job_id).await {
            Ok(logs) if !logs.is_empty() => {
                let _ = writeln!(diagnostics, "{logs}");
            }
            Ok(_) => {}
            Err(err) => {
                let _ = writeln!(
                    diagnostics,
                    "Error: Unable to retrieve task logs. reason = {err}"
                );
            }
        }
    }

    // cleanup quantum task
    if let Err(err) = qrmi.task_stop(&job_id).await {
        let _ = writeln!(diagnostics, "Error: Failed to stop task. reason = {err}");
    }
    result
}

#[cfg(test)]
mod runner_outcome_tests {
    use super::*;

    /// A resource whose task immediately reaches `status`, recording each call.
    struct Resource {
        status: TaskStatus,
        fail_start: bool,
        fail_logs: bool,
        fail_result: bool,
        calls: Vec<&'static str>,
    }

    impl Resource {
        fn new(status: TaskStatus) -> Self {
            Self {
                status,
                fail_start: false,
                fail_logs: false,
                fail_result: false,
                calls: Vec::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl QuantumResource for Resource {
        async fn resource_id(&mut self) -> qrmi::Result<String> {
            Ok("test".into())
        }
        async fn resource_type(&mut self) -> qrmi::Result<qrmi::models::ResourceType> {
            Ok(qrmi::models::ResourceType::MaestroLocal)
        }
        async fn is_accessible(&mut self) -> qrmi::Result<bool> {
            Ok(true)
        }
        async fn task_start(&mut self, _: Payload) -> qrmi::Result<String> {
            self.calls.push("start");
            if self.fail_start {
                Err(qrmi::QrmiError::InvalidInput("bad payload".into()))
            } else {
                Ok("1".into())
            }
        }
        async fn task_status(&mut self, _: &str) -> qrmi::Result<TaskStatus> {
            self.calls.push("status");
            Ok(self.status.clone())
        }
        async fn task_result(&mut self, _: &str) -> qrmi::Result<qrmi::models::TaskResult> {
            self.calls.push("result");
            if self.fail_result {
                Err(qrmi::QrmiError::InvalidInput("result unavailable".into()))
            } else {
                Ok(qrmi::models::TaskResult {
                    value: "task-result".into(),
                })
            }
        }
        async fn task_logs(&mut self, _: &str) -> qrmi::Result<String> {
            self.calls.push("logs");
            if self.fail_logs {
                Err(qrmi::QrmiError::UnsupportedFunction(
                    "logs unavailable".into(),
                ))
            } else {
                Ok("backend diagnostic".into())
            }
        }
        async fn task_stop(&mut self, _: &str) -> qrmi::Result<()> {
            self.calls.push("stop");
            Ok(())
        }
    }

    fn payload() -> Payload {
        Payload::MaestroLocal {
            input: "{}".into(),
            job_type: "request".into(),
            qubits: 0,
            simulator_type: 0,
            simulation_method: 0,
            observables: String::new(),
            config: "{}".into(),
        }
    }

    /// Run `resource` to its end, returning the outcome, stdout and stderr.
    async fn run(
        resource: &mut Resource,
        output_file: Option<&str>,
        running: bool,
    ) -> (RunnerResult, String, String) {
        let (mut output, mut diagnostics) = (Vec::new(), Vec::new());
        let result = run_task(
            resource,
            payload(),
            output_file,
            &AtomicBool::new(running),
            Duration::ZERO,
            &mut output,
            &mut diagnostics,
        )
        .await;
        (
            result,
            String::from_utf8(output).unwrap(),
            String::from_utf8(diagnostics).unwrap(),
        )
    }

    #[tokio::test]
    async fn completed_task_writes_result_and_cleans_up() {
        let mut resource = Resource::new(TaskStatus::Completed);
        let (result, output, diagnostics) = run(&mut resource, None, true).await;
        result.unwrap();
        assert!(output.contains("task-result"));
        assert_eq!(diagnostics, "");
        assert_eq!(resource.calls, ["start", "status", "result", "stop"]);
    }

    #[tokio::test]
    async fn unsuccessful_outcomes_report_diagnostics_then_clean_up() {
        for (status, fail_logs, fail_result, running, expected) in [
            (TaskStatus::Failed, false, false, true, "Failed"),
            (TaskStatus::Cancelled, false, false, true, "Cancelled"),
            (TaskStatus::Failed, true, false, true, "Failed"),
            (
                TaskStatus::Completed,
                false,
                true,
                true,
                "result unavailable",
            ),
            (TaskStatus::Running, false, false, false, "cancelled"),
        ] {
            let mut resource = Resource::new(status);
            resource.fail_logs = fail_logs;
            resource.fail_result = fail_result;
            let (result, output, diagnostics) = run(&mut resource, None, running).await;
            assert!(result.unwrap_err().to_string().contains(expected));
            assert!(diagnostics.contains(expected), "{diagnostics}");
            // A failed log query is reported without hiding the original error.
            let detail = if fail_logs {
                "logs unavailable"
            } else {
                "backend diagnostic"
            };
            assert!(diagnostics.contains(detail), "{diagnostics}");
            assert!(!output.contains(detail));
            // Logs are retrieved before the task is stopped.
            assert_eq!(resource.calls[resource.calls.len() - 2..], ["logs", "stop"]);
        }
    }

    #[tokio::test]
    async fn output_write_failure_is_an_error_and_cleans_up() {
        let mut resource = Resource::new(TaskStatus::Completed);
        let path = std::env::temp_dir()
            .join(uuid::Uuid::new_v4().to_string())
            .join("output");
        let (result, _, diagnostics) = run(&mut resource, path.to_str(), true).await;
        assert!(result.is_err());
        assert!(diagnostics.starts_with("Error: "), "{diagnostics}");
        assert_eq!(resource.calls.last(), Some(&"stop"));
    }

    #[tokio::test]
    async fn start_failure_is_reported_without_cleanup() {
        let mut resource = Resource::new(TaskStatus::Completed);
        resource.fail_start = true;
        let (result, output, diagnostics) = run(&mut resource, None, true).await;
        assert!(result.is_err());
        assert_eq!(output, "");
        assert!(diagnostics.contains("bad payload"), "{diagnostics}");
        assert_eq!(resource.calls, ["start"]);
    }
}
