//! Backend of the converter manager window: what is installed, installing,
//! removing, and the download cache.

use crate::design;
use crate::engine::jobs::progress_of;
use radial_packs::catalog::{self, PACKS};
use radial_packs::installer::{Event, Installer};
use radial_packs::state::Source;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};

/// Allows one install at a time: two packs often share a converter program,
/// and two installs writing the same folder would corrupt it.
#[derive(Default)]
pub struct InstallLock(Mutex<Option<String>>);

impl InstallLock {
    /// Claims the lock for `pack`, or says which pack holds it.
    pub fn begin(&self, pack: &str) -> Result<(), String> {
        let mut current = self.0.lock().unwrap();
        match current.as_deref() {
            Some(other) => {
                let title = catalog::pack(other).map(|p| p.title).unwrap_or(other);
                Err(format!("{title} is being installed. Wait for it to finish."))
            }
            None => {
                *current = Some(pack.to_string());
                Ok(())
            }
        }
    }

    pub fn end(&self) {
        *self.0.lock().unwrap() = None;
    }

    fn holder(&self) -> Option<String> {
        self.0.lock().unwrap().clone()
    }
}

#[derive(Serialize)]
pub struct EngineInfo {
    name: String,
    /// Version of the installed copy, when one is installed.
    version: Option<String>,
    /// "downloaded" (managed by us) or "system" (already on this computer).
    source: Option<&'static str>,
}

#[derive(Serialize)]
pub struct PackInfo {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    engines: Vec<EngineInfo>,
    installed: bool,
    /// Download size on this platform; `None` when nothing is published for it.
    download_bytes: Option<u64>,
    /// Space the pack's programs take on disk (shared programs count for each pack using them).
    disk_bytes: u64,
    /// Titles of other packs that use the same program.
    shared_with: Vec<&'static str>,
    /// True while this pack is being installed.
    busy: bool,
}

#[derive(Serialize)]
pub struct Overview {
    packs: Vec<PackInfo>,
    /// Space used by kept downloads.
    cache_bytes: u64,
    /// Space used by installed programs.
    installed_bytes: u64,
    folder: String,
}

#[derive(Serialize, Clone)]
struct Update {
    pack: String,
    /// "installing", "done" or "failed".
    state: &'static str,
    message: Option<String>,
    percent: Option<u8>,
    error: Option<String>,
}

fn installer() -> Result<Installer, String> {
    Installer::for_this_machine()
}

/// Tells the manager window to reload its list (something changed elsewhere, e.g. from a toast).
pub fn notify_changed(app: &AppHandle) {
    let _ = app.emit("manager-changed", ());
}

fn publish(app: &AppHandle, update: Update) {
    let _ = app.emit("manager-update", update);
}

#[tauri::command(async)]
pub fn get_overview(app: AppHandle) -> Result<Overview, String> {
    let installer = installer()?;
    let state = installer.state()?;
    let busy = app.state::<InstallLock>().holder();

    let packs: Vec<PackInfo> = PACKS
        .iter()
        .map(|pack| {
            let engines = pack
                .engines
                .iter()
                .map(|id| {
                    let installed = state.engines.get(*id);
                    EngineInfo {
                        name: catalog::engine(id).map(|e| e.name).unwrap_or(id).to_string(),
                        version: installed.map(|e| e.version.clone()),
                        source: installed.map(|e| match e.source {
                            Source::Downloaded => "downloaded",
                            Source::System => "system",
                        }),
                    }
                })
                .collect();
            let shared_with = PACKS
                .iter()
                .filter(|other| other.id != pack.id && other.engines.iter().any(|e| pack.engines.contains(e)))
                .map(|other| other.title)
                .collect();
            PackInfo {
                id: pack.id,
                title: pack.title,
                description: pack.description,
                engines,
                installed: state.packs.contains(pack.id),
                download_bytes: installer.pack_download_bytes(pack.id),
                disk_bytes: pack.engines.iter().map(|e| installer.engine_disk_bytes(e)).sum(),
                shared_with,
                busy: busy.as_deref() == Some(pack.id),
            }
        })
        .collect();

    Ok(Overview {
        packs,
        cache_bytes: installer.cache_bytes(),
        installed_bytes: catalog::ENGINES.iter().map(|e| installer.engine_disk_bytes(e.id)).sum(),
        folder: installer.home().display().to_string(),
    })
}

/// Installs a pack in the background, reporting `manager-update` events.
#[tauri::command]
pub fn install_pack(app: AppHandle, pack: String) -> Result<(), String> {
    if catalog::pack(&pack).is_none() {
        return Err(format!("unknown pack \"{pack}\""));
    }
    app.state::<InstallLock>().begin(&pack)?;
    std::thread::spawn(move || {
        let say = |state, message: Option<String>, percent, error: Option<String>| {
            publish(&app, Update { pack: pack.clone(), state, message, percent, error });
        };
        say("installing", Some("Starting".into()), None, None);

        let mut started = Instant::now();
        let mut engine_now = String::new();
        let result = installer().and_then(|installer| {
            installer.install(&[pack.clone()], &mut |event| {
                if let Event::Downloading { engine, .. } = &event {
                    if *engine != engine_now {
                        engine_now = engine.clone();
                        started = Instant::now();
                    }
                }
                let progress = progress_of(&event, started.elapsed());
                say("installing", Some(progress.message), progress.percent, None);
            })
        });

        app.state::<InstallLock>().end();
        match result {
            Ok(_) => say("done", None, Some(100), None),
            Err(error) => {
                eprintln!("[manager] could not install {pack}: {error}");
                say("failed", None, None, Some(error));
            }
        }
        notify_changed(&app);
    });
    Ok(())
}

#[tauri::command(async)]
pub fn remove_pack(app: AppHandle, pack: String) -> Result<(), String> {
    if app.state::<InstallLock>().holder().is_some() {
        return Err("Wait for the current install to finish first.".into());
    }
    installer()?.remove(&pack)?;
    notify_changed(&app);
    Ok(())
}

#[tauri::command(async)]
pub fn clear_downloads(app: AppHandle) -> Result<(), String> {
    if app.state::<InstallLock>().holder().is_some() {
        return Err("Wait for the current install to finish first.".into());
    }
    installer()?.clear_cache()?;
    notify_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn open_data_folder() -> Result<(), String> {
    let home = installer()?.home().to_path_buf();
    std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    crate::reveal::open_folder(&home);
    Ok(())
}

/// The look chosen in the manager's settings; same as the tray's Theme menu.
#[tauri::command]
pub fn set_design(app: AppHandle, id: String) -> Result<(), String> {
    if !design::is_known(&id) {
        return Err(format!("unknown design \"{id}\""));
    }
    design::set(&app, &id);
    Ok(())
}

/// The designs for the settings picker, as (id, label).
#[tauri::command]
pub fn list_designs() -> Vec<(&'static str, &'static str)> {
    design::DESIGNS.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_install_runs_at_a_time() {
        let lock = InstallLock::default();
        assert!(lock.begin("audio").is_ok());
        let err = lock.begin("video").unwrap_err();
        assert!(err.contains("Audio"), "{err}");
        lock.end();
        assert!(lock.begin("video").is_ok());
    }

    #[test]
    fn the_holder_is_reported_while_busy() {
        let lock = InstallLock::default();
        assert_eq!(lock.holder(), None);
        lock.begin("text").unwrap();
        assert_eq!(lock.holder().as_deref(), Some("text"));
    }
}
