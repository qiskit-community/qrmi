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
    // Cargo adds Cargo.toml.orig to every packaged crate, so this tells a
    // build of the packaged crate (e.g. when `cargo publish` verifies it, or
    // as a crates.io dependency) from a build in a checkout of the repository.
    let packaged = std::path::Path::new("Cargo.toml.orig").exists();

    // qrmi.h is only needed by the C and Lua builds, which use a checkout of
    // the repository. Skip it when building the packaged crate: a build
    // script must not write outside OUT_DIR there, and cargo refuses to
    // publish if it does.
    if !packaged {
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
    }
    if std::env::var("CARGO_FEATURE_MUNGE").is_ok() {
        println!("cargo:rustc-link-lib=munge");
    }

    println!("cargo:rerun-if-changed=/src/*");
    println!("cargo:rerun-if-changed=/build.rs");

    // The packaged crate has no .git: its commit is the one cargo recorded in
    // .cargo_vcs_info.json when packaging it. Running git there would find no
    // repository, or a different one if the crate sits inside it.
    let git_hash = if packaged {
        packaged_git_hash()
    } else {
        checkout_git_hash()
    }
    .unwrap_or_else(|| "unknown".to_string());

    // Exposes GIT_HASH to env!("GIT_HASH") in lib.rs, at compile time.
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

/// Short hash of the commit checked out, from git.
fn checkout_git_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let hash = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!hash.is_empty()).then_some(hash)
}

/// Short hash of the commit the crate was packaged from, from the
/// .cargo_vcs_info.json cargo adds to it, with a "-dirty" suffix when the
/// working tree had uncommitted changes. Cargo leaves the file out when
/// packaging with --allow-dirty outside a clean git checkout.
///
/// The file is small and its format fixed, e.g.:
///   {"git": {"sha1": "d922ad92ae9c...", "dirty": true}, "path_in_vcs": ""}
/// so it is read with a string search rather than a JSON parser, to keep
/// build dependencies out of every build of qrmi.
fn packaged_git_hash() -> Option<String> {
    println!("cargo:rerun-if-changed=.cargo_vcs_info.json");
    let info = std::fs::read_to_string(".cargo_vcs_info.json").ok()?;
    let sha1 = json_string_value(&info, "sha1")?;
    if sha1.len() < 12 || !sha1.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let dirty = info.split_once("\"dirty\"").is_some_and(|(_, rest)| {
        rest.trim_start()
            .trim_start_matches(':')
            .trim_start()
            .starts_with("true")
    });
    Some(format!(
        "{}{}",
        &sha1[..12],
        if dirty { "-dirty" } else { "" }
    ))
}

/// The string value of the first "key" in a JSON document.
fn json_string_value<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let (_, rest) = json.split_once(&format!("\"{key}\""))?;
    let rest = rest
        .trim_start()
        .strip_prefix(':')?
        .trim_start()
        .strip_prefix('"')?;
    rest.split_once('"').map(|(value, _)| value)
}
