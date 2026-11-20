fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_os == "windows" && target_env == "msvc" {
        // PE Subsystem 6.01 -> runs on Windows 7 SP1, Windows 8, Windows 8.1, Windows 10, Windows 11
        println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:CONSOLE,6.01");
    }
}
