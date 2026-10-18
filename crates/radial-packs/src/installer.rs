//! Planning, installing, removing and checking engines.

use crate::catalog::{self, PACKS};
use crate::manifest::{ArchiveKind, Artifact, Manifest};
use crate::state::{InstalledEngine, Source, State};
use crate::{extract, fetch, paths, system};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

/// Progress notifications; the CLI turns these into progress bars.
#[derive(Debug, Clone)]
pub enum Event {
    Downloading { engine: String, done: u64, total: Option<u64> },
    Verifying { engine: String },
    Extracting { engine: String },
    Adopted { engine: String, path: PathBuf },
    Installed { engine: String },
}

#[derive(Debug, Clone)]
pub enum Action {
    AlreadyInstalled,
    /// Point at a tool that is already on the machine.
    Adopt { path: PathBuf },
    Download { version: String, artifact: Artifact },
    Unavailable { reason: String },
}

#[derive(Debug, Clone)]
pub struct PlanItem {
    pub engine: &'static str,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub packs: Vec<String>,
    pub items: Vec<PlanItem>,
}

impl Plan {
    pub fn download_bytes(&self) -> u64 {
        self.items
            .iter()
            .map(|i| match &i.action {
                Action::Download { artifact, .. } => artifact.size_bytes,
                _ => 0,
            })
            .sum()
    }

    /// Why this plan cannot be carried out, if it cannot.
    pub fn blockers(&self) -> Vec<String> {
        self.items
            .iter()
            .filter_map(|i| match &i.action {
                Action::Unavailable { reason } => Some(format!("{}: {reason}", i.engine)),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct DoctorLine {
    pub engine: String,
    pub ok: bool,
    pub detail: String,
}

pub struct Installer {
    home: PathBuf,
    manifest: Manifest,
    platform: String,
}

impl Installer {
    pub fn new(home: PathBuf, manifest: Manifest, platform: String) -> Self {
        Self { home, manifest, platform }
    }

    /// The installer for this machine: default home, and the built-in manifest
    /// unless `RADIAL_MANIFEST` names another one.
    pub fn for_this_machine() -> Result<Self, String> {
        Ok(Self::new(paths::default_home(), Manifest::from_env_or_embedded()?, crate::platform::key()))
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn state(&self) -> Result<State, String> {
        State::load(&self.home)
    }

    /// Executable of an installed engine, if it is installed and still on disk.
    /// The converter app calls this to find its tools.
    pub fn find_engine(home: &Path, id: &str) -> Option<PathBuf> {
        let state = State::load(home).ok()?;
        state.engines.get(id).filter(|e| e.path.is_file()).map(|e| e.path.clone())
    }

    /// Works out what installing `pack_ids` would do, without doing it.
    pub fn plan(&self, pack_ids: &[String]) -> Result<Plan, String> {
        for id in pack_ids {
            if catalog::pack(id).is_none() {
                let known: Vec<_> = PACKS.iter().map(|p| p.id).collect();
                return Err(format!("unknown pack \"{id}\" (available: {})", known.join(", ")));
            }
        }
        let state = self.state()?;
        let items = catalog::engines_for(pack_ids.iter().map(String::as_str))
            .into_iter()
            .map(|engine| PlanItem { engine, action: self.action_for(engine, &state) })
            .collect();
        Ok(Plan { packs: pack_ids.to_vec(), items })
    }

    fn action_for(&self, engine: &'static str, state: &State) -> Action {
        if state.engines.get(engine).is_some_and(|e| e.path.is_file()) {
            return Action::AlreadyInstalled;
        }
        // A published artifact wins: it is pinned, so every machine gets the same build.
        if let Some((version, artifact)) = self.manifest.artifact(engine, &self.platform) {
            return Action::Download { version: version.to_string(), artifact: artifact.clone() };
        }
        // Developer path: no artifact published, so use a tool already installed.
        let exe = catalog::engine(engine).map(|e| e.exe).unwrap_or(engine);
        match system::find(exe) {
            Some(path) => Action::Adopt { path },
            None => Action::Unavailable {
                reason: format!(
                    "no download is published for {} on {} and none is installed on this machine",
                    engine, self.platform
                ),
            },
        }
    }

    /// Installs the packs. Engines already present are left alone.
    pub fn install(&self, pack_ids: &[String], on_event: &mut dyn FnMut(Event)) -> Result<Plan, String> {
        let plan = self.plan(pack_ids)?;
        let blockers = plan.blockers();
        if !blockers.is_empty() {
            return Err(blockers.join("\n"));
        }

        let mut state = self.state()?;
        for item in &plan.items {
            match &item.action {
                Action::AlreadyInstalled => {}
                Action::Adopt { path } => {
                    state.engines.insert(
                        item.engine.to_string(),
                        InstalledEngine { version: "system".into(), source: Source::System, path: path.clone() },
                    );
                    on_event(Event::Adopted { engine: item.engine.to_string(), path: path.clone() });
                }
                Action::Download { version, artifact } => {
                    let exe = self.download_engine(item.engine, version, artifact, on_event)?;
                    state.engines.insert(
                        item.engine.to_string(),
                        InstalledEngine { version: version.clone(), source: Source::Downloaded, path: exe },
                    );
                    on_event(Event::Installed { engine: item.engine.to_string() });
                }
                Action::Unavailable { .. } => unreachable!("blockers were checked above"),
            }
            // Saved after each engine, so an interrupted install keeps what finished.
            state.save(&self.home)?;
        }
        state.packs.extend(pack_ids.iter().cloned());
        state.save(&self.home)?;
        Ok(plan)
    }

    fn download_engine(
        &self,
        engine: &str,
        version: &str,
        artifact: &Artifact,
        on_event: &mut dyn FnMut(Event),
    ) -> Result<PathBuf, String> {
        check_relative(&artifact.bin)?;
        let download = paths::downloads_dir(&self.home).join(format!("{engine}-{version}.part"));
        let engine_dir = paths::engines_dir(&self.home).join(engine);
        let partial = engine_dir.join(format!("{version}.partial"));
        let final_dir = engine_dir.join(version);

        let result = (|| -> Result<PathBuf, String> {
            fetch::download(&artifact.url, &download, &mut |done, total| {
                on_event(Event::Downloading { engine: engine.to_string(), done, total });
            })?;
            on_event(Event::Verifying { engine: engine.to_string() });
            fetch::verify(&download, &artifact.sha256)?;

            on_event(Event::Extracting { engine: engine.to_string() });
            let _ = std::fs::remove_dir_all(&partial);
            if artifact.archive == ArchiveKind::Raw {
                std::fs::create_dir_all(&partial).map_err(|e| e.to_string())?;
                std::fs::copy(&download, partial.join(&artifact.bin)).map_err(|e| e.to_string())?;
            } else {
                extract::extract(&download, artifact.archive, &partial)?;
            }
            let exe_rel = locate_executable(&partial, &artifact.bin)?;
            make_executable(&partial.join(&exe_rel))?;

            // Swap in only after everything worked, so a failure never leaves a half-installed engine.
            let _ = std::fs::remove_dir_all(&final_dir);
            std::fs::rename(&partial, &final_dir)
                .map_err(|e| format!("cannot finish installing {engine}: {e}"))?;
            Ok(final_dir.join(exe_rel))
        })();

        let _ = std::fs::remove_file(&download);
        if result.is_err() {
            let _ = std::fs::remove_dir_all(&partial);
        }
        result
    }

    /// Removes a pack. Returns the engines that were dropped because no
    /// remaining pack needs them.
    pub fn remove(&self, pack_id: &str) -> Result<Vec<String>, String> {
        let mut state = self.state()?;
        if !state.packs.remove(pack_id) {
            return Err(format!("pack \"{pack_id}\" is not installed"));
        }
        let still_needed = catalog::engines_for(state.packs.iter().map(String::as_str));
        let candidates = catalog::pack(pack_id).map(|p| p.engines).unwrap_or(&[]);

        let mut removed = Vec::new();
        for engine in candidates.iter().filter(|e| !still_needed.contains(*e)) {
            if let Some(installed) = state.engines.remove(*engine) {
                if installed.source == Source::Downloaded {
                    let _ = std::fs::remove_dir_all(paths::engines_dir(&self.home).join(engine));
                }
                removed.push(engine.to_string());
            }
        }
        state.save(&self.home)?;
        Ok(removed)
    }

    /// Runs every installed engine's version command and reports what works.
    pub fn doctor(&self) -> Result<Vec<DoctorLine>, String> {
        let state = self.state()?;
        Ok(state
            .engines
            .iter()
            .map(|(id, installed)| {
                let arg = catalog::engine(id).map(|e| e.version_arg).unwrap_or("--version");
                match run_version(&installed.path, arg) {
                    Ok(line) => DoctorLine { engine: id.clone(), ok: true, detail: line },
                    Err(why) => DoctorLine { engine: id.clone(), ok: false, detail: why },
                }
            })
            .collect())
    }
}

/// Where the executable ended up, relative to the extracted folder.
///
/// The manifest's exact path is used when it exists. Some packages (Windows
/// installers especially) nest their files under a folder whose name depends on
/// how they were authored, so otherwise a file of the same name is searched
/// for, and used only if there is exactly one.
fn locate_executable(root: &Path, bin: &str) -> Result<PathBuf, String> {
    if root.join(bin).is_file() {
        return Ok(PathBuf::from(bin));
    }
    let name = Path::new(bin).file_name().ok_or_else(|| format!("bad executable path \"{bin}\""))?;
    let mut found = Vec::new();
    find_files_named(root, root, name, &mut found);
    match found.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err(format!("the archive does not contain {bin}")),
        many => Err(format!(
            "expected {bin} but found {} files named {}; cannot tell which to run",
            many.len(),
            name.to_string_lossy()
        )),
    }
}

/// Collects paths (relative to `root`) of regular files called `name`. Symlinks are not followed.
fn find_files_named(root: &Path, dir: &Path, name: &std::ffi::OsStr, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.filter_map(Result::ok) {
        let Ok(kind) = entry.file_type() else { continue };
        let path = entry.path();
        if kind.is_dir() {
            find_files_named(root, &path, name, out);
        } else if kind.is_file() && entry.file_name() == name {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_path_buf());
            }
        }
    }
}

/// `bin` comes from the manifest; refuse anything that could point outside the engine folder.
fn check_relative(bin: &str) -> Result<(), String> {
    let ok = Path::new(bin).components().all(|c| matches!(c, Component::Normal(_)));
    if ok && !bin.is_empty() {
        Ok(())
    } else {
        Err(format!("manifest path \"{bin}\" must be relative and stay inside the engine folder"))
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("cannot make {} executable: {e}", path.display()))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn run_version(exe: &Path, arg: &str) -> Result<String, String> {
    let output = Command::new(exe)
        .arg(arg)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run {}: {e}", exe.display()))?;
    if !output.status.success() {
        return Err(format!("{} exited with {}", exe.display(), output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::io::Write;

    const PLATFORM: &str = "test-platform";

    struct Fixture {
        _dir: tempfile::TempDir,
        installer: Installer,
    }

    /// A fake "ffmpeg" shell script, zipped, served from a local file URL.
    fn fixture(sha_override: Option<&str>, bin: &str) -> Fixture {
        fixture_with_entries(&["bin/ffmpeg"], sha_override, bin)
    }

    fn fixture_with_entries(entries: &[&str], sha_override: Option<&str>, bin: &str) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("ffmpeg.zip");
        let mut writer = zip::ZipWriter::new(std::fs::File::create(&zip_path).unwrap());
        let options = zip::write::SimpleFileOptions::default().unix_permissions(0o644);
        for entry in entries {
            writer.start_file(*entry, options).unwrap();
            writer.write_all(b"#!/bin/sh\necho fake ffmpeg 1.0\n").unwrap();
        }
        writer.finish().unwrap();

        let sha = sha_override.map(String::from).unwrap_or_else(|| fetch::sha256_file(&zip_path).unwrap());
        let url = url::Url::from_file_path(&zip_path).unwrap();
        let manifest = Manifest::parse(&format!(
            r#"{{"version":1,"engines":{{"ffmpeg":{{"version":"1.0","platforms":{{
              "{PLATFORM}":{{"url":"{url}","sha256":"{sha}","archive":"zip","bin":"{bin}","size_bytes":10}}}}}}}}}}"#
        ))
        .unwrap();
        let home = dir.path().join("home");
        Fixture { installer: Installer::new(home, manifest, PLATFORM.into()), _dir: dir }
    }

    fn packs(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn installs_verifies_and_records_an_engine() {
        let f = fixture(None, "bin/ffmpeg");
        f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap();

        let exe = Installer::find_engine(f.installer.home(), "ffmpeg").expect("recorded");
        assert!(exe.is_file());
        assert!(exe.starts_with(f.installer.home().join("engines/ffmpeg/1.0")));
        let state = f.installer.state().unwrap();
        assert!(state.packs.contains("audio"));
        assert_eq!(state.engines["ffmpeg"].source, Source::Downloaded);
        // No leftovers from the download.
        assert!(!paths::downloads_dir(f.installer.home()).join("ffmpeg-1.0.part").exists());
    }

    #[test]
    fn packs_sharing_an_engine_download_it_once() {
        let f = fixture(None, "bin/ffmpeg");
        let downloads = Cell::new(0);
        let mut count = |e: Event| {
            if matches!(e, Event::Installed { .. }) {
                downloads.set(downloads.get() + 1);
            }
        };
        f.installer.install(&packs(&["audio", "images"]), &mut count).unwrap();
        f.installer.install(&packs(&["video"]), &mut count).unwrap();
        assert_eq!(downloads.get(), 1, "ffmpeg is shared by audio, images and video");
        assert_eq!(f.installer.state().unwrap().packs.len(), 3);
    }

    #[test]
    fn a_bad_checksum_installs_nothing() {
        let f = fixture(Some("deadbeef"), "bin/ffmpeg");
        let err = f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap_err();
        assert!(err.contains("checksum mismatch"), "{err}");
        let state = f.installer.state().unwrap();
        assert!(state.packs.is_empty() && state.engines.is_empty());
        assert!(!paths::engines_dir(f.installer.home()).join("ffmpeg/1.0").exists());
        assert!(!paths::engines_dir(f.installer.home()).join("ffmpeg/1.0.partial").exists());
    }

    #[test]
    fn removing_a_pack_keeps_an_engine_another_pack_needs() {
        let f = fixture(None, "bin/ffmpeg");
        f.installer.install(&packs(&["audio", "images"]), &mut |_| {}).unwrap();

        assert!(f.installer.remove("audio").unwrap().is_empty());
        assert!(Installer::find_engine(f.installer.home(), "ffmpeg").is_some());

        assert_eq!(f.installer.remove("images").unwrap(), vec!["ffmpeg"]);
        assert!(Installer::find_engine(f.installer.home(), "ffmpeg").is_none());
        assert!(!paths::engines_dir(f.installer.home()).join("ffmpeg").exists());
    }

    #[test]
    fn removing_a_pack_that_is_not_installed_says_so() {
        let f = fixture(None, "bin/ffmpeg");
        assert!(f.installer.remove("audio").unwrap_err().contains("not installed"));
    }

    #[test]
    fn unknown_pack_lists_the_valid_ones() {
        let f = fixture(None, "bin/ffmpeg");
        let err = f.installer.install(&packs(&["nope"]), &mut |_| {}).unwrap_err();
        assert!(err.contains("unknown pack") && err.contains("audio"), "{err}");
    }

    #[test]
    fn an_engine_with_no_download_and_no_system_copy_blocks_the_install() {
        let f = fixture(None, "bin/ffmpeg");
        // The fixture only publishes ffmpeg for a made-up platform; a made-up
        // engine name can't be on this machine, so use a platform with nothing.
        let installer = Installer::new(f.installer.home().to_path_buf(), Manifest::parse(r#"{"version":1,"engines":{}}"#).unwrap(), "nowhere".into());
        let plan = installer.plan(&packs(&["documents"])).unwrap();
        // LibreOffice may or may not be on the dev machine; either answer is valid, but never a panic.
        assert_eq!(plan.items.len(), 1);
        if let Action::Unavailable { reason } = &plan.items[0].action {
            assert!(reason.contains("no download is published"));
            assert!(installer.install(&packs(&["documents"]), &mut |_| {}).is_err());
        }
    }

    #[test]
    fn manifest_paths_cannot_escape_the_engine_folder() {
        for bad in ["../evil", "/etc/passwd", "a/../../b", ""] {
            assert!(check_relative(bad).is_err(), "{bad:?} should be rejected");
        }
        assert!(check_relative("bin/ffmpeg").is_ok());
        let f = fixture(None, "../escape");
        let err = f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap_err();
        assert!(err.contains("must be relative"), "{err}");
    }

    #[test]
    fn doctor_runs_the_installed_engine() {
        let f = fixture(None, "bin/ffmpeg");
        f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap();
        let lines = f.installer.doctor().unwrap();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].ok, "{:?}", lines[0]);
        assert_eq!(lines[0].detail, "fake ffmpeg 1.0");
    }

    #[test]
    fn a_missing_executable_shows_up_in_doctor() {
        let f = fixture(None, "bin/ffmpeg");
        f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap();
        let exe = Installer::find_engine(f.installer.home(), "ffmpeg").unwrap();
        std::fs::remove_file(&exe).unwrap();
        // find_engine now refuses a recorded path that vanished.
        assert!(Installer::find_engine(f.installer.home(), "ffmpeg").is_none());
        let lines = f.installer.doctor().unwrap();
        assert!(!lines[0].ok);
    }

    #[test]
    fn a_differently_nested_executable_is_found_by_name() {
        // The manifest expects bin/ffmpeg, but the package nested it elsewhere.
        let f = fixture_with_entries(&["ProgramFiles/App/ffmpeg"], None, "bin/ffmpeg");
        f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap();
        let exe = Installer::find_engine(f.installer.home(), "ffmpeg").unwrap();
        assert!(exe.ends_with("ProgramFiles/App/ffmpeg"), "{}", exe.display());
        assert_eq!(f.installer.doctor().unwrap()[0].detail, "fake ffmpeg 1.0");
    }

    #[test]
    fn two_candidates_are_refused_rather_than_guessed() {
        let f = fixture_with_entries(&["a/ffmpeg", "b/ffmpeg"], None, "bin/ffmpeg");
        let err = f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap_err();
        assert!(err.contains("cannot tell which"), "{err}");
        assert!(f.installer.state().unwrap().engines.is_empty());
    }

    #[test]
    fn an_archive_without_the_executable_is_an_error() {
        let f = fixture_with_entries(&["readme.txt"], None, "bin/ffmpeg");
        let err = f.installer.install(&packs(&["audio"]), &mut |_| {}).unwrap_err();
        assert!(err.contains("does not contain"), "{err}");
    }
}
