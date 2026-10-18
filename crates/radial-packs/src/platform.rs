/// Key used in the manifest: `macos-aarch64`, `macos-x86_64`, `windows-x86_64`, ...
pub fn key() -> String {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };
    format!("{os}-{}", std::env::consts::ARCH)
}
