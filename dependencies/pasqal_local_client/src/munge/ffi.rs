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


use std::os::raw::{c_char, c_int, c_void};

#[link(name = "munge")]
extern "C" {
    pub(crate) fn munge_encode(
        cred: *mut *mut c_char,
        ctx: *mut std::ffi::c_void,
        data: *const c_void,
        len: usize,
    ) -> c_int;

    pub(crate) fn munge_strerror(err: c_int) -> *const c_char;
}