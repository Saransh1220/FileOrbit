//! The look of the ring and toast: which designs exist, which one is active,
//! and telling every window when it changes.

use crate::settings;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub const DEFAULT: &str = "current";

/// Selectable designs as (id, menu label). Ids must match `RING_VARIANTS` in
/// `src/ring/variants/ringVariants.ts` (a test below checks that).
pub const DESIGNS: &[(&str, &str)] = &[
    ("current", "Current"),
    ("orbit", "Orbit"),
    ("bold", "Bold"),
    ("blueprint", "Blueprint"),
    ("halo", "Halo"),
    ("chips", "Chips"),
    ("clock", "Clock"),
    ("brutal", "Brutal"),
];

pub fn is_known(id: &str) -> bool {
    DESIGNS.iter().any(|(known, _)| *known == id)
}

/// The active design id, shared with the windows through `get_design`.
pub struct CurrentDesign(Mutex<String>);

impl CurrentDesign {
    pub fn new(id: String) -> Self {
        Self(Mutex::new(id))
    }
}

/// The saved design, or the default when none is saved or the saved one no longer exists.
pub fn load(app: &AppHandle) -> String {
    settings::read_design(app).filter(|id| is_known(id)).unwrap_or_else(|| DEFAULT.to_string())
}

pub fn current(app: &AppHandle) -> String {
    app.state::<CurrentDesign>().0.lock().unwrap().clone()
}

/// Switches design: remembers it, saves it, and tells every window.
pub fn set(app: &AppHandle, id: &str) {
    if !is_known(id) {
        eprintln!("[design] ignoring unknown design {id:?}");
        return;
    }
    *app.state::<CurrentDesign>().0.lock().unwrap() = id.to_string();
    settings::write_design(app, id);
    eprintln!("[design] now {id}");
    let _ = app.emit("design-changed", id);
}

#[tauri::command]
pub fn get_design(app: AppHandle) -> String {
    current(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY: &str = include_str!("../../src/ring/variants/ringVariants.ts");

    #[test]
    fn default_is_a_known_design() {
        assert!(is_known(DEFAULT));
    }

    #[test]
    fn unknown_ids_are_rejected() {
        assert!(!is_known("nope"));
        assert!(!is_known(""));
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<_> = DESIGNS.iter().map(|(id, _)| *id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), DESIGNS.len());
    }

    #[test]
    fn menu_designs_match_the_ui_registry() {
        for (id, _) in DESIGNS {
            assert!(REGISTRY.contains(&format!("id: \"{id}\"")), "{id} is missing from ringVariants.ts");
        }
        let in_ui = REGISTRY.matches("id: \"").count();
        assert_eq!(in_ui, DESIGNS.len(), "ringVariants.ts and design.rs list different designs");
    }
}
