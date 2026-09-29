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

//! High-level entry point for discovering the QPU resources assigned to a job.

use crate::models::{Config, ResourceType};
use crate::QuantumResource;
use crate::Result;
use std::collections::HashMap;

/// Discovers the QPU resources assigned to the current job -- read from the
/// `QRMI_JOB_QPU_RESOURCES` / `QRMI_JOB_QPU_TYPES` environment variables, or
/// their legacy `SLURM_JOB_QPU_RESOURCES` / `SLURM_JOB_QPU_TYPES`
/// equivalents -- and exposes the ones that are currently accessible as
/// [`QuantumResource`] instances.
///
/// This mirrors [`crate::resource_provider::ResourceProvider`] in spirit,
/// except that the set of resources comes from the job's environment rather
/// than from querying a vendor endpoint, and the result is discovered once
/// and cached (repeated calls to [`resource`](QRMIService::resource) return
/// the *same* instance) rather than re-fetched on every call.
///
/// # Example
///
/// ```no_run
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     use qrmi::QRMIService;
///
///     let mut service = QRMIService::new().await?;
///     for resource in service.resources() {
///         println!("{}", resource.resource_id().await?);
///     }
///
///     if let Some(resource) = service.resource("ibm_torino") {
///         let token = resource.acquire().await?;
///         println!("acquisition token = {}", token);
///     }
///     Ok(())
/// }
/// ```
pub struct QRMIService {
    resources: HashMap<String, Box<dyn QuantumResource + Send + Sync>>,
}

impl QRMIService {
    /// Discovers and constructs the accessible QRMI resources for the
    /// current job.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `QRMI_PLUGIN_ERROR` is set (the QRMI plugin recorded a resource
    ///   acquisition failure there).
    /// - The job's QPU resource/type environment variables are not set, or
    ///   specify inconsistent numbers of resources and types.
    /// - Constructing or querying the accessibility of one of the specified
    ///   resources fails.
    pub async fn new() -> Result<Self> {
        crate::common::initialize();

        let (qpus, qpu_types) = crate::common::get_job_qpu_resources_and_types()?;
        log::debug!("qpus: {:?}", qpus);
        log::debug!("qpu types: {:?}", qpu_types);

        let mut candidates = Vec::new();
        for (qpu, qpu_type) in qpus.iter().zip(qpu_types.iter()) {
            let qpu = qpu.trim();
            let Some(resource_type) = ResourceType::from_qpu_type_str(qpu_type) else {
                log::warn!(
                    "Unsupported resource type: {} specified for {}",
                    qpu_type,
                    qpu
                );
                continue;
            };
            candidates.push(crate::common::create_resource(&resource_type, qpu)?);
        }
        Self::from_candidates(candidates).await
    }

    /// Constructs the accessible QRMI resources defined in a QRMI
    /// configuration file, without reading any environment variables.
    ///
    /// This is the counterpart of [`new`](Self::new) for applications that
    /// don't run under a workload manager: each static resource definition
    /// is built with its type's `from_config()`, using the definition's
    /// `environment` map merged with `runtime_config`. `runtime_config`
    /// holds values the configuration file can't know in advance, e.g.
    /// `QRMI_JOB_ID`, `QRMI_JOB_UID` or `QRMI_JOB_TIMEOUT_SECONDS`, and takes
    /// precedence over the definition's values.
    ///
    /// Dynamic resource definitions (`is_dynamic: true`) are skipped; use
    /// [`crate::create_provider`] to discover their backends.
    ///
    /// # Errors
    ///
    /// Returns an error if constructing one of the defined resources fails,
    /// e.g. because a required key is missing. Resources whose status can't
    /// be read are skipped.
    pub async fn from_config(
        config: &Config,
        runtime_config: &HashMap<String, String>,
    ) -> Result<Self> {
        crate::common::initialize();

        let mut candidates = Vec::new();
        for def in config.resource_map.values() {
            if def.is_dynamic() {
                log::debug!("{} is a dynamic resource definition. ignored.", def.name);
                continue;
            }
            let mut resource_config = def.environment.clone();
            resource_config.extend(runtime_config.clone());
            candidates.push(crate::common::create_resource_from_config(
                &def.r#type,
                &def.name,
                resource_config,
            )?);
        }
        Self::from_candidates(candidates).await
    }

    /// Keeps the candidates that are currently online. A candidate whose
    /// status can't be read is treated as not accessible, so that one
    /// unreachable resource doesn't hide the others.
    async fn from_candidates(
        candidates: Vec<Box<dyn QuantumResource + Send + Sync>>,
    ) -> Result<Self> {
        let mut resources: HashMap<String, Box<dyn QuantumResource + Send + Sync>> = HashMap::new();
        for mut resource in candidates {
            let resource_id = resource.resource_id().await?;
            match resource.status().await {
                Ok(res_status)
                    if matches!(res_status.status, crate::models::ResourceStatusCode::Online) =>
                {
                    resources.insert(resource_id, resource);
                }
                Ok(_) => log::debug!("{} is not accessible now. ignored.", resource_id),
                Err(e) => log::warn!("{}: failed to get status ({}). ignored.", resource_id, e),
            }
        }

        Ok(Self { resources })
    }

    /// Returns all accessible QRMI resources.
    pub fn resources(&mut self) -> Vec<&mut (dyn QuantumResource + Send + Sync + 'static)> {
        self.resources.values_mut().map(|r| r.as_mut()).collect()
    }

    /// Returns a single resource matching the specified resource identifier,
    /// i.e. backend name for IBM Quantum, or `None` if not found.
    pub fn resource(
        &mut self,
        resource_id: &str,
    ) -> Option<&mut (dyn QuantumResource + Send + Sync + 'static)> {
        self.resources.get_mut(resource_id).map(|r| r.as_mut())
    }

    /// Consumes this service, yielding ownership of its accessible
    /// resources.
    ///
    /// Not part of the public API: callers who just want to use the
    /// resources should use [`resources`](Self::resources) or
    /// [`resource`](Self::resource) instead, which borrow rather than take
    /// ownership. This exists for the C and Python bindings
    /// (`cext::qrmi_service_resources`, `pyext::PyQRMIService`), which each
    /// need to move every resource into its own, independently owned
    /// handle/object -- mirroring how `cext::qrmi_provider_resources` and
    /// `pyext::PyResourceProvider::resources` each wrap the
    /// `Box<dyn QuantumResource>`s returned by `ResourceProvider::resources`.
    pub(crate) fn into_resource_map(
        self,
    ) -> HashMap<String, Box<dyn QuantumResource + Send + Sync>> {
        self.resources
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Serves `GET /accessible` for a Pasqal Local resource, answering every
    /// connection with the same accessible response.
    fn mock_warden() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = r#"{"is_accessible":true,"message":""}"#;
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
            }
        });
        url
    }

    #[tokio::test]
    async fn from_config_builds_static_resources_without_env() {
        let url = mock_warden();
        let path = std::env::temp_dir().join(format!("qrmi_service_{}.json", std::process::id()));
        std::fs::write(
            &path,
            serde_json::json!({
                "resources": [
                    {
                        "name": "PASQAL_LOCAL",
                        "type": "pasqal-local",
                        "environment": {"QRMI_WARDEN_URL": url}
                    },
                    {
                        "name": "PASQAL_UNREACHABLE",
                        "type": "pasqal-local",
                        "environment": {"QRMI_WARDEN_URL": "http://127.0.0.1:1"}
                    },
                    {
                        "name": "ibm_dynamic",
                        "type": "ibm-quantum-system",
                        "is_dynamic": true,
                        "environment": {}
                    }
                ]
            })
            .to_string(),
        )
        .unwrap();
        let config = Config::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).unwrap();

        // QRMI_JOB_ID/QRMI_JOB_UID are required by Pasqal Local and only
        // supplied at runtime, not by the config file.
        let runtime_config = HashMap::from([
            ("QRMI_JOB_ID".to_string(), "1".to_string()),
            ("QRMI_JOB_UID".to_string(), "1000".to_string()),
        ]);
        let mut service = QRMIService::from_config(&config, &runtime_config)
            .await
            .unwrap();

        let ids: Vec<String> = service.resources_ids();
        assert_eq!(ids, vec!["PASQAL_LOCAL".to_string()]);
        assert!(std::env::var("PASQAL_LOCAL_QRMI_WARDEN_URL").is_err());
        assert!(std::env::var("QRMI_JOB_ID").is_err());
    }

    #[tokio::test]
    async fn from_config_reports_missing_runtime_values() {
        let path =
            std::env::temp_dir().join(format!("qrmi_service_missing_{}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"{"resources":[{"name":"PASQAL_LOCAL","type":"pasqal-local","environment":{"QRMI_WARDEN_URL":"http://127.0.0.1:1"}}]}"#,
        )
        .unwrap();
        let config = Config::load(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).unwrap();

        let err = QRMIService::from_config(&config, &HashMap::new())
            .await
            .err()
            .expect("QRMI_JOB_UID is not configured");
        assert_eq!(err.kind(), crate::QrmiErrorKind::MissingConfigKey);
    }

    impl QRMIService {
        fn resources_ids(&mut self) -> Vec<String> {
            self.resources.keys().cloned().collect()
        }
    }
}
