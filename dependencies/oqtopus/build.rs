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
