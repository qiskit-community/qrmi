//
// (C) Copyright Pasqal SAS 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MungeError {
    #[error("munge encode failed: {0}")]
    EncodeFailed(String),

    /// Munge support wasn't linked in at build time and `libmunge.so`
    /// couldn't be loaded dynamically either.
    #[error("munge unavailable: {0}")]
    Unavailable(String),
}
