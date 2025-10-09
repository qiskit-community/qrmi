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

/// Target is a path-like string indicating where the error occurred.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    /// This field MUST contain the name of the problematic field (with dot-syntax if necessary), query parameter, or header.
    pub name: String,

    /// This field MUST contain 'field', 'parameter', or 'header'.
    pub r#type: String,
}

/// A detailed error object which provides granular context (for example, validation failures, missing fields, or service-specific issues).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    /// Error code which can be used in client code. Solutions for various error codes are available here <https://docs.quantum.ibm.com/errors>
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    /// (Deprecated) Location is a path-like string indicating where the error occurred. Prefer using the 'target' field instead. It typically begins with 'path', 'query', 'header', or 'body'. Example: 'body.items\[3\].tags' or 'path.thing-id'.
    pub location: Option<String>,

    /// Message is a human-readable explanation of the error.
    pub message: String,

    /// Link to documentation on how to handle errors.
    pub more_info: String,

    /// Target is a path-like string indicating where the error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub target: Option<Target>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    /// Value is the value at the given location, echoed back to the client to help with debugging. This can be useful for e.g. validating that the client didn't send extra whitespace or help when the client did not log an outgoing request.
    pub value: Option<String>,
}

/// API response returned when failed.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// A list of detailed error objects. Each error entry provides granular context (for example, validation failures, missing fields, or service-specific issues).
    pub errors: Vec<Error>,

    /// The HTTP status code of the error response.
    pub status_code: u16,

    /// A short, human-readable summary of the error. May be omitted when not applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<String>,

    /// A unique identifier used to group request events across distributed systems. Ensures traceability across services
    pub trace: String,

    /// A unique identifier for this error occurrence, typically used for tracking error occurrence in direct-access service logs.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub correlation_id: Option<String>,
}

/// Wrapper of HTTP error responses
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtendedErrorResponse {
    Json(ErrorResponse),
    Text(String),
}

/// IAM API error responses
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IAMErrorResponse {
    #[serde(rename(deserialize = "errorCode"))]
    pub code: String,
    #[serde(rename(deserialize = "errorMessage"))]
    pub message: String,
    #[serde(rename(deserialize = "errorDetails"))]
    pub details: Option<String>,
}
