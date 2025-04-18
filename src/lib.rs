// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

pub mod common;
pub mod consts;
pub mod ibm;
pub mod models;

use crate::models::{Payload, Target, TaskResult, TaskStatus};
use anyhow::Result;

use pyo3::prelude::*;

/// Defines interfaces to quantum resources.
pub trait QuantumResource {
    /// Returns true if device is accessible, otherwise false.
    ///
    /// # Arguments
    ///
    /// * `id`: Identifier of quantum device.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut qrmi = qrmi::QiskitRuntimeService::default();
    ///
    ///     let device: &str = "ibm_torino";
    ///     let accessible = qrmi.is_accessible(device);
    ///     if accessible == false {
    ///         panic!("{} is not accessible.", device);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn is_accessible(&mut self, id: &str) -> bool;

    /// Acquires quantum resource and returns acquisition token if succeeded. If no one owns the lock, it acquires the lock and returns immediately. If another owns the lock, block until we are able to acquire lock.
    ///
    /// # Arguments
    ///
    /// * `id`: Identifier of quantum device.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut qrmi = qrmi::QiskitRuntimeService::default();
    ///     let token = qrmi.acquire(device).unwrap();
    ///     println!("acquisition token = {}", token);
    ///     Ok(())
    /// }
    /// ```
    fn acquire(&mut self, id: &str) -> Result<String>;

    /// Releases quantum resource
    ///
    /// # Arguments
    ///
    /// * `id`: acquisition token obtained by previous [`acquire()`](crate::QuantumResource::acquire) call.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut qrmi = qrmi::QiskitRuntimeService::default();
    ///     qrmi.release("your_acquisition_token").await?;
    ///     Ok(())
    /// }
    /// ```
    fn release(&mut self, id: &str) -> Result<()>;

    /// Start a task and returns an identifier of this task if succeeded.
    ///
    /// # Arguments
    ///
    /// * `payload`: payload for task execution. This might be serialized data or streaming.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use std::fs::File;
    ///     use std::io::prelude::*;
    ///     use std::io::BufReader;
    ///
    ///     let mut qrmi = qrmi::QiskitRuntimeService::default();
    ///
    ///     let f = File::open("sampler_input.json").expect("file not found");
    ///     let mut buf_reader = BufReader::new(f);
    ///     let mut contents = String::new();
    ///     buf_reader.read_to_string(&mut contents)?;
    ///
    ///     let payload = qrmi::models::Payload::QiskitPrimitive {
    ///          input: contents,
    ///          program_id: args.program_id,
    ///     };
    ///     let job_id = qrmi.task_start(payload).unwrap();
    ///     println!("Job ID: {}", job_id);
    ///     Ok(())
    /// }
    /// ```
    fn task_start(&mut self, payload: Payload) -> Result<String>;

    /// Stops the task specified by `task_id`. This function is called if the user cancels the job or if the time limit for job execution is exceeded. The implementation must cancel the task if it is still running.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task to be stopped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut qrmi = qrmi::QiskitRuntimeService::default();
    ///     qrmi.task_stop("your_task_id").unwrap();
    ///     Ok(())
    /// }
    /// ```
    fn task_stop(&mut self, task_id: &str) -> Result<()>;

    /// Returns the current status of the task specified by `task_id`.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task to be stopped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{QiskitRuntimeService};
    ///
    ///     let mut qrmi = QiskitRuntimeService::default();
    ///     let status = qrmi.task_status("your_task_id").unwrap();
    ///     println!("{:?}", status);
    ///     Ok(())
    /// }
    /// ```
    fn task_status(&mut self, task_id: &str) -> Result<TaskStatus>;

    /// Returns the results of the task.
    ///
    /// # Arguments
    ///
    /// * `task_id`: Identifier of the task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{QiskitRuntimeService};
    ///
    ///     let mut qrmi = QiskitRuntimeService::default();
    ///     let result = qrmi.task_result(&job_id).unwrap();
    ///     println!("{:?}", result.value);
    ///     Ok(())
    /// }
    /// ```
    fn task_result(&mut self, task_id: &str) -> Result<TaskResult>;

    /// Returns a Target for the specified device. Vendor specific serialized data. This might contain the constraints(instructions, properteis and timing information etc.) of a particular device to allow compilers to compile an input circuit to something that works and is optimized for a device. In IBM implementation, it contains JSON representations of [BackendConfiguration](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json) and [BackendProperties](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json) so that we are able to create a Target object by calling `qiskit_ibm_runtime.utils.backend_converter.convert_to_target` or uquivalent functions.
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{QiskitRuntimeService};
    ///
    ///     let mut qrmi = QiskitRuntimeService::default();
    ///     let target = qrmi.target("ibm_torino").unwrap();
    ///     println!("{:?}", target.value);
    ///     Ok(())
    /// }
    /// ```
    fn target(&mut self, id: &str) -> Result<Target>;

    /// Returns other specific to system or device data
    ///
    /// # Example
    ///
    /// ```no_run
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     use qrmi::{QiskitRuntimeService};
    ///
    ///     let mut qrmi = QiskitRuntimeService::default();
    ///     let metadata = qrmi.metadata();
    ///     println!("{:?}", metadata);
    ///     Ok(())
    /// }
    /// ```
    fn metadata(&mut self) -> std::collections::HashMap<String, String>;
}

/// A Python module implemented in Rust.
#[pymodule]
fn qrmi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<crate::ibm::IBMDirectAccess>()?;
    m.add_class::<crate::ibm::IBMQiskitRuntimeService>()?;
    m.add_class::<crate::models::TaskStatus>()?;
    m.add_class::<crate::models::Payload>()?;
    m.add_class::<crate::models::TaskResult>()?;
    Ok(())
}
