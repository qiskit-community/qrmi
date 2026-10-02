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

use crate::common::{resolve_opt, resolve_opt_required};
use crate::error::QrmiError;
use crate::ibm::error::IbmError;
use crate::models::{
    Payload, ResourceCapacity, ResourceStatus, ResourceStatusCode, ResourceType, Target,
    TaskResult, TaskStatus,
};
use crate::{QuantumResource, Result};
use log::{debug, error, info, warn};
use quantum_system_api::utils::s3::S3Client;
use quantum_system_api::{
    models::Backend, models::BackendLanesConfiguration, models::BackendStatus, models::Job,
    models::JobStatus, models::Jobs, models::LogLevel, models::ProgramId, AuthMethod, Client,
    ClientBuilder, QuantumSystemError,
};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::Jitter;
use serde_json::json;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;

/// QRMI implementation for IBM Quantum System API
pub struct IBMQuantumSystem {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
    /// Where the `QRMI_IBM_QS_*` settings are read from. Kept so that
    /// `QRMI_JOB_TIMEOUT_SECONDS` is read when `task_start` runs.
    settings: Settings,
    /// S3 bucket that job results and logs are read from. `None` if S3 was
    /// not configured at construction.
    s3: Option<S3Store>,
}

impl IBMQuantumSystem {
    /// Constructs a QRMI to access IBM Quantum System API Service
    ///
    /// # Environment variables
    ///
    /// * `QRMI_IBM_QS_ENDPOINT`: IBM Quantum System API endpoint URL
    /// * `QRMI_IBM_QS_AWS_ACCESS_KEY_ID`: AWS Access Key ID to access S3 bucket
    /// * `QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY`: AWS Secret Access Key to access S3 bucket
    /// * `QRMI_IBM_QS_S3_ENDPOINT`: S3 API endpoint URL
    /// * `QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI`: S3 API endpoint URL accessed from Quantum System API service. Depending on the network configuration, the IP address used to access S3 may differ between access from the API client and access from the Quantum System API service. In such cases, this environment variable should specify the URL used when accessing S3 from the DA API service.
    /// * `QRMI_IBM_QS_S3_BUCKET`: S3 Bucket name
    /// * `QRMI_IBM_QS_S3_REGION`: S3 Region name
    /// * `QRMI_IBM_QS_IAM_ENDPOINT`: IBM Cloud IAM API endpoint URL
    /// * `QRMI_IBM_QS_IAM_APIKEY`: IBM Cloud API Key
    /// * `QRMI_IBM_QS_SERVICE_CRN`: Provisioned Quantum System API Service instance
    /// * `QRMI_JOB_TIMEOUT_SECONDS`: Time (in seconds) after which job should time out and get cancelled.
    pub fn new(resource_id: &str) -> Result<Self> {
        Self::from_settings(resource_id, Settings::env(resource_id))
    }

    /// Constructs a QRMI to access IBM Quantum System API Service from a
    /// config map, instead of environment variables.
    ///
    /// Accepts the same keys as [`Self::new`]'s environment variables,
    /// minus the `<resource_id>_` prefix. Each key also accepts its fully
    /// lowercased form (e.g. `qrmi_ibm_qs_endpoint`) as a fallback if the
    /// exact-case key isn't present in the map.
    pub fn from_config(resource_id: &str, config: HashMap<String, String>) -> Result<Self> {
        Self::from_settings(resource_id, Settings::map(config))
    }

    /// Shared logic for [`Self::new`] and [`Self::from_config`].
    fn from_settings(resource_id: &str, settings: Settings) -> Result<Self> {
        // Check to see if the settings required to run this program are set.
        let daapi_endpoint = settings.require("QRMI_IBM_QS_ENDPOINT")?;

        let mut builder = ClientBuilder::new(daapi_endpoint);

        let apikey = settings.require("QRMI_IBM_QS_IAM_APIKEY")?;
        let service_crn = settings.require("QRMI_IBM_QS_SERVICE_CRN")?;
        let iam_endpoint_url = settings.require("QRMI_IBM_QS_IAM_ENDPOINT")?;

        let auth_method = AuthMethod::IbmCloudIam {
            apikey,
            service_crn,
            iam_endpoint_url,
        };
        builder.with_auth(auth_method);

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        builder
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy);

        let s3_endpoint_for_daapi = settings.get("QRMI_IBM_QS_S3_ENDPOINT_FOR_QSAPI");

        // S3 is optional: only the task_* methods need it. If it is configured,
        // the API client uses it to upload job input, and our own S3 client
        // uses it to download job results and logs.
        let s3 = match s3_env(&settings) {
            Ok(s3) => {
                builder.with_s3bucket(
                    &s3.access_key_id,
                    &s3.secret_access_key,
                    &s3.endpoint,
                    &s3.bucket,
                    &s3.region,
                    s3_endpoint_for_daapi,
                );
                Some(S3Store {
                    client: S3Client::new(
                        s3.endpoint,
                        s3.access_key_id,
                        s3.secret_access_key,
                        s3.region,
                    ),
                    bucket: s3.bucket,
                })
            }
            Err(e) => {
                info!("No S3 bucket configured ({e}).");
                None
            }
        };

        Ok(Self {
            api_client: builder.build()?,
            backend_name: resource_id.to_string(),
            settings,
            s3,
        })
    }

    /// The S3 bucket, or an error if S3 was not configured at construction.
    ///
    /// The error is the one [`s3_env`] reports for the first missing setting
    /// ([`QrmiError::EnvVarNotSet`] or [`QrmiError::MissingConfigKey`]).
    fn s3_store(&self) -> Result<&S3Store> {
        match &self.s3 {
            Some(s3) => Ok(s3),
            None => Err(s3_env(&self.settings).err().unwrap_or_else(|| {
                // Only reachable if the settings were added after construction.
                QrmiError::InvalidConfig(format!(
                    "S3 bucket was not configured when '{}' was created",
                    self.backend_name
                ))
            })),
        }
    }
}

/// Where the `QRMI_IBM_QS_*` settings are read from: OS environment variables
/// prefixed with `<resource_id>_`, or an unprefixed config map (config maps are
/// already scoped to one resource, so there's nothing to prefix).
struct Settings {
    prefix: String,
    config: Option<HashMap<String, String>>,
}

impl Settings {
    fn env(resource_id: &str) -> Self {
        Self {
            prefix: format!("{resource_id}_"),
            config: None,
        }
    }

    fn map(config: HashMap<String, String>) -> Self {
        Self {
            prefix: String::new(),
            config: Some(config),
        }
    }

    /// The full key looked up for `name`, as it appears in errors.
    fn key(&self, name: &str) -> String {
        format!("{}{name}", self.prefix)
    }

    fn get(&self, name: &str) -> Option<String> {
        resolve_opt(&self.key(name), self.config.as_ref())
    }

    /// Like [`Self::get`], but a missing value is an error
    /// ([`QrmiError::EnvVarNotSet`] or [`QrmiError::MissingConfigKey`]).
    fn require(&self, name: &str) -> Result<String> {
        resolve_opt_required(&self.key(name), self.config.as_ref())
    }
}

/// S3 connection details, read once at construction.
struct S3Env {
    bucket: String,
    endpoint: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

fn s3_env(settings: &Settings) -> Result<S3Env> {
    Ok(S3Env {
        bucket: settings.require("QRMI_IBM_QS_S3_BUCKET")?,
        endpoint: settings.require("QRMI_IBM_QS_S3_ENDPOINT")?,
        access_key_id: settings.require("QRMI_IBM_QS_AWS_ACCESS_KEY_ID")?,
        secret_access_key: settings.require("QRMI_IBM_QS_AWS_SECRET_ACCESS_KEY")?,
        region: settings.require("QRMI_IBM_QS_S3_REGION")?,
    })
}

/// S3 client and bucket that job results and logs are read from.
struct S3Store {
    client: S3Client,
    bucket: String,
}

impl IBMQuantumSystem {
    /// Records the final state of finished jobs owned by this user on this
    /// backend to S3 and deletes them from the Quantum System API service.
    ///
    /// A job is owned by this user if its input object (`input_<job_id>.json`)
    /// exists in the S3 bucket of this resource and its `qrmi:uid` tag is the
    /// real user ID of this process. For each owned job whose status is not
    /// `Running`:
    ///
    /// 1. Adds the following tags to the input object in S3:
    ///    * `qrmi:status`: job status (`Completed`, `Failed` or `Cancelled`)
    ///    * `qrmi:status:created`: `metrics.timestamps.created` of the job
    ///    * `qrmi:status:finished`: `metrics.timestamps.finished` of the job
    ///      (omitted if not available)
    ///    * `qrmi:status:circuits_execution_time_ns`:
    ///      `metrics.circuits_execution_time_ns` of the job (omitted if not available)
    /// 2. Deletes the job.
    ///
    /// Other jobs are skipped (neither tagged nor deleted), so that jobs of
    /// other users or clients are not deleted without recording their status:
    ///
    /// * the input object does not exist in the bucket (logged at debug level)
    /// * the `qrmi:uid` tag is missing or different (logged at debug level)
    /// * the tags cannot be read (logged at error level)
    ///
    /// No job is deleted if S3 is not configured.
    ///
    /// Errors are written to the log with `log::error!` and do not stop the
    /// processing of other jobs. A failure to tag the input object does not
    /// prevent the job from being deleted.
    ///
    /// A job is regarded as finished based on [`effective_status`], i.e. a
    /// `Completed` job is not deleted until `metrics.circuits_execution_time_ns`
    /// becomes available.
    async fn delete_completed_jobs(&self) {
        self.delete_completed_jobs_impl(None).await
    }

    /// Same as [`Self::delete_completed_jobs`], except that the job `stopped_job_id`
    /// is regarded as finished based on the status reported by the API, without
    /// waiting for `metrics.circuits_execution_time_ns`. Used by `task_stop`.
    async fn delete_completed_jobs_impl(&self, stopped_job_id: Option<&str>) {
        let jobs = match self.api_client.list_jobs::<Jobs>().await {
            Ok(jobs) => jobs,
            Err(err) => {
                error!("failed to list jobs: {err}");
                return;
            }
        };

        let s3 = match self.s3_store() {
            Ok(s3) => s3,
            Err(err) => {
                error!("S3 is not configured. completed jobs are not deleted: {err}");
                return;
            }
        };
        let (s3_client, bucket) = (&s3.client, &s3.bucket);

        // SAFETY: getuid() is always successful and has no side effects.
        let my_uid = unsafe { libc::getuid() }.to_string();

        for job in jobs
            .jobs
            .iter()
            .filter(|job| job.backend == self.backend_name)
        {
            let status = if stopped_job_id == Some(job.id.as_str()) {
                job.status.clone()
            } else {
                effective_status(job)
            };
            if matches!(status, JobStatus::Running) {
                continue;
            }

            let key = format!("input_{}.json", job.id);
            match s3_client.try_get_object_tags(bucket, &key).await {
                Ok(Some(current)) => {
                    if current.get("qrmi:uid") != Some(&my_uid) {
                        debug!(
                            "job {} is not owned by this user (qrmi:uid={:?}, uid={my_uid}). skipped.",
                            job.id,
                            current.get("qrmi:uid")
                        );
                        continue;
                    }
                }
                Ok(None) => {
                    debug!(
                        "{key} is not found in bucket {bucket}. job {} is not owned by this user. skipped.",
                        job.id
                    );
                    continue;
                }
                Err(err) => {
                    error!(
                        "failed to read tags of {key} (job {}). skipped: {err}",
                        job.id
                    );
                    continue;
                }
            }

            let mut tags = HashMap::from([("qrmi:status".to_string(), status.to_string())]);
            if let Some(metrics) = &job.metrics {
                if let Some(ns) = metrics.circuits_execution_time_ns {
                    tags.insert(
                        "qrmi:status:circuits_execution_time_ns".to_string(),
                        ns.to_string(),
                    );
                }
                tags.insert(
                    "qrmi:status:created".to_string(),
                    metrics.timestamps.created.clone(),
                );
                if let Some(finished) = &metrics.timestamps.finished {
                    tags.insert("qrmi:status:finished".to_string(), finished.clone());
                }
            }
            if let Err(err) = s3_client.add_object_tags(bucket, &key, &tags).await {
                error!("failed to add tags to {key} (job {}): {err}", job.id);
            }

            if let Err(err) = self.api_client.delete_job(&job.id).await {
                error!("failed to delete job {}: {err}", job.id);
            }
        }
    }

    /// Reads the final status of a job which has already been deleted by
    /// [`Self::delete_completed_jobs`] from the `qrmi:status` tag of its input
    /// object (`input_<job_id>.json`) in S3.
    async fn job_status_from_s3(&self, task_id: &str) -> Result<JobStatus> {
        let s3 = self.s3_store()?;
        let key = format!("input_{task_id}.json");
        let tags = s3.client.get_object_tags(&s3.bucket, &key).await?;
        match tags.get("qrmi:status").map(String::as_str) {
            Some("Running") => Ok(JobStatus::Running),
            Some("Completed") => Ok(JobStatus::Completed),
            Some("Failed") => Ok(JobStatus::Failed),
            Some("Cancelled") => Ok(JobStatus::Cancelled),
            Some(other) => Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: format!("unknown qrmi:status tag value on {key}: {other}"),
            }),
            None => Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: format!("qrmi:status tag is not found on {key}"),
            }),
        }
    }

    /// Returns the status of the job, based on [`effective_status`]. If the job is not found in the Quantum
    /// System API service (i.e. it has been deleted by
    /// [`Self::delete_completed_jobs`]), the status recorded in S3 is used
    /// instead. If that also fails, the original "job not found" error is returned.
    async fn job_status(&self, task_id: &str) -> Result<JobStatus> {
        match self.api_client.get_job::<Job>(task_id).await {
            Ok(job) => Ok(effective_status(&job)),
            Err(err @ QuantumSystemError::JobNotFound(_)) => {
                self.job_status_from_s3(task_id).await.map_err(|s3_err| {
                    error!("failed to read status of job {task_id} from S3: {s3_err}");
                    QrmiError::from(err)
                })
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Body of [`QuantumResource::task_result`], without cleanup.
    async fn read_task_result(&self, task_id: &str) -> Result<TaskResult> {
        let s3 = self.s3_store()?;

        let (status, failure_reason) = match self.api_client.get_job::<Job>(task_id).await {
            Ok(job) => {
                let status = effective_status(&job);
                let reason_code = job.reason_code.map_or("".to_string(), |v| v.to_string());
                let reason_message = job.reason_message.unwrap_or("".to_string());
                let reason_solution = job.reason_solution.unwrap_or("".to_string());
                (
                    status,
                    format!(
                        "task failed. code: {reason_code}, message: {reason_message}, solution: {reason_solution}"
                    ),
                )
            }
            // Deleted by delete_completed_jobs(). Use the status recorded in S3.
            // The failure details are not available in this case.
            Err(err @ QuantumSystemError::JobNotFound(_)) => (
                self.job_status_from_s3(task_id).await.map_err(|s3_err| {
                    error!("failed to read status of job {task_id} from S3: {s3_err}");
                    QrmiError::from(err)
                })?,
                "task failed. (details are not available since the job has been deleted)"
                    .to_string(),
            ),
            Err(err) => return Err(err.into()),
        };
        if matches!(status, JobStatus::Failed) {
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: failure_reason,
            });
        }
        if matches!(status, JobStatus::Cancelled) {
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: "task was cancelled".to_string(),
            });
        }
        if matches!(status, JobStatus::Running) {
            return Err(QrmiError::TaskNotReady {
                task_id: task_id.to_string(),
                reason: "task is running".to_string(),
            });
        }
        let s3_object_key = format!("results_{}.json", task_id);
        let object = s3.client.get_object(&s3.bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(TaskResult {
            value: retrieved_txt,
        })
    }

    /// Body of [`QuantumResource::task_logs`], without cleanup.
    async fn read_task_logs(&self, task_id: &str) -> Result<String> {
        let s3 = self.s3_store()?;

        let s3_object_key = format!("logs_{}.json", task_id);
        let object = s3.client.get_object(&s3.bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(retrieved_txt)
    }
}

/// How long to wait for `metrics.circuits_execution_time_ns` after a job is
/// reported as `Completed`. See [`effective_status`].
const CIRCUITS_EXECUTION_TIME_WAIT_TIMEOUT: Duration = Duration::from_secs(60 * 60);

/// Returns the status of the job as seen by QRMI.
///
/// The Quantum System API service reports `Completed` before the job has
/// actually finished; the job has really finished only when
/// `metrics.circuits_execution_time_ns` becomes available. Until then, a
/// `Completed` job is treated as `Running`. Other statuses are returned as is.
///
/// If `metrics.circuits_execution_time_ns` is still not available after
/// [`CIRCUITS_EXECUTION_TIME_WAIT_TIMEOUT`] since the job finished
/// (`metrics.timestamps.finished`, or `metrics.timestamps.created` if not
/// available), the job is regarded as `Completed` with a warning, so that it
/// does not stay `Running` forever. The same applies if the timestamp cannot
/// be parsed.
fn effective_status(job: &Job) -> JobStatus {
    if !matches!(job.status, JobStatus::Completed) {
        return job.status.clone();
    }
    let Some(metrics) = &job.metrics else {
        // No timestamps to measure the timeout from.
        warn!(
            "job {} is Completed but has no metrics. regarded as Completed.",
            job.id
        );
        return JobStatus::Completed;
    };
    if metrics.circuits_execution_time_ns.is_some() {
        return JobStatus::Completed;
    }

    let since = metrics
        .timestamps
        .finished
        .as_deref()
        .unwrap_or(&metrics.timestamps.created);
    let elapsed = chrono::DateTime::parse_from_rfc3339(since)
        .ok()
        .and_then(|t| {
            (chrono::Utc::now() - t.with_timezone(&chrono::Utc))
                .to_std()
                .ok()
        });
    match elapsed {
        Some(elapsed) if elapsed < CIRCUITS_EXECUTION_TIME_WAIT_TIMEOUT => JobStatus::Running,
        Some(_) => {
            warn!(
                "metrics.circuits_execution_time_ns of job {} is not available {} seconds after {since}. regarded as Completed.",
                job.id,
                CIRCUITS_EXECUTION_TIME_WAIT_TIMEOUT.as_secs()
            );
            JobStatus::Completed
        }
        None => {
            warn!(
                "failed to evaluate timestamp {since:?} of job {}. regarded as Completed.",
                job.id
            );
            JobStatus::Completed
        }
    }
}

/// Builds the S3 object tags attached to the input object (`input_<id>.json`)
/// uploaded by [`IBMQuantumSystem::task_start`].
///
/// * `qrmi:jid`: value of the `QRMI_JOB_ID` environment variable. Omitted if not set.
/// * `qrmi:uid`: real user ID of this process (`getuid()`).
/// * `qrmi:gid`: real group ID of this process (`getgid()`).
/// * `qrmi:program_type`: program ID of the primitive (e.g. `sampler`, `estimator`).
fn input_object_tags(program_id: &ProgramId) -> HashMap<String, String> {
    let mut tags = HashMap::new();
    match std::env::var("QRMI_JOB_ID") {
        Ok(jid) => {
            tags.insert("qrmi:jid".to_string(), jid);
        }
        Err(_) => debug!("QRMI_JOB_ID is not set. qrmi:jid tag is omitted."),
    }
    // SAFETY: getuid() and getgid() are always successful and have no side effects.
    let (uid, gid) = unsafe { (libc::getuid(), libc::getgid()) };
    tags.insert("qrmi:uid".to_string(), uid.to_string());
    tags.insert("qrmi:gid".to_string(), gid.to_string());
    tags.insert("qrmi:program_type".to_string(), program_id.to_string());
    tags
}

#[async_trait]
impl QuantumResource for IBMQuantumSystem {
    async fn resource_id(&mut self) -> Result<String> {
        Ok(self.backend_name.clone())
    }

    async fn resource_type(&mut self) -> Result<ResourceType> {
        Ok(ResourceType::IBMQuantumSystem)
    }

    async fn is_accessible(&mut self) -> Result<bool> {
        let backend = self
            .api_client
            .get_backend::<Backend>(&self.backend_name)
            .await?;
        Ok(matches!(backend.status, BackendStatus::Online))
    }

    async fn status(&mut self) -> Result<ResourceStatus> {
        let (backend, lane_config, jobs) = tokio::try_join!(
            self.api_client.get_backend::<Backend>(&self.backend_name),
            self.api_client
                .get_backend_lanes_configuration::<BackendLanesConfiguration>(&self.backend_name),
            self.api_client.list_jobs::<Jobs>()
        )?;

        // `list_jobs` also returns finished jobs; only running ones occupy a lane.
        let count = jobs
            .jobs
            .iter()
            .filter(|job| job.backend == self.backend_name)
            .filter(|job| matches!(job.status, JobStatus::Running))
            .count() as u64;

        let status = match backend.status {
            BackendStatus::Online => ResourceStatusCode::Online,
            BackendStatus::Offline => ResourceStatusCode::Offline,
            BackendStatus::Paused => ResourceStatusCode::Paused,
        };

        Ok(ResourceStatus {
            status,
            status_reason: None,
            busy: backend.locked,
            healthy: None,
            capacity: Some(ResourceCapacity {
                available_slots: lane_config.hpc_workload_manager.lanes.saturating_sub(count),
                max_slots: lane_config.hpc_workload_manager.lanes,
            }),
            pending_job_count: None,
        })
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        self.delete_completed_jobs().await;

        let timeout = self.settings.require("QRMI_JOB_TIMEOUT_SECONDS")?;
        let timeout_secs = timeout
            .parse::<u64>()
            .map_err(|source| QrmiError::ParseError {
                name: self.settings.key("QRMI_JOB_TIMEOUT_SECONDS"),
                value: timeout,
                source: Box::new(source),
            })?;

        let Payload::QiskitPrimitive { input, program_id } = payload else {
            return Err(QrmiError::UnsupportedPayload(format!("{payload:?}")));
        };

        let job_input: serde_json::Value = serde_json::from_str(input.as_str())?;
        let program_id_enum = ProgramId::from_str(&program_id)
            .map_err(|_| IbmError::UnknownProgramId(program_id.clone()))?;

        let tags = input_object_tags(&program_id_enum);

        let job = self
            .api_client
            .run_primitive(
                &self.backend_name,
                program_id_enum,
                timeout_secs,
                LogLevel::Debug,
                &job_input,
                None,
                Some(&tags),
            )
            .await?;
        Ok(job.job_id)
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let status = match self.api_client.get_job_status(task_id).await {
            Ok(status) => status,
            // Already finished and deleted by delete_completed_jobs(). Nothing to stop.
            Err(QuantumSystemError::JobNotFound(_))
                if self.job_status_from_s3(task_id).await.is_ok() =>
            {
                return Ok(());
            }
            Err(err) => return Err(err.into()),
        };
        if matches!(status, JobStatus::Running) {
            let _ = self.api_client.cancel_job(task_id, false).await;
        }
        // Cancellation is synchronous, so the job is no longer `Running` here.
        // Record its final status to S3 and delete it (along with other
        // finished jobs of this user) without waiting for
        // `metrics.circuits_execution_time_ns`.
        self.delete_completed_jobs_impl(Some(task_id)).await;
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let result = self.job_status(task_id).await.map(|status| match status {
            JobStatus::Running => TaskStatus::Running,
            JobStatus::Completed => TaskStatus::Completed,
            JobStatus::Cancelled => TaskStatus::Cancelled,
            JobStatus::Failed => TaskStatus::Failed,
        });
        // Clean up finished jobs after the status has been read, so that the
        // final status of this job is returned (and recorded to S3) first.
        self.delete_completed_jobs().await;
        result
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let result = self.read_task_result(task_id).await;
        // Clean up finished jobs after the result has been read.
        self.delete_completed_jobs().await;
        result
    }

    async fn task_logs(&mut self, task_id: &str) -> Result<String> {
        let result = self.read_task_logs(task_id).await;
        // Clean up finished jobs after the logs have been read.
        self.delete_completed_jobs().await;
        result
    }

    async fn target(&mut self) -> Result<Target> {
        let mut resp = json!({});
        if let Ok(config) = self
            .api_client
            .get_backend_configuration::<serde_json::Value>(&self.backend_name)
            .await
        {
            resp["configuration"] = config;
        } else {
            resp["configuration"] = json!(null);
        }

        if let Ok(props) = self
            .api_client
            .get_backend_properties::<serde_json::Value>(&self.backend_name)
            .await
        {
            resp["properties"] = props;
        } else {
            resp["properties"] = json!(null);
        }

        Ok(Target {
            value: resp.to_string(),
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}

#[cfg(test)]
#[path = "tests/quantum_system.rs"]
mod tests;
