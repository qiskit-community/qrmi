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
pub struct ServiceVersionV1 {
    /// API version value (N.N.N format)
    pub version: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceVersionV2Info {
    /// API name
    pub api_name: String,
    /// API descriptions
    pub api_description: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceVersionV2Version {
    /// Calendar-based API version value (YYYYMMDD format)
    pub ibm_api_version: String,
    /// Is this version the latest
    pub latest: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceVersionV2Alpha {
    /// API metadata
    pub info: ServiceVersionV2Info,
    /// API Version related data
    pub version: ServiceVersionV2Version,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ServiceVersion {
    V2(ServiceVersionV2Alpha),
    V1(ServiceVersionV1),
}
