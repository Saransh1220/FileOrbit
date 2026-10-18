//! The list of pinned engine downloads, shipped inside the CLI.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveKind {
    Zip,
    TarGz,
    /// A single executable, downloaded as-is.
    Raw,
    /// macOS disk image (LibreOffice). Extraction not implemented yet.
    Dmg,
    /// Windows installer package (LibreOffice). Extraction not implemented yet.
    Msi,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub sha256: String,
    pub archive: ArchiveKind,
    /// Path of the executable inside the extracted archive (the file name for `raw`).
    pub bin: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineRelease {
    pub version: String,
    /// Keyed by platform, e.g. `macos-aarch64`.
    pub platforms: BTreeMap<String, Artifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub engines: BTreeMap<String, EngineRelease>,
}

/// Names a JSON file to use instead of the built-in manifest (company mirrors,
/// offline installs, testing against local copies).
pub const MANIFEST_ENV: &str = "RADIAL_MANIFEST";

impl Manifest {
    /// The file named by `RADIAL_MANIFEST` if set, otherwise the built-in one.
    pub fn from_env_or_embedded() -> Result<Self, String> {
        match std::env::var_os(MANIFEST_ENV) {
            None => Ok(Self::embedded()),
            Some(path) => {
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("cannot read {MANIFEST_ENV} file {}: {e}", path.to_string_lossy()))?;
                Self::parse(&text)
            }
        }
    }

    /// The manifest compiled into this build.
    pub fn embedded() -> Self {
        Self::parse(include_str!("../engines.manifest.json")).expect("embedded manifest is valid")
    }

    pub fn parse(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid manifest: {e}"))
    }

    /// Release version and artifact for an engine on a platform, if published.
    pub fn artifact(&self, engine: &str, platform: &str) -> Option<(&str, &Artifact)> {
        let release = self.engines.get(engine)?;
        let artifact = release.platforms.get(platform)?;
        Some((release.version.as_str(), artifact))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_manifest_parses() {
        let m = Manifest::embedded();
        assert_eq!(m.version, 1);
    }

    #[test]
    fn looks_up_by_engine_and_platform() {
        let m = Manifest::parse(
            r#"{"version":1,"engines":{"ffmpeg":{"version":"7.1","platforms":{
              "macos-aarch64":{"url":"u","sha256":"abc","archive":"tar-gz","bin":"ffmpeg","size_bytes":5}}}}}"#,
        )
        .unwrap();
        let (version, artifact) = m.artifact("ffmpeg", "macos-aarch64").unwrap();
        assert_eq!(version, "7.1");
        assert_eq!(artifact.archive, ArchiveKind::TarGz);
        assert!(m.artifact("ffmpeg", "windows-x86_64").is_none());
        assert!(m.artifact("pandoc", "macos-aarch64").is_none());
    }

    #[test]
    fn rejects_garbage() {
        assert!(Manifest::parse("{").is_err());
    }
}
