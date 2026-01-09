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


mod ffi;
mod error; 

use crate::munge::error::MungeError;

use std::ffi::CStr;
use std::ptr;


pub fn encode(payload: &[u8]) -> Result<String, MungeError> {
    let mut cred_ptr = ptr::null_mut();

    let rc = unsafe {
        ffi::munge_encode(
            &mut cred_ptr,
            ptr::null_mut(),
            payload.as_ptr() as *const _,
            payload.len(),
        )
    };

    if rc != 0 {
        let msg = unsafe {
            CStr::from_ptr(ffi::munge_strerror(rc))
                .to_string_lossy()
                .into_owned()
        };
        return Err(MungeError::EncodeFailed(msg));
    }

    if cred_ptr.is_null() {
        return Err(MungeError::EncodeFailed("null credential".into()));
    }

    let token = unsafe {
        CStr::from_ptr(cred_ptr)
            .to_string_lossy()
            .into_owned()
    };

    Ok(token)
}
