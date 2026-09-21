//! This crate is built as a cdylib with PyO3's `extension-module` feature,
//! which tells PyO3's build script to *not* link against libpython at
//! build time. That means this .so/.dylib can be built and shipped even on
//! machines that don't have libpython available.
//!
//! It is not meant to be loaded directly by the OS loader as a normal
//! dependency (that would fail: it has unresolved `Py_*` symbols). Instead
//! it is meant to be `dlopen`'d at runtime (see the `py_loader` crate),
//! *after* libpython has already been loaded into the process with
//! `RTLD_GLOBAL`, so the dynamic linker can resolve those symbols lazily
//! against the already-loaded libpython.

//! This crate is built as a cdylib with PyO3's `extension-module` feature,
//! which tells PyO3's build script to *not* link against libpython at
//! build time. That means this .so/.dylib can be built and shipped even on
//! machines that don't have libpython available.
//!
//! It is not meant to be loaded directly by the OS loader as a normal
//! dependency (that would fail: it has unresolved `Py_*` symbols). Instead
//! it is meant to be `dlopen`'d at runtime (see the `py_loader` crate),
//! *after* libpython has already been loaded into the process with
//! `RTLD_GLOBAL`, so the dynamic linker can resolve those symbols lazily
//! against the already-loaded libpython.
//!
//! No special setup is needed here to make the interpreter find its
//! stdlib and site-packages: `PYTHONHOME` is a standard env var that
//! CPython itself reads directly during `Py_Initialize`, as long as it's
//! present in the process environment (inherited from the shell that
//! launched `app`, or set programmatically before `app`/`py_loader` runs
//! this code) -- see `py_loader`'s use of `PYTHONHOME` to find the
//! `.so` itself, which is the same value CPython needs here.

use pyo3::prelude::*;
use std::os::raw::c_int;

/// C ABI entry point, looked up by name (`dlsym`) from `py_loader`.
///
/// Returns 0 on success, -1 on failure (Python exception or panic).
#[no_mangle]
pub extern "C" fn test() -> c_int {
    let result = std::panic::catch_unwind(|| -> PyResult<()> {
        // Note: as of PyO3 0.26 this is deprecated in favor of
        // `Python::initialize()`. Swap it out if you're on a newer version.
        Python::initialize();

        Python::attach(|py| {
            // In pyo3 0.21-0.22 the method is `run_bound` (takes &str).
            // In pyo3 <=0.20 and >=0.23 it's back to `run`, but 0.23+
            // takes a &CStr instead of &str (e.g. c"..." literal) -
            // adjust if you bump the pyo3 version in Cargo.toml.
            py.run(
                c"import sys; print(f'[py_bridge] hello from Python {sys.version}')\n\
                 print('[py_bridge] sys.path =', sys.path)",
                None,
                None,
            )
        })
    });

    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => {
            eprintln!("[py_bridge] python error: {e}");
            -1
        }
        Err(_) => {
            eprintln!("[py_bridge] panicked while calling python");
            -1
        }
    }
}
