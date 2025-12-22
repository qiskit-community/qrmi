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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Backend {
    Simulator,
    Harmony,
    Aria1,
    Aria2,
    Forte1,
    ForteEnterprise1,
    ForteEnterprise2,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Backend::Simulator => "simulator",
            Backend::Harmony => "qpu.harmony",
            Backend::Aria1 => "qpu.aria-1",
            Backend::Aria2 => "qpu.aria-2",
            Backend::Forte1 => "qpu.forte-1",
            Backend::ForteEnterprise1 => "qpu.forte-enterprise-1",
            Backend::ForteEnterprise2 => "qpu.forte-enterprise-2",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Backend {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "simulator" => Ok(Backend::Simulator),
            "qpu.harmony" => Ok(Backend::Harmony),
            "qpu.aria-1" => Ok(Backend::Aria1),
            "qpu.aria-2" => Ok(Backend::Aria2),
            "qpu.forte-1" => Ok(Backend::Forte1),
            "qpu.forte-enterprise-1" => Ok(Backend::ForteEnterprise1),
            "qpu.forte-enterprise-2" => Ok(Backend::ForteEnterprise2),
            _ => Err(()),
        }
    }
}
