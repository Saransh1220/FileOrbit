use std::path::{Path, PathBuf};

/// Environment variable that overrides the install folder (tests, portable
/// installs, enterprise mirrors).
pub const HOME_ENV: &str = "RADIAL_HOME";

/// `~/Library/Application Support/Radial Convert` or `%LOCALAPPDATA%\Radial Convert`.
pub fn default_home() -> PathBuf {
    if let Some(custom) = std::env::var_os(HOME_ENV) {
        return PathBuf::from(custom);
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Radial Convert")
}

pub fn engines_dir(home: &Path) -> PathBuf {
    home.join("engines")
}

pub fn state_path(home: &Path) -> PathBuf {
    home.join("installed.json")
}

pub fn downloads_dir(home: &Path) -> PathBuf {
    home.join("downloads")
}
