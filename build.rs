// This code is part of Qiskit.
//
// (C) Copyright IBM 2025-2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use std::process::Command;

// For C API bindings
fn main() {
    for (key, value) in std::env::vars() {
        eprintln!("{key}: {value}");
    }
    // Pull the config from the cbindgen.toml file.
    let config = cbindgen::Config::from_file("cbindgen.toml").unwrap();

    match cbindgen::generate_with_config(".", config) {
        Ok(value) => {
            value.write_to_file("qrmi.h");
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
    if std::env::var("CARGO_FEATURE_MUNGE").is_ok() {
        println!("cargo:rustc-link-lib=munge");
    }

    println!("cargo:rerun-if-changed=/src/*");
    println!("cargo:rerun-if-changed=/build.rs");

    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());

    // Exposes GIT_HASH to env!("GIT_HASH") in lib.rs, at compile time.
    println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());
}
