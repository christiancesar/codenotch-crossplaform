fn main() {
    let mut attrs = tauri_build::Attributes::new();

    // tauri-build embeds its Common Controls v6 manifest into the binary only, so on Windows the
    // lib's unit-test executable dies at startup with STATUS_ENTRYPOINT_NOT_FOUND. The linker
    // embeds the same manifest into every target instead, tests included.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        attrs = attrs.windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }

    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
