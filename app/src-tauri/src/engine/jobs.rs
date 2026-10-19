//! Job lifecycle: run a conversion off the UI thread and report progress.
//!
//! If the conversion needs a program that isn't installed, the job pauses in
//! a "needs pack" state; the toast offers to install it, and once it is
//! installed the same job carries on.

use super::binaries::{self, Tool};
use super::pending::{PendingJob, PendingJobs};
use super::{download_text, engines, naming, registry};
use radial_packs::installer::{Event, Installer};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobState {
    Running,
    Done,
    Failed,
    /// Waiting for the user to install a pack.
    NeedsPack,
    /// A pack is downloading.
    Installing,
}

#[derive(Clone, Serialize)]
pub struct JobUpdate {
    pub id: u64,
    pub name: String,
    pub target: String,
    pub state: JobState,
    pub output: Option<String>,
    pub error: Option<String>,
    /// For `NeedsPack` / `Installing`: the pack id and its display title.
    pub pack: Option<String>,
    pub pack_title: Option<String>,
    /// For `NeedsPack` / `Installing`: the converter program(s) the pack needs, e.g. "ffmpeg".
    pub engine: Option<String>,
    /// For `NeedsPack`: download size in whole megabytes, when known.
    pub download_mb: Option<u64>,
    /// 0-100 while a download size is known.
    pub progress: Option<u8>,
    /// Short phase text such as "Downloading" or "Unpacking".
    pub message: Option<String>,
}

impl JobUpdate {
    fn new(id: u64, name: &str, target: &str, state: JobState) -> Self {
        Self {
            id,
            name: name.to_string(),
            target: target.to_ascii_uppercase(),
            state,
            output: None,
            error: None,
            pack: None,
            pack_title: None,
            engine: None,
            download_mb: None,
            progress: None,
            message: None,
        }
    }

    fn with(&self, state: JobState) -> Self {
        Self { state, output: None, error: None, progress: None, message: None, ..self.clone() }
    }
}

/// A pack that has to be installed before a conversion can run.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingPack {
    pub pack: &'static str,
    pub title: String,
}

/// Checks, before converting, that the program for this route is installed.
/// `find` is injected so this can be tested without touching the machine.
pub fn missing_pack(
    input: &Path,
    target_id: &str,
    find: &dyn Fn(Tool) -> Result<PathBuf, String>,
) -> Option<MissingPack> {
    let from_ext = registry::normalized_ext(input)?;
    let engine = registry::engine_for(&from_ext, target_id)?;
    let pack = registry::pack_for(&from_ext, target_id)?;
    if find(engines::tool_for(engine)).is_ok() {
        return None;
    }
    let title = radial_packs::catalog::pack(pack).map(|p| p.title).unwrap_or(pack);
    Some(MissingPack { pack, title: title.to_string() })
}

/// What the toast shows for one installer event.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Progress {
    pub percent: Option<u8>,
    pub message: String,
}

/// `elapsed` is how long this engine's download has been running (for speed and time left).
pub fn progress_of(event: &Event, elapsed: Duration) -> Progress {
    let (percent, message) = match event {
        Event::Downloading { engine, done, total } => (
            total.filter(|t| *t > 0).map(|t| ((done * 100) / t).min(100) as u8),
            download_text::downloading(engine, *done, *total, elapsed),
        ),
        Event::Verifying { engine } => (None, format!("Checking the {engine} download")),
        Event::Extracting { engine } => (None, format!("Unpacking {engine}, this can take a minute")),
        Event::Adopted { engine, .. } => (None, format!("Using {engine} already on this computer")),
        Event::Installed { engine } => (Some(100), format!("{engine} installed")),
    };
    Progress { percent, message }
}

/// The converter program names a pack needs, and the size of what would be
/// downloaded for it on this machine (None when that cannot be worked out).
fn pack_needs(pack: &str) -> (String, Option<u64>) {
    let names = radial_packs::catalog::pack(pack)
        .map(|p| {
            p.engines
                .iter()
                .map(|id| radial_packs::catalog::engine(id).map(|e| e.name).unwrap_or(id))
                .collect::<Vec<_>>()
                .join(" + ")
        })
        .unwrap_or_default();
    let bytes = Installer::for_this_machine()
        .and_then(|i| i.plan(&[pack.to_string()]))
        .map(|plan| plan.download_bytes())
        .ok();
    (names, bytes)
}

/// Converts one file synchronously and returns the path written.
pub fn convert_file(input: &Path, target_id: &str, job_id: u64) -> Result<PathBuf, String> {
    let from_ext = registry::normalized_ext(input)
        .ok_or_else(|| "the file has no extension, so its type is unknown".to_string())?;
    let engine = registry::engine_for(&from_ext, target_id)
        .ok_or_else(|| format!("cannot convert .{from_ext} to .{target_id}"))?;
    if !input.is_file() {
        return Err("the source file no longer exists".to_string());
    }
    let output = naming::reserve_output_path(input, target_id)?;
    if let Err(message) = engines::run(engine, input, &from_ext, &output, target_id, job_id) {
        // The output name was reserved with an empty file; don't leave it behind.
        let _ = std::fs::remove_file(&output);
        return Err(message);
    }
    Ok(output)
}

/// Starts a conversion in the background and streams updates to the toast.
pub fn start(app: AppHandle, input: PathBuf, target_id: String) {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    run(app, id, input, target_id);
}

fn run(app: AppHandle, id: u64, input: PathBuf, target_id: String) {
    let name = input.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let base = JobUpdate::new(id, &name, &target_id, JobState::Running);

    if let Some(missing) = missing_pack(&input, &target_id, &binaries::find) {
        app.state::<PendingJobs>().insert(id, PendingJob { input, target: target_id, pack: missing.pack });
        let mut update = base.with(JobState::NeedsPack);
        update.pack = Some(missing.pack.to_string());
        update.pack_title = Some(missing.title);
        let (engine, bytes) = pack_needs(missing.pack);
        update.engine = Some(engine);
        update.download_mb = bytes.map(download_text::mb);
        crate::toast::publish(&app, &update);
        return;
    }

    crate::toast::publish(&app, &base);
    std::thread::spawn(move || finish(&app, &base, &input, &target_id));
}

/// Runs the conversion and publishes how it ended.
fn finish(app: &AppHandle, base: &JobUpdate, input: &Path, target_id: &str) {
    let update = match convert_file(input, target_id, base.id) {
        Ok(path) => {
            let mut done = base.with(JobState::Done);
            done.output = Some(path.to_string_lossy().into_owned());
            done
        }
        Err(message) => {
            eprintln!("[job {}] failed: {message}", base.id);
            let mut failed = base.with(JobState::Failed);
            failed.error = Some(message);
            failed
        }
    };
    crate::toast::publish(app, &update);
}

/// The user clicked Install on a "needs pack" toast: install it with live
/// progress, then finish the conversion that was waiting.
pub fn install_pack_for_job(app: AppHandle, id: u64) {
    let Some(pending) = app.state::<PendingJobs>().take(id) else {
        return;
    };
    std::thread::spawn(move || {
        let name = pending.input.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let base = JobUpdate::new(id, &name, &pending.target, JobState::Running);
        let title = radial_packs::catalog::pack(pending.pack).map(|p| p.title).unwrap_or(pending.pack);
        let engine = pack_needs(pending.pack).0;

        let installing = |progress: Option<&Progress>| {
            let mut update = base.with(JobState::Installing);
            update.pack = Some(pending.pack.to_string());
            update.pack_title = Some(title.to_string());
            update.engine = Some(engine.clone());
            update.progress = progress.and_then(|p| p.percent);
            update.message = progress.map(|p| p.message.clone());
            update
        };
        crate::toast::publish(&app, &installing(None));

        // Only publish when the visible numbers change; downloads report every chunk.
        let mut last: Option<Progress> = None;
        let mut started = Instant::now();
        let mut downloading_engine = String::new();
        let result = Installer::for_this_machine().and_then(|installer| {
            installer.install(&[pending.pack.to_string()], &mut |event| {
                // Speed is per engine: restart the clock when a different one begins.
                if let Event::Downloading { engine, .. } = &event {
                    if *engine != downloading_engine {
                        downloading_engine = engine.clone();
                        started = Instant::now();
                    }
                }
                let progress = progress_of(&event, started.elapsed());
                if last.as_ref() != Some(&progress) {
                    crate::toast::publish(&app, &installing(Some(&progress)));
                    last = Some(progress);
                }
            })
        });

        match result {
            Ok(_) => {
                crate::toast::publish(&app, &base);
                finish(&app, &base, &pending.input, &pending.target);
            }
            Err(message) => {
                eprintln!("[job {id}] could not install {}: {message}", pending.pack);
                let mut failed = base.with(JobState::Failed);
                failed.error = Some(message);
                crate::toast::publish(&app, &failed);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn absent(_: Tool) -> Result<PathBuf, String> {
        Err("not installed".into())
    }

    fn present(_: Tool) -> Result<PathBuf, String> {
        Ok("/usr/bin/x".into())
    }

    #[test]
    fn a_missing_program_names_its_pack() {
        let missing = missing_pack(Path::new("/a/report.docx"), "pdf", &absent).unwrap();
        assert_eq!(missing, MissingPack { pack: "documents", title: "Documents".into() });
        let audio = missing_pack(Path::new("/a/song.mp3"), "wav", &absent).unwrap();
        assert_eq!(audio.pack, "audio");
    }

    #[test]
    fn nothing_is_missing_when_the_program_is_there() {
        assert!(missing_pack(Path::new("/a/report.docx"), "pdf", &present).is_none());
    }

    #[test]
    fn an_impossible_route_is_not_a_missing_pack() {
        // Reported later as "cannot convert", not as an install prompt.
        assert!(missing_pack(Path::new("/a/thing.xyz"), "pdf", &absent).is_none());
        assert!(missing_pack(Path::new("/a/noext"), "pdf", &absent).is_none());
    }

    #[test]
    fn download_progress_is_a_percentage() {
        let p = progress_of(&Event::Downloading { engine: "ffmpeg".into(), done: 25, total: Some(100) }, Duration::ZERO);
        assert_eq!(p.percent, Some(25));
        assert_eq!(p.message, "Downloading ffmpeg: 0 of 0 MB");
        let unknown = progress_of(&Event::Downloading { engine: "x".into(), done: 5, total: None }, Duration::ZERO);
        assert_eq!(unknown.percent, None);
        let zero = progress_of(&Event::Downloading { engine: "x".into(), done: 0, total: Some(0) }, Duration::ZERO);
        assert_eq!(zero.percent, None);
    }

    #[test]
    fn progress_never_exceeds_one_hundred() {
        let p = progress_of(&Event::Downloading { engine: "x".into(), done: 150, total: Some(100) }, Duration::ZERO);
        assert_eq!(p.percent, Some(100));
    }

    #[test]
    fn later_phases_have_their_own_labels() {
        assert_eq!(progress_of(&Event::Verifying { engine: "x".into() }, Duration::ZERO).message, "Checking the x download");
        assert!(progress_of(&Event::Extracting { engine: "x".into() }, Duration::ZERO).message.starts_with("Unpacking x"));
    }
}
