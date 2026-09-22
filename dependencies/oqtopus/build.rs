//
// (C) Copyright IBM 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

fn main() {
    // macOS's Mach-O linker rejects undefined symbols in a dylib by default,
    // unlike Linux's ELF linker. Since the `extension-module` feature
    // deliberately leaves the Py_* symbols unresolved (they are meant to be
    // resolved at dlopen time, either by the host Python interpreter or by
    // us manually loading libpython first), we need to tell the macOS linker
    // to allow this.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
    }
}
