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

use std::os::raw::{c_char, c_int, c_void};

// Statically linked munge, available when built with `--features munge`
// (requires libmunge headers/lib present at build time).
#[cfg(feature = "munge")]
mod linked {
    use super::*;

    #[link(name = "munge")]
    extern "C" {
        fn munge_encode(
            cred: *mut *mut c_char,
            ctx: *mut c_void,
            data: *const c_void,
            len: usize,
        ) -> c_int;

        fn munge_strerror(err: c_int) -> *const c_char;
    }

    pub(crate) unsafe fn call_munge_encode(
        cred: *mut *mut c_char,
        ctx: *mut c_void,
        data: *const c_void,
        len: usize,
    ) -> Result<c_int, String> {
        Ok(munge_encode(cred, ctx, data, len))
    }

    pub(crate) unsafe fn call_munge_strerror(err: c_int) -> Result<*const c_char, String> {
        Ok(munge_strerror(err))
    }
}

// Fallback when not built with `--features munge`: try to dlopen libmunge.so
// at runtime, so the client still works on hosts that have it installed
// without requiring a special build.
// ponytail: only tries the default soname, add SONAME/path overrides if a
// host ever needs a non-standard libmunge location.
#[cfg(not(feature = "munge"))]
mod dynamic {
    use super::*;
    use libloading::{Library, Symbol};
    use std::sync::OnceLock;

    type MungeEncodeFn =
        unsafe extern "C" fn(*mut *mut c_char, *mut c_void, *const c_void, usize) -> c_int;
    type MungeStrerrorFn = unsafe extern "C" fn(c_int) -> *const c_char;

    static LIB: OnceLock<Result<Library, String>> = OnceLock::new();

    fn library() -> Result<&'static Library, String> {
        LIB.get_or_init(|| unsafe {
            Library::new("libmunge.so").map_err(|e| {
                format!(
                    "munge support was not compiled in and libmunge.so could not be \
                     loaded dynamically ({e}). Install munge or rebuild with --features munge."
                )
            })
        })
        .as_ref()
        .map_err(|e| e.clone())
    }

    pub(crate) unsafe fn call_munge_encode(
        cred: *mut *mut c_char,
        ctx: *mut c_void,
        data: *const c_void,
        len: usize,
    ) -> Result<c_int, String> {
        let lib = library()?;
        let func: Symbol<MungeEncodeFn> = lib
            .get(b"munge_encode\0")
            .map_err(|e| format!("munge_encode symbol not found: {e}"))?;
        Ok(func(cred, ctx, data, len))
    }

    pub(crate) unsafe fn call_munge_strerror(err: c_int) -> Result<*const c_char, String> {
        let lib = library()?;
        let func: Symbol<MungeStrerrorFn> = lib
            .get(b"munge_strerror\0")
            .map_err(|e| format!("munge_strerror symbol not found: {e}"))?;
        Ok(func(err))
    }
}

#[cfg(feature = "munge")]
pub(crate) use linked::{call_munge_encode as munge_encode, call_munge_strerror as munge_strerror};

#[cfg(not(feature = "munge"))]
pub(crate) use dynamic::{
    call_munge_encode as munge_encode, call_munge_strerror as munge_strerror,
};
