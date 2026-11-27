// This code is part of Qiskit.
//
// (C) Copyright Pasqal 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.
use tokio::runtime::Runtime;

/// Builds the tokio runtime that the C and Python bindings drive their calls
/// with.
///
/// `Runtime::new()` does not report every failure through its `Result`: when
/// the OS refuses the first worker thread, or the sockets for signal
/// handling, tokio panics instead. A panic aborts a C host and reaches Python
/// as `PanicException`, so both kinds of failure are returned as an error.
pub(crate) fn new_runtime() -> crate::Result<Runtime> {
    runtime_from(Runtime::new)
}

fn runtime_from(
    build: impl FnOnce() -> std::io::Result<Runtime> + std::panic::UnwindSafe,
) -> crate::Result<Runtime> {
    let reason = match std::panic::catch_unwind(build) {
        Ok(Ok(runtime)) => return Ok(runtime),
        Ok(Err(err)) => err.to_string(),
        Err(panic) => panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "tokio panicked".to_string()),
    };
    Err(anyhow::anyhow!("failed to create a tokio runtime: {reason}").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_errors_are_returned() {
        let err = runtime_from(|| Err(std::io::Error::other("no file descriptors"))).unwrap_err();
        assert_eq!(
            err.to_string(),
            "failed to create a tokio runtime: no file descriptors"
        );
    }

    #[test]
    fn runtime_panics_are_returned() {
        let err = runtime_from(|| panic!("OS can't spawn worker thread")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "failed to create a tokio runtime: OS can't spawn worker thread"
        );
        assert_eq!(err.kind(), crate::error::QrmiErrorKind::Other);
    }

    #[test]
    fn runtime_is_built() {
        assert_eq!(new_runtime().unwrap().block_on(async { 1 }), 1);
    }
}
