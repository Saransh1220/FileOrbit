//! Small saved preferences (just the chosen design for now), stored as JSON
//! in the app's config folder. A missing or unreadable file means defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Default, Serialize, Deserialize)]
struct Saved {
    design: Option<String>,
}

fn file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("settings.json"))
}

fn read(path: &PathBuf) -> Saved {
    std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

pub fn read_design(app: &AppHandle) -> Option<String> {
    read(&file(app)?).design
}

pub fn write_design(app: &AppHandle, id: &str) {
    let Some(path) = file(app) else {
        return;
    };
    let mut saved = read(&path);
    saved.design = Some(id.to_string());
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match serde_json::to_string_pretty(&saved) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&path, text) {
                eprintln!("[settings] could not save {}: {e}", path.display());
            }
        }
        Err(e) => eprintln!("[settings] could not encode settings: {e}"),
    }
}
