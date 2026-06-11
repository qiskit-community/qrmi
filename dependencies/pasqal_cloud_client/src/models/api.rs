//
// (C) Copyright Pasqal 2025, 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::models::batch::JobStatus;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("auth token is missing or expired and refresh credentials are not configured")]
    MissingCredentialsForRefresh,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthTokenResponse {
    pub access_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceResponseData {
    pub status: String,
    pub availability: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetDeviceSpecsResponseData {
    pub device_type: String,
    pub specs: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBatchResponseData {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCudaqJobResponseData {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBatchResponseData {
    pub status: JobStatus,
    pub job_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetCudaqJobResponseData {
    pub status: String,
    #[serde(default)]
    pub result: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetJobResponseData {
    pub status: JobStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelBatchResponseData {}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub runs: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct JobResult {
    pub(crate) counter: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Batch {
    pub sequence_builder: String,
    pub jobs: Vec<Job>,
    pub device_type: String,
    pub project_id: String,
}
