//
// (C) Copyright Pasqal SAS 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.


use std::fmt;

#[derive(Debug)]
pub enum MungeError {
    EncodeFailed(String),
}

impl fmt::Display for MungeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MungeError::EncodeFailed(msg) => write!(f, "munge encode failed: {msg}"),
        }
    }
}

impl std::error::Error for MungeError {}
