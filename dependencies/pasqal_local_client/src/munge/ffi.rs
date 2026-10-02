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

extern "C" {
    fn free(ptr: *mut c_void);
}

/// Releases a credential string allocated by `munge_encode`.
pub(crate) unsafe fn free_cred(ptr: *mut c_char) {
    free(ptr as *mut c_void);
}

// Requires libmunge headers/lib at build time.
#[cfg(feature = "munge")]
mod linked {
    use super::*;
    use std::sync::Once;

    #[link(name = "munge")]
    extern "C" {
        fn munge_encode(
            cred: *mut *mut c_char,
            ctx: *mut c_void,
            data: *const c_void,
            len: c_int,
        ) -> c_int;

        fn munge_strerror(err: c_int) -> *const c_char;
    }

    static LOG_ONCE: Once = Once::new();

    pub(crate) unsafe fn call_munge_encode(
        cred: *mut *mut c_char,
        ctx: *mut c_void,
        data: *const c_void,
        len: c_int,
    ) -> Result<c_int, String> {
        LOG_ONCE.call_once(|| {
            log::debug!(
                "munge: using build-time dynamically linked libmunge (feature = \"munge\")"
            );
        });
        Ok(munge_encode(cred, ctx, data, len))
    }

    pub(crate) unsafe fn call_munge_strerror(err: c_int) -> Result<*const c_char, String> {
        Ok(munge_strerror(err))
    }
}

// Fallback: dlopen libmunge at runtime so the client works on hosts that have qrmi
// installed without a special build or the munge-devel package installed.
#[cfg(not(feature = "munge"))]
mod dynamic {
    use super::*;
    use libloading::Library;
    use std::sync::OnceLock;

    type MungeEncodeFn =
        unsafe extern "C" fn(*mut *mut c_char, *mut c_void, *const c_void, c_int) -> c_int;
    type MungeStrerrorFn = unsafe extern "C" fn(c_int) -> *const c_char;

    struct Munge {
        // `encode` and `strerror` are raw fn pointers copied out of `Symbol<'lib, _>`, so the
        // borrow checker no longer ties them to the library. Owning the `Library` here
        // defers its `dlclose` until this struct is dropped, keeping them valid.
        _lib: Library,
        encode: MungeEncodeFn,
        strerror: MungeStrerrorFn,
    }

    // Only a successful load is cached, so a process started before munge was installed
    // picks it up on the next request instead of failing until restart.
    static MUNGE: OnceLock<Munge> = OnceLock::new();

    unsafe fn load(name: &str) -> Result<Munge, libloading::Error> {
        let lib = Library::new(name)?;
        let encode = *lib.get::<MungeEncodeFn>(b"munge_encode\0")?;
        let strerror = *lib.get::<MungeStrerrorFn>(b"munge_strerror\0")?;
        Ok(Munge {
            _lib: lib,
            encode,
            strerror,
        })
    }

    fn munge() -> Result<&'static Munge, String> {
        // Munge has no Windows port and the sonames below are Linux-only.
        if cfg!(not(target_os = "linux")) {
            return Err("munge is only supported on Linux.".into());
        }
        if let Some(m) = MUNGE.get() {
            return Ok(m);
        }
        // The fn pointer types above match ABI 2, so prefer the versioned soname and only
        // fall back to the unversioned devel symlink.
        let mut errors = Vec::new();
        for name in ["libmunge.so.2", "libmunge.so"] {
            match unsafe { load(name) } {
                Ok(m) => {
                    log::debug!("munge: loaded {name} dynamically at runtime (dlopen)");
                    // A concurrent caller may have won the race; either result is equivalent.
                    return Ok(MUNGE.get_or_init(|| m));
                }
                Err(e) => errors.push(format!("{name}: {e}")),
            }
        }
        Err(format!(
            "libmunge could not be loaded ({}). Install munge (the package providing \
             libmunge.so.2) on this host.",
            errors.join("; ")
        ))
    }

    pub(crate) unsafe fn call_munge_encode(
        cred: *mut *mut c_char,
        ctx: *mut c_void,
        data: *const c_void,
        len: c_int,
    ) -> Result<c_int, String> {
        Ok((munge()?.encode)(cred, ctx, data, len))
    }

    pub(crate) unsafe fn call_munge_strerror(err: c_int) -> Result<*const c_char, String> {
        Ok((munge()?.strerror)(err))
    }
}

#[cfg(feature = "munge")]
pub(crate) use linked::{call_munge_encode as munge_encode, call_munge_strerror as munge_strerror};

#[cfg(not(feature = "munge"))]
pub(crate) use dynamic::{
    call_munge_encode as munge_encode, call_munge_strerror as munge_strerror,
};
