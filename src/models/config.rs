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

use qrmi_core_api::{ResourceDef, ResourceDefs};

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
