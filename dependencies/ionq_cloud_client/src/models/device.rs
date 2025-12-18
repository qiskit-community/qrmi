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

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "UPPERCASE"))]
pub enum DeviceType {
    Simulator,
    Harmony,
    Aria1,
    Aria2,
    Forte1,
    ForteEnterprise1,
    ForteEnterprise2,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            DeviceType::Simulator => "simulator",
            DeviceType::Harmony => "qpu.harmony",
            DeviceType::Aria1 => "qpu.aria-1",
            DeviceType::Aria2 => "qpu.aria-2",
            DeviceType::Forte1 => "qpu.forte-1",
            DeviceType::ForteEnterprise1 => "qpu.forte-enterprise-1",
            DeviceType::ForteEnterprise2 => "qpu.forte-enterprise-2",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for DeviceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "simulator" => Ok(DeviceType::Simulator),
            "qpu.harmony" => Ok(DeviceType::Harmony),
            "qpu.aria-1" => Ok(DeviceType::Aria1),
            "qpu.aria-2" => Ok(DeviceType::Aria2),
            "qpu.forte-1" => Ok(DeviceType::Forte1),
            "qpu.forte-enterprise-1" => Ok(DeviceType::ForteEnterprise1),
            "qpu.forte-enterprise-2" => Ok(DeviceType::ForteEnterprise2),
            _ => Err(()),
        }
    }
}

