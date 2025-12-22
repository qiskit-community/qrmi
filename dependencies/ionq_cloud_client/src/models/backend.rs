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

use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Backend {
    Simulator,
    QpuHarmony,
    QpuAria1,
    QpuAria2,
    QpuForte1,
    QpuForteEnterprise1,
    QpuForteEnterprise2,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Backend::Simulator => "simulator",
            Backend::QpuHarmony => "qpu.harmony",
            Backend::QpuAria1 => "qpu.aria-1",
            Backend::QpuAria2 => "qpu.aria-2",
            Backend::QpuForte1 => "qpu.forte-1",
            Backend::QpuForteEnterprise1 => "qpu.forte-enterprise-1",
            Backend::QpuForteEnterprise2 => "qpu.forte-enterprise-2",
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Backend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_trim = s.trim();
        let s_norm = s_trim.to_ascii_lowercase();

        let backend = match s_norm.as_str() {
            // IMPORTANT: IonQ docs show "simulator" is valid for job creation.
            "simulator" | "sim" => Backend::Simulator,
            "qpu.harmony" | "harmony" | "qpu_harmony" => Backend::QpuHarmony,
            "qpu.aria-1" | "qpu.aria1" | "aria-1" | "aria1" => Backend::QpuAria1,
            "qpu.aria-2" | "qpu.aria2" | "aria-2" | "aria2" => Backend::QpuAria2,
            "qpu.forte-1" | "qpu.forte1" | "forte-1" | "forte1" => Backend::QpuForte1,
            "qpu.forte-enterprise-1" | "qpu.forte_enterprise-1" | "forte-enterprise-1" => {
                Backend::QpuForteEnterprise1
            }
            "qpu.forte-enterprise-2" | "qpu.forte_enterprise-2" | "forte-enterprise-2" => {
                Backend::QpuForteEnterprise2
            }
            _ => bail!("unknown IonQ backend: '{s_trim}'"),
        };
        Ok(backend)
    }
}
