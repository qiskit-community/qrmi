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

pub(crate) mod common;
pub(crate) mod consts;
pub mod ibm;
pub mod pasqal;

mod cext;
pub mod models;
#[cfg(feature = "pyo3")]
pub mod pyext;

use crate::models::{Payload, ResourceType, Target, TaskResult, TaskStatus};
use anyhow::Result;
use async_trait::async_trait;

/// Defines interfaces to quantum resources.
#[async_trait]
pub trait QuantumResource: Send + Sync {
    async fn resource_id(&mut self) -> Result<String>;
    /// Returns resource identifier of this quantum resource.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let resource_id = qrmi.resource_id().await?;
    ///     println!("{resource_id}"); // prints "ibm_torino"
    ///     Ok(())
    /// }
    /// ```
    async fn resource_type(&mut self) -> Result<ResourceType>;
    ///
    /// Returns resource type of this quantum resource.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let resource_type = qrmi.resource_type().await?;
    ///     println!("{}", resource_type.as_str()); // prints "qiskit_runtime_service"
    ///     Ok(())
    /// }
    /// ```
    async fn is_accessible(&mut self) -> Result<bool>;

    /// Acquires quantum resource and returns acquisition token if succeeded. If no one owns the lock, it acquires the lock and returns immediately. If another owns the lock, block until we are able to acquire lock.
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let token = qrmi.acquire().await?;
    ///     println!("acquisition token = {}", token);
    ///     Ok(())
    /// }
    /// ```
    async fn acquire(&mut self) -> Result<String>;

    /// Releases quantum resource
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     qrmi.release("your_acquisition_token").await?;
    ///     Ok(())
    /// }
    /// ```
    async fn release(&mut self, id: &str) -> Result<()>;

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
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
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
    async fn task_start(&mut self, payload: Payload) -> Result<String>;

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
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     qrmi.task_stop("your_task_id").await?;
    ///     Ok(())
    /// }
    /// ```
    async fn task_stop(&mut self, task_id: &str) -> Result<()>;

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
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let status = qrmi.task_status("your_task_id").await?;
    ///     println!("{:?}", status);
    ///     Ok(())
    /// }
    /// ```
    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus>;

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
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let job_id = "4EAAA9E2-AD53-4C5C-8EF1-C1A3F219C427";
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let result = qrmi.task_result(&job_id).await?;
    ///     println!("{:?}", result.value);
    ///     Ok(())
    /// }
    /// ```
    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult>;

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
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let job_id = "4EAAA9E2-AD53-4C5C-8EF1-C1A3F219C427";
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let log = qrmi.task_logs(&job_id).await?;
    ///     println!("{:?}", log);
    ///     Ok(())
    /// }
    /// ```
    async fn task_logs(&mut self, task_id: &str) -> Result<String>;

    /// Returns a Target for the specified device. Vendor specific serialized data. This might contain the constraints(instructions, properteis and timing information etc.) of a particular device to allow compilers to compile an input circuit to something that works and is optimized for a device. In IBM implementation, it contains JSON representations of [BackendConfiguration](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json) and [BackendProperties](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json) so that we are able to create a Target object by calling `qiskit_ibm_runtime.utils.backend_converter.convert_to_target` or uquivalent functions.
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let target = qrmi.target().await?;
    ///     println!("{:?}", target.value);
    ///     Ok(())
    /// }
    /// ```
    async fn target(&mut self) -> Result<Target>;

    /// Returns other specific to system or device data
    ///
    /// # Example
    ///
    /// ```no_run
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{ibm::IBMQiskitRuntimeService, QuantumResource};
    ///
    ///     let mut qrmi = IBMQiskitRuntimeService::new("ibm_torino")?;
    ///     let metadata = qrmi.metadata().await;
    ///     println!("{:?}", metadata);
    ///     Ok(())
    /// }
    /// ```
    async fn metadata(&mut self) -> std::collections::HashMap<String, String>;
}
