//! What is installed, saved as `installed.json`.

use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// Downloaded and managed by us, under the install home.
    Downloaded,
    /// An existing tool on the machine that we only point at.
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledEngine {
    pub version: String,
    pub source: Source,
    /// Full path to the executable.
    pub path: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub packs: BTreeSet<String>,
    pub engines: BTreeMap<String, InstalledEngine>,
}

impl State {
    /// Missing file means nothing is installed yet; a corrupt file is an error.
    pub fn load(home: &Path) -> Result<Self, String> {
        let path = paths::state_path(home);
        match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|e| format!("{} is corrupt ({e}); delete it to start over", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("cannot read {}: {e}", path.display())),
        }
    }

    /// Written to a temp file first, then renamed, so a crash never leaves half a file.
    pub fn save(&self, home: &Path) -> Result<(), String> {
        std::fs::create_dir_all(home).map_err(|e| format!("cannot create {}: {e}", home.display()))?;
        let path = paths::state_path(home);
        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, json).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("cannot save {}: {e}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_an_empty_state() {
        let dir = tempfile::tempdir().unwrap();
        let state = State::load(dir.path()).unwrap();
        assert!(state.packs.is_empty() && state.engines.is_empty());
    }

    #[test]
    fn round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = State::default();
        state.packs.insert("audio".into());
        state.engines.insert(
            "ffmpeg".into(),
            InstalledEngine { version: "7.1".into(), source: Source::Downloaded, path: "/x/ffmpeg".into() },
        );
        state.save(dir.path()).unwrap();
        let back = State::load(dir.path()).unwrap();
        assert!(back.packs.contains("audio"));
        assert_eq!(back.engines["ffmpeg"].source, Source::Downloaded);
    }

    #[test]
    fn corrupt_file_is_reported_not_ignored() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(paths::state_path(dir.path()), "not json").unwrap();
        assert!(State::load(dir.path()).unwrap_err().contains("corrupt"));
    }
}
