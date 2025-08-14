//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Error {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub location: Option<String>,
    pub message: String,
    pub more_info: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub value: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub errors: Vec<Error>,
    pub status_code: i64,
    pub title: String,
    pub trace: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ExtendedErrorResponse {
    Json(ErrorResponse),
    Text(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct IAMErrorResponse {
    #[serde(rename(deserialize = "errorCode"))]
    pub code: String,
    #[serde(rename(deserialize = "errorMessage"))]
    pub message: String,
    #[serde(rename(deserialize = "errorDetails"))]
    pub details: Option<String>,
}
