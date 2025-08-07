//
// (C) Copyright IBM 2024
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Helpers to generate UUID.

use uuid::Uuid;

/// Create a new random (V4) UUID
///
/// # Example
///
/// ```rust
/// use direct_access_api::utils::uuid;
///
/// let _uuid = uuid::new_v4();
/// ```
#[allow(dead_code)]
pub fn new_v4() -> String {
    Uuid::new_v4().to_string()
}
