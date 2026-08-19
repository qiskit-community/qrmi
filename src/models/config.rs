// This code is part of Qiskit.
//
// (C) Copyright IBM 2025-2026
//
// This program and the accompanying materials are made available under the
// terms of the GNU General Public License version 3, as published by the
// Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
//

#![allow(dead_code)]

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/// QRMI resource types
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// IBM Quantum System
    IBMQuantumSystem,
    /// Qiskit Runtime Service
    QiskitRuntimeService,
    /// Pasqal Cloud
    PasqalCloud,
    // Pasqal Local
    PasqalLocal,
    /// Alice Bob Felis
    AliceBobFelis,
    // IQM Server
    IQMServer,
}
impl<'de> serde::Deserialize<'de> for ResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ResourceType::from_qpu_type_str(&s).ok_or_else(|| {
            serde::de::Error::unknown_variant(
                &s,
                &[
                    "ibm-quantum-system",
                    "qiskit-runtime-service",
                    "pasqal-cloud",
                    "pasqal-local",
                    "alice-bob-felis",
                    "iqm-server",
                ],
            )
        })
    }
}
impl ResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::IBMQuantumSystem => "ibm-quantum-system",
            ResourceType::QiskitRuntimeService => "qiskit-runtime-service",
            ResourceType::PasqalCloud => "pasqal-cloud",
            ResourceType::PasqalLocal => "pasqal-local",
            ResourceType::AliceBobFelis => "alice-bob-felis",
            ResourceType::IQMServer => "iqm-server",
        }
    }

    /// Parses one of QRMI's `qpu_type` strings (e.g. as used in the
    /// `QRMI_JOB_QPU_TYPES` environment variable, or the `"type"` field of a
    /// `qrmi_config.json` resource definition) into a `ResourceType`.
    ///
    /// Returns `None` for an unrecognized string, rather than an error,
    /// since callers iterating over a job's QPU list generally want to warn
    /// and skip an unsupported entry rather than abort the whole scan.
    pub fn from_qpu_type_str(s: &str) -> Option<Self> {
        match s {
            "ibm-quantum-system" => Some(ResourceType::IBMQuantumSystem),
            "qiskit-runtime-service" => Some(ResourceType::QiskitRuntimeService),
            "pasqal-cloud" => Some(ResourceType::PasqalCloud),
            "pasqal-local" => Some(ResourceType::PasqalLocal),
            "alice-bob-felis" => Some(ResourceType::AliceBobFelis),
            "iqm-server" => Some(ResourceType::IQMServer),
            _ => None,
        }
    }
}

/// A QRMI resource definition
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourceDef {
    /// resource name
    pub name: String,

    /// resource type
    pub r#type: ResourceType,

    /// If true, backends are discovered dynamically via ResourceProvider.
    /// If false (default), this is a static resource definition.
    #[serde(default)]
    pub is_dynamic: bool,

    /// environment variables
    pub environment: HashMap<String, String>,
}

impl ResourceDef {
    /// Returns true if this resource definition is dynamic (i.e. backends are
    /// discovered via ResourceProvider::resources()).
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }
}

/// A set of QRMI resource definitions
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourceDefs {
    /// resource name
    pub resources: Vec<ResourceDef>,
}

/// QRMI configuration file
///
/// # Example
///
/// ```no_run
/// use qrmi::models::Config;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load("./qrmi_example_config.json")?;
///
/// if let Some(resource) = config.resource_map.get("ibm_osaka") {
///     println!("Found resource: {}", resource.name);
///     println!("Type: {:?}", resource.r#type);
///     for (key, value) in &resource.environment {
///         println!("Environment variable: {}={}", key, value);
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct Config {
    pub resource_map: HashMap<String, ResourceDef>,
}
impl Config {
    pub fn load(filename: &str) -> Result<Config> {
        let f = match File::open(filename) {
            Ok(v) => v,
            Err(err) => {
                bail!("Failed to open {}. reason = {}", filename, err);
            }
        };

        // reads qrmi_config.json and parse it.
        let mut buf_reader = BufReader::new(f);
        let mut config_json_str = String::new();
        buf_reader.read_to_string(&mut config_json_str)?;
        // returns Err if fails to parse a file - invalid JSON, invalid resource type etc.
        let items = serde_json::from_str::<ResourceDefs>(&config_json_str)?;
        let mut item_map: HashMap<String, ResourceDef> = HashMap::new();
        for item in items.resources {
            item_map.insert(item.name.clone(), item);
        }
        Ok(Self {
            resource_map: item_map,
        })
    }
}
