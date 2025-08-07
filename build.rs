// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

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

    println!("cargo:rerun-if-changed=/src/*");
    println!("cargo:rerun-if-changed=/build.rs");
}
