// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
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
    /// IBM Direct Access
    IBMDirectAccess,
    /// Qiskit Runtime Service
    QiskitRuntimeService,
    /// Pasqal Cloud
    PasqalCloud,
    // Pasqal Local
    PasqalLocal,
}
impl<'de> serde::Deserialize<'de> for ResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "direct-access" => Ok(ResourceType::IBMDirectAccess),
            "qiskit-runtime-service" => Ok(ResourceType::QiskitRuntimeService),
            "pasqal-cloud" => Ok(ResourceType::PasqalCloud),
            "pasqal-local" => Ok(ResourceType::PasqalLocal),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &[
                    "direct-access",
                    "qiskit-runtime-service",
                    "pasqal-cloud",
                    "pasqal-local",
                ],
            )),
        }
    }
}
impl ResourceType {
    pub fn as_str(&self) -> &str {
        match self {
            ResourceType::IBMDirectAccess => "direct-access",
            ResourceType::QiskitRuntimeService => "qiskit-runtime-service",
            ResourceType::PasqalCloud => "pasqal-cloud",
            ResourceType::PasqalLocal => "pasqal-local",
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

    /// environment variables
    pub environment: HashMap<String, String>,
}

/// A set of QRMI resource definitions
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourceDefs {
    /// resource name
    pub resources: Vec<ResourceDef>,
}

/// QRMI configuration file
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
