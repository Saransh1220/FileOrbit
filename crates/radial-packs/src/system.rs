//! Finding a tool that is already on the machine (developer fallback).

use std::path::PathBuf;

/// Set to any value to pretend no tools are installed on the machine, so a
/// developer's computer can behave like a clean user's (testing the installer).
pub const IGNORE_ENV: &str = "RADIAL_NO_SYSTEM_TOOLS";

#[cfg(target_os = "macos")]
const EXTRA_DIRS: &[&str] = &[
    "/opt/homebrew/bin",
    "/usr/local/bin",
    "/usr/bin",
    "/Applications/LibreOffice.app/Contents/MacOS",
];

#[cfg(target_os = "windows")]
const EXTRA_DIRS: &[&str] = &[
    r"C:\Program Files\LibreOffice\program",
    r"C:\Program Files (x86)\LibreOffice\program",
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const EXTRA_DIRS: &[&str] = &["/usr/local/bin", "/usr/bin"];

/// Looks for `exe` (no extension) on PATH, then in well-known install folders.
/// A GUI app launched from Finder or the Start menu has a minimal PATH, which
/// is why the well-known folders are searched explicitly.
pub fn find(exe: &str) -> Option<PathBuf> {
    if std::env::var_os(IGNORE_ENV).is_some() {
        return None;
    }
    let file = if cfg!(windows) { format!("{exe}.exe") } else { exe.to_string() };
    let path_dirs = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .unwrap_or_default();
    path_dirs
        .into_iter()
        .chain(EXTRA_DIRS.iter().map(PathBuf::from))
        .map(|dir| dir.join(&file))
        .find(|candidate| candidate.is_file())
}
