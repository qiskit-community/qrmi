// This code is part of Qiskit.
//
// (C) Copyright IBM, Pasqal 2025, 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

pub mod alice_bob;
pub(crate) mod common;
pub(crate) mod consts;
pub mod error;
pub use error::{QrmiError, QrmiErrorKind};
pub mod ibm;
pub mod iqm;
pub mod pasqal;
pub mod resource_provider;
pub use resource_provider::create_provider;
pub use resource_provider::ResourceProvider;

pub mod service;
pub use service::QRMIService;

mod cext;
pub mod models;
#[cfg(feature = "pyo3")]
pub mod pyext;

// Embeds QRMI's version/git-hash into a `.version_info` ELF section for
// offline inspection via `strings`/`readelf`. Linux (ELF) only: macOS
// (Mach-O) and Windows (PE/COFF) use different, incompatible section-name
// syntaxes, and this marker isn't needed on those platforms, so the whole
// thing is compiled out there instead of trying to support every format.
#[cfg(target_os = "linux")]
mod version_info {
    // `env!` and `concat!` are macros: they are expanded by the compiler's
    // macro expansion pass, long before any type-checking or codegen. This
    // is NOT the same as calling `std::env::var(...)` at runtime — the
    // value of GIT_HASH (set by build.rs above) is baked into the source
    // text itself as if you had typed the literal string by hand.
    const VERSION_STR: &str = concat!(
        "QRMI_BUILD_VERSION:",
        env!("CARGO_PKG_VERSION"),
        ";QRMI_GIT_HASH:",
        env!("GIT_HASH"),
    );

    // A `const` is evaluated by the compiler's constant evaluator (CTFE —
    // Compile-Time Function Evaluation) at compile time. `.len()` on a
    // `&'static str` is computed here, not when the plugin runs.
    const VERSION_LEN: usize = VERSION_STR.len() + 1;

    // A `const fn` can be called from a `const` context, in which case the
    // compiler runs its body through CTFE — effectively a small interpreter
    // built into rustc — during compilation, not at runtime. The `while`
    // loop below never becomes a real loop in the compiled machine code;
    // rustc executes it once, internally, while compiling, and only the
    // resulting array of bytes is kept.
    const fn str_to_array<const N: usize>(s: &str) -> [u8; N] {
        let bytes = s.as_bytes();
        let mut arr = [0u8; N];
        let mut i = 0;
        while i < bytes.len() {
            arr[i] = bytes[i];
            i += 1;
        }
        arr
    }

    // By the time we reach this line, `str_to_array(VERSION_STR)` has
    // ALREADY been fully evaluated by the compiler; VERSION_INFO's contents
    // are a fixed, known byte sequence baked directly into the binary's
    // `.version_info` section (see #[link_section] below). `#[used]` and
    // `#[no_mangle]` are linker/codegen directives, not runtime behavior:
    // they only affect whether/how the linker keeps and names this data —
    // they do not cause any code to run when the plugin is loaded.
    #[no_mangle]
    #[used]
    #[link_section = ".version_info"]
    pub static VERSION_INFO: [u8; VERSION_LEN] = str_to_array(VERSION_STR);
}

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use async_trait::async_trait;

/// Result type used throughout the `QuantumResource` / `ResourceProvider` APIs.
pub type Result<T> = std::result::Result<T, QrmiError>;

/// Defines interfaces to quantum resources.
#[allow(unused_variables)]
#[async_trait]
pub trait QuantumResource: Send + Sync {
    /// Returns resource identifier of this quantum resource.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let resource_id = qrmi.resource_id().await?;
    ///     println!("{resource_id}"); // prints "ibm_torino"
    ///     Ok(())
    /// }
    /// ```
    async fn resource_id(&mut self) -> Result<String> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::resource_id".to_string(),
        ))
    }

    /// Returns resource type of this quantum resource.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let resource_type = qrmi.resource_type().await?;
    ///     println!("{}", resource_type.as_str()); // prints "ibm-quantum-compute-service"
    ///     Ok(())
    /// }
    /// ```
    async fn resource_type(&mut self) -> Result<ResourceType> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::resource_type".to_string(),
        ))
    }

    /// Returns true if device is accessible, otherwise false. A target quantum resource is not considered accessible if quantum workloads cannot be executed, even when the system itself is reachable, for example due to maintenance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let accessible = qrmi.is_accessible().await?;
    ///     if !accessible {
    ///         println!("ibm_torino is not accessible");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    async fn is_accessible(&mut self) -> Result<bool> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::is_accessible".to_string(),
        ))
    }

    /// Acquires quantum resource and returns acquisition token if succeeded. If no one owns the lock, it acquires the lock and returns immediately. If another owns the lock, block until we are able to acquire lock.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let token = qrmi.acquire().await?;
    ///     println!("acquisition token = {}", token);
    ///     Ok(())
    /// }
    /// ```
    async fn acquire(&mut self) -> Result<String> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::acquire".to_string(),
        ))
    }

    /// Releases quantum resource
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     qrmi.release("your_acquisition_token").await?;
    ///     Ok(())
    /// }
    /// ```
    async fn release(&mut self, id: &str) -> Result<()> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::release".to_string(),
        ))
    }

    /// Start a task and returns an identifier of this task if succeeded.
    ///
    /// # Arguments
    ///
    /// * `payload`: payload for task execution. This might be serialized data or streaming.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use std::fs::File;
    ///     use std::io::prelude::*;
    ///     use std::io::BufReader;
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///
    ///     let f = File::open("sampler_input.json").expect("file not found");
    ///     let mut buf_reader = BufReader::new(f);
    ///     let mut contents = String::new();
    ///     buf_reader.read_to_string(&mut contents)?;
    ///
    ///     let payload = qrmi::models::Payload::QiskitPrimitive {
    ///          input: contents,
    ///          program_id: "sampler".to_string(),
    ///     };
    ///     let job_id = qrmi.task_start(payload).await?;
    ///     println!("Job ID: {}", job_id);
    ///     Ok(())
    /// }
    /// ```
    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::task_start".to_string(),
        ))
    }

    /// Stops the task specified by `task_id`. This function is called if the user cancels the job or if the time limit for job execution is exceeded. The implementation must cancel the task if it is still running.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task to be stopped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     qrmi.task_stop("your_task_id").await?;
    ///     Ok(())
    /// }
    /// ```
    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::task_stop".to_string(),
        ))
    }

    /// Returns the current status of the task specified by `task_id`.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task to be stopped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let status = qrmi.task_status("your_task_id").await?;
    ///     println!("{:?}", status);
    ///     Ok(())
    /// }
    /// ```
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::task_status".to_string(),
        ))
    }

    /// Returns the results of the task.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let job_id = "4EAAA9E2-AD53-4C5C-8EF1-C1A3F219C427";
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let result = qrmi.task_result(&job_id).await?;
    ///     println!("{:?}", result.value);
    ///     Ok(())
    /// }
    /// ```
    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::task_result".to_string(),
        ))
    }

    /// Returns the log messages of the task.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let job_id = "4EAAA9E2-AD53-4C5C-8EF1-C1A3F219C427";
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let log = qrmi.task_logs(&job_id).await?;
    ///     println!("{:?}", log);
    ///     Ok(())
    /// }
    /// ```
    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::task_logs".to_string(),
        ))
    }

    /// Returns a Target for the specified device. Vendor specific serialized data. This might contain the constraints(instructions, properteis and timing information etc.) of a particular device to allow compilers to compile an input circuit to something that works and is optimized for a device. In IBM implementation, it contains JSON representations of [BackendConfiguration](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json) and [BackendProperties](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json) so that we are able to create a Target object by calling `qiskit_ibm_runtime.utils.backend_converter.convert_to_target` or uquivalent functions.
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let target = qrmi.target().await?;
    ///     println!("{:?}", target.value);
    ///     Ok(())
    /// }
    /// ```
    async fn target(&mut self) -> Result<Target> {
        Err(QrmiError::UnsupportedFunction(
            "qrmi::QuantumResource::target".to_string(),
        ))
    }

    /// Returns other specific to system or device data
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQuantumComputeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQuantumComputeService::new("ibm_torino")?;
    ///     let metadata = qrmi.metadata().await;
    ///     println!("{:?}", metadata);
    ///     Ok(())
    /// }
    /// ```
    async fn metadata(&mut self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::<String, String>::new()
    }
}
