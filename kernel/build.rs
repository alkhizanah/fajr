fn main() {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    // Tell cargo to pass the linker script to the linker..
    println!("cargo:rustc-link-arg=-Tkernel/src/arch/{arch}/linker.ld");
    // ..and to re-run if it changes.
    println!("cargo:rerun-if-changed=kernel/src/arch/{arch}/linker.ld");
}
