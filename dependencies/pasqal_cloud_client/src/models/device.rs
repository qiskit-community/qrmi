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

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "UPPERCASE"))]
pub enum DeviceType {
    Fresnel,
    FresnelCan1,
    EmuMps,
    EmuFree,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            DeviceType::Fresnel => "FRESNEL",
            DeviceType::FresnelCan1 => "FRESNEL_CAN1",
            DeviceType::EmuMps => "EMU_MPS",
            DeviceType::EmuFree => "EMU_FREE",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for DeviceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "FRESNEL" => Ok(DeviceType::Fresnel),
            "FRESNEL_CAN1" => Ok(DeviceType::FresnelCan1),
            "EMU_MPS" => Ok(DeviceType::EmuMps),
            "EMU_FREE" => Ok(DeviceType::EmuFree),
            _ => Err(()),
        }
    }
}

