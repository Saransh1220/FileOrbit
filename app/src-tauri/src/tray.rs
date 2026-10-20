//! Menu-bar / system-tray icon: the app's only always-visible surface.

use crate::design;
use tauri::{
    menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
    App, Wry,
};

/// Menu item ids for design choices look like `design:orbit`.
const DESIGN_PREFIX: &str = "design:";

pub fn build(app: &App) -> tauri::Result<()> {
    let active = design::current(app.handle());
    let choices: Vec<CheckMenuItem<Wry>> = design::DESIGNS
        .iter()
        .map(|(id, label)| CheckMenuItem::with_id(app, format!("{DESIGN_PREFIX}{id}"), *label, true, *id == active, None::<&str>))
        .collect::<tauri::Result<_>>()?;
    let choice_refs: Vec<&dyn IsMenuItem<Wry>> = choices.iter().map(|c| c as &dyn IsMenuItem<Wry>).collect();
    let theme = Submenu::with_items(app, "Theme", true, &choice_refs)?;

    let status = MenuItem::with_id(app, "status", "Manage converters", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Radial Convert", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&status, &theme, &separator, &quit])?;

    let mut builder = TrayIconBuilder::new().menu(&menu).on_menu_event(move |app, event| {
        let id = event.id.as_ref();
        if let Some(chosen) = id.strip_prefix(DESIGN_PREFIX) {
            design::set(app, chosen);
            // A check item toggles itself when clicked; keep exactly the chosen one checked.
            for item in &choices {
                let _ = item.set_checked(item.id().as_ref() == id);
            }
            return;
        }
        match id {
            "quit" => app.exit(0),
            "status" => crate::show_main_window(app),
            _ => {}
        }
    });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
