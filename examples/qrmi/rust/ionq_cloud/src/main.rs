use anyhow::{Context, Result};
use clap::Parser;
use log::{debug, info, warn};
use qrmi::ionq::{IonQCloud, IonQMock};
use qrmi::models::{Payload, TaskResult, TaskStatus, Target};
use qrmi::QuantumResource;
use std::fs;
use std::time::{Duration, Instant};
use tokio::time::sleep;

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
    #[arg(long, default_value_t = false)]
    mock: bool,

    /// Input format used only to choose the default program when --input/--input-file is not set.
    /// Options: qasm2 | qasm3 | qir
    #[arg(long, default_value = "qasm2")]
    format: String,

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

fn pick_default_program(format: &str) -> String {
    match format.to_lowercase().as_str() {
        "qasm3" => DEFAULT_QASM3.to_string(),
        // For QIR, you almost certainly want to pass --input-file with a real module.
        "qir" => {
            warn!("format=qir selected but no default QIR module is provided; using a placeholder. Prefer --input-file.");
            "; QIR placeholder - pass --input-file with real IR\n".to_string()
        }
        _ => DEFAULT_QASM2.to_string(),
    }
}

fn looks_final(status: &TaskStatus) -> bool {
    // Avoid hard-coding enum variants here so the example remains robust if
    // TaskStatus evolves. We just rely on Debug string contents.
    let s = format!("{status:?}").to_lowercase();
    s.contains("completed") || s.contains("failed") || s.contains("cancel")
}

async fn try_print_target(qr: &mut dyn QuantumResource) {
    match qr.target().await {
        Ok(Target { value }) => {
            info!("target:\n{value}");
        }
        Err(e) => {
            warn!("target() failed: {e}");
        }
    }
}

async fn try_print_logs(qr: &mut dyn QuantumResource, task_id: &str) {
    match qr.task_logs(task_id).await {
        Ok(logs) => info!("task logs:\n{logs}"),
        Err(e) => warn!("task_logs() failed: {e}"),
    }
}

async fn try_print_result(qr: &mut dyn QuantumResource, task_id: &str) {
    match qr.task_result(task_id).await {
        Ok(TaskResult { value }) => info!("task result:\n{value}"),
        Err(e) => warn!("task_result() failed: {e}"),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Logging: use RUST_LOG to control verbosity.
    // Example: RUST_LOG=info,qrmi=debug,ionq_cloud_api=debug
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

    // Resolve program text
    let program = if let Some(s) = args.input {
        s
    } else if let Some(p) = args.input_file {
        fs::read_to_string(&p).with_context(|| format!("failed reading --input-file={p}"))?
    } else {
        pick_default_program(&args.format)
    };

    debug!("program bytes={}", program.len());

    // Pick implementation (real vs mock)
    let mut qr: Box<dyn QuantumResource> = if args.mock {
        Box::new(IonQMock::new(&args.backend)?)
    } else {
        Box::new(IonQCloud::new(&args.backend)?)
    };

    // Accessibility probe
    let accessible = qr.is_accessible().await?;
    info!("is_accessible={accessible}");
    if !accessible && !args.mock {
        warn!("Backend '{}' reports not accessible; continuing anyway.", args.backend);
    }

    // Acquire session (IonQ uses sessions; your implementation should store/track it internally)
    let session_id = qr.acquire().await?;
    info!("acquired session_id={session_id}");

    // Optional: print target info if implemented
    try_print_target(qr.as_mut()).await;

    // Start task
    let payload = Payload::IonQCloud {
        input: program,
        target: args.backend.clone(),
        shots: args.shots,
    };

    let task_id = qr.task_start(payload).await?;
    info!("task_id={task_id}");

    // Poll status
    let deadline = Instant::now() + Duration::from_secs(args.timeout_s);
    loop {
        let st = qr.task_status(&task_id).await?;
        info!("status={st:?}");

        if looks_final(&st) {
            break;
        }

        if Instant::now() >= deadline {
            warn!("timeout reached; attempting to stop task...");
            // Best-effort cancel
            let _ = qr.task_stop(&task_id).await;
            break;
        }

        sleep(Duration::from_secs(args.poll_s)).await;
    }

    // Try to fetch result/logs (best-effort)
    try_print_result(qr.as_mut(), &task_id).await;
    try_print_logs(qr.as_mut(), &task_id).await;

    // Release session
    qr.release(&session_id).await?;
    info!("released session_id={session_id}");

    Ok(())
}
