//! "Show in Finder / Explorer".

use std::path::Path;
use std::process::Command;

/// Opens a folder in Finder / Explorer.
pub fn open_folder(dir: &Path) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(dir).spawn();
    #[cfg(target_os = "windows")]
    let _ = Command::new("explorer").arg(dir).spawn();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let _ = Command::new("xdg-open").arg(dir).spawn();
}

/// The Explorer argument that selects a file: `/select,"C:\a b\c.pdf"`.
///
/// The quotes must surround only the path. Quoting the whole argument (which
/// is what passing it as one normal argument does) makes Explorer give up and
/// open the Documents folder, which is what every path containing a space did.
#[cfg_attr(not(windows), allow(dead_code))]
fn explorer_select_arg(path: &Path) -> String {
    format!("/select,\"{}\"", path.display())
}

/// Opens the folder holding `path` with the file highlighted.
pub fn reveal(path: &Path) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg("-R").arg(path).spawn();
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let _ = Command::new("explorer").raw_arg(explorer_select_arg(path)).spawn();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    if let Some(dir) = path.parent() {
        let _ = Command::new("xdg-open").arg(dir).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_quotes_surround_only_the_path() {
        let arg = explorer_select_arg(Path::new("C:/Users/me/Class_3_ periwinkle sheet.pdf"));
        assert!(arg.starts_with("/select,\""), "{arg}");
        assert!(arg.ends_with("sheet.pdf\""), "{arg}");
        assert!(!arg.starts_with('"'), "the whole argument must not be quoted: {arg}");
    }
}
