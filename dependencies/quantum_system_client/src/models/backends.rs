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

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all(serialize = "lowercase"))]
/// Status of the backend
pub enum BackendStatus {
    /// online (you can send jobs)
    Online,
    /// offline (you cannot send jobs)
    Offline,
    /// paused (you cannot send jobs)
    Paused,
}

impl<'de> serde::Deserialize<'de> for BackendStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "online" => Ok(BackendStatus::Online),
            "offline" => Ok(BackendStatus::Offline),
            "paused" => Ok(BackendStatus::Paused),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["online", "offline", "paused"],
            )),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[allow(dead_code)]
/// backend details
pub struct Backend {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    /// Additional details related to the backend status. May include status messages about maintenance work.
    pub message: Option<String>,

    /// Enum: "online" "offline" "paused"
    pub status: BackendStatus,

    /// Name of the backend
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    /// Version of the backend
    pub version: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
/// A list of [`Backend`] available for direct access.
pub struct Backends {
    pub backends: Vec<Backend>,
}
