//! Menu-bar / system-tray icon: the app's only always-visible surface.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App,
};

pub fn build(app: &App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "Show status", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Radial Convert", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&status, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "status" => crate::show_main_window(app),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
