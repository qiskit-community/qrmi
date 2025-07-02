//
// (C) Copyright IBM 2024
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use serde::{Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all(serialize = "lowercase"))]
/// ID of the primitive to be executed
pub enum ProgramId {
    Estimator,
    Sampler,
}

impl FromStr for ProgramId {
    type Err = ProgramIdParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "estimator" => Ok(ProgramId::Estimator),
            "sampler" => Ok(ProgramId::Sampler),
            _ => Err(ProgramIdParseError),
        }
    }
}

#[derive(Debug)]
pub struct ProgramIdParseError;

impl<'de> Deserialize<'de> for ProgramId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "estimator" => Ok(ProgramId::Estimator),
            "sampler" => Ok(ProgramId::Sampler),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["estimator", "sampler"],
            )),
        }
    }
}

impl fmt::Display for ProgramId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            ProgramId::Estimator => "estimator",
            ProgramId::Sampler => "sampler",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all(serialize = "lowercase"))]
/// Job logging level
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
}
impl FromStr for LogLevel {
    type Err = LogLevelParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(LogLevel::Critical),
            "error" => Ok(LogLevel::Error),
            "warning" => Ok(LogLevel::Warning),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => Err(LogLevelParseError),
        }
    }
}
#[derive(Debug)]
pub struct LogLevelParseError;

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "critical" => Ok(LogLevel::Critical),
            "error" => Ok(LogLevel::Error),
            "warning" => Ok(LogLevel::Warning),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["critical", "error", "warning", "info", "debug"],
            )),
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            LogLevel::Critical => "critical",
            LogLevel::Error => "error",
            LogLevel::Warning => "warning",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
/// Job status
pub enum JobStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl<'de> Deserialize<'de> for JobStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Running" => Ok(JobStatus::Running),
            "Completed" => Ok(JobStatus::Completed),
            "Failed" => Ok(JobStatus::Failed),
            "Cancelled" => Ok(JobStatus::Cancelled),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["Running", "Completed", "Failed", "Cancelled"],
            )),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all(serialize = "lowercase"))]
/// Storage type
pub enum StorageType {
    #[allow(non_camel_case_types)]
    IBMCloud_COS,
    #[allow(non_camel_case_types)]
    S3_Compatible,
}

impl<'de> Deserialize<'de> for StorageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "ibmcloud_cos" => Ok(StorageType::IBMCloud_COS),
            "s3_compatible" => Ok(StorageType::S3_Compatible),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["ibmcloud_cos", "s3_compatible"],
            )),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[allow(dead_code)]
/// Quantum usage
pub struct Usage {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub quantum_nanoseconds: Option<i64>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[allow(dead_code)]
/// Storage option. Currently, only S3 storage is supported by this client.
pub struct StorageOption {
    pub r#type: StorageType,
    pub presigned_url: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)]
/// Storage specification used in [`Job`]
pub struct Storage {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub input: Option<StorageOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub logs: Option<StorageOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub results: Option<StorageOption>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)]
/// Job. Refer Direct Access API specifications for more details.
pub struct Job {
    pub backend: String,
    pub created_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub end_time: Option<String>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub log_level: Option<LogLevel>,
    pub program_id: ProgramId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reason_code: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reason_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub reason_solution: Option<String>,
    pub status: JobStatus,
    pub storage: Storage,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub timeout_secs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
/// A list of [`Job`].
pub struct Jobs {
    pub jobs: Vec<Job>,
}
