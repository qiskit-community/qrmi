use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use log::{debug, info, warn};
use qrmi::ionq::{IonQCloud, IonQMock};
use qrmi::models::{Payload, TaskResult, TaskStatus, Target};
use qrmi::QuantumResource;
use std::fs;
use std::time::Duration;
use tokio::time::{sleep, timeout};

const DEFAULT_QASM2: &str = r#"OPENQASM 2.0;
include "qelib1.inc";
qreg q[1];
creg c[1];
h q[0];
measure q[0] -> c[0];
"#;

const DEFAULT_QASM3: &str = r#"OPENQASM 3.0;
qubit[1] q;
bit[1] c;
h q[0];
c[0] = measure q[0];
"#;

#[derive(ValueEnum, Debug, Clone)]
enum Format {
    Qasm2,
    Qasm3,
    Qir,
}

#[derive(Parser, Debug)]
#[command(name = "qrmi-ionq-cloud-example")]
#[command(about = "Run a simple QRMI IonQCloud job (or IonQMock offline).")]
struct Args {
    /// IonQ backend name, e.g. simulator, qpu.harmony, qpu.aria-1, ...
    #[arg(long, default_value = "simulator")]
    backend: String,

    /// Number of shots
    #[arg(long, default_value_t = 100)]
    shots: i32,

    /// Use the in-process offline mock (no network / no key required)
    #[arg(long)]
    mock: bool,

    /// Default input format (only used when --input/--input-file is not set)
    #[arg(long, value_enum, default_value_t = Format::Qasm2)]
    format: Format,

    /// Provide program text directly
    #[arg(long)]
    input: Option<String>,

    /// Read program text from a file
    #[arg(long)]
    input_file: Option<String>,

    /// Poll interval in seconds
    #[arg(long, default_value_t = 2)]
    poll_s: u64,

    /// Max time to wait for completion in seconds
    #[arg(long, default_value_t = 180)]
    timeout_s: u64,
}

fn default_program(fmt: &Format) -> String {
    match fmt {
        Format::Qasm2 => DEFAULT_QASM2.to_string(),
        Format::Qasm3 => DEFAULT_QASM3.to_string(),
        Format::Qir => {
            warn!("format=qir selected but no default QIR module is provided; prefer --input-file");
            "; QIR placeholder - pass --input-file with real IR\n".to_string()
        }
    }
}

fn load_program(args: &Args) -> Result<String> {
    if let Some(s) = &args.input {
        return Ok(s.clone());
    }
    if let Some(p) = &args.input_file {
        return fs::read_to_string(p).with_context(|| format!("failed reading --input-file={p}"));
    }
    Ok(default_program(&args.format))
}

fn is_final(st: &TaskStatus) -> bool {
    matches!(
        st,
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
    )
}

async fn best_effort_target(qr: &mut dyn QuantumResource) {
    match qr.target().await {
        Ok(Target { value }) => info!("target:\n{value}"),
        Err(e) => warn!("target() failed: {e}"),
    }
}

async fn best_effort_result(qr: &mut dyn QuantumResource, task_id: &str) {
    match qr.task_result(task_id).await {
        Ok(TaskResult { value }) => info!("task result:\n{value}"),
        Err(e) => warn!("task_result() failed: {e}"),
    }
}

async fn best_effort_logs(qr: &mut dyn QuantumResource, task_id: &str) {
    match qr.task_logs(task_id).await {
        Ok(logs) => info!("task logs:\n{logs}"),
        Err(e) => warn!("task_logs() failed: {e}"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if !args.mock {
        let k = std::env::var("QRMI_IONQ_CLOUD_API_KEY").unwrap_or_default();
        if k.is_empty() {
            warn!(
                "QRMI_IONQ_CLOUD_API_KEY is not set. IonQCloud calls will likely fail.\n\
                 Set it with: export QRMI_IONQ_CLOUD_API_KEY=\"...\" \n\
                 Or run with --mock for offline testing."
            );
        }
    }

    let program = load_program(&args)?;
    debug!("program bytes={}", program.len());

    let mut qr: Box<dyn QuantumResource> = if args.mock {
        Box::new(IonQMock::new(&args.backend)?)
    } else {
        Box::new(IonQCloud::new(&args.backend)?)
    };

    info!("is_accessible={}", qr.is_accessible().await?);
    best_effort_target(qr.as_mut()).await;

    let payload = Payload::IonQCloud {
        input: program,
        shots: args.shots,
    };

    let task_id = qr.task_start(payload).await?;
    info!("task_id={task_id}");

    let poll = Duration::from_secs(args.poll_s);
    let wait = Duration::from_secs(args.timeout_s);

    let final_status = match timeout(wait, async {
        loop {
            let st = qr.task_status(&task_id).await?;
            info!("status={st:?}");
            if is_final(&st) {
                return Ok::<TaskStatus, anyhow::Error>(st);
            }
            sleep(poll).await;
        }
    })
    .await
    {
        Ok(r) => r?,
        Err(_) => {
            warn!("timeout reached; attempting to cancel task...");
            let _ = qr.task_stop(&task_id).await;
            // best-effort readback:
            qr.task_status(&task_id).await.unwrap_or(TaskStatus::Cancelled)
        }
    };

    if matches!(final_status, TaskStatus::Completed) {
        best_effort_result(qr.as_mut(), &task_id).await;
    } else {
        warn!("job finished in non-completed state: {final_status:?}");
        // still try to print result/logs for debugging
        best_effort_result(qr.as_mut(), &task_id).await;
    }
    best_effort_logs(qr.as_mut(), &task_id).await;

    Ok(())
}
