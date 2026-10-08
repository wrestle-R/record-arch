use crate::{core::state::Shared, rpc};
use serde_json::json;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
pub fn install(app: &tauri::App) -> tauri::Result<()> {
    let editor = MenuItem::with_id(app, "editor", "Open editor", true, None::<&str>)?;
    let record = MenuItem::with_id(app, "record", "New recording", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Stop recording", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause recording", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Resume recording", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Record Arch", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&editor, &record, &stop, &pause, &resume, &quit])?;
    let mut tray = TrayIconBuilder::with_id("record-arch")
        .tooltip("Record Arch")
        .menu(&menu);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.on_menu_event(|app, event| {
        let channel = match event.id.as_ref() {
            "editor" => "arch-open-editor",
            "record" => "arch-show-recorder",
            "stop" => "arch-record-stop",
            "pause" => "arch-record-pause",
            "resume" => "arch-record-resume",
            "quit" => "arch-quit",
            _ => return,
        };
        let state = app.state::<Shared>().inner().clone();
        let app = app.clone();
        std::thread::spawn(move || {
            if channel == "arch-quit" {
                let recording = state.lock().unwrap().recorder.status()["recording"] == true;
                if recording {
                    let _ = rpc::dispatch(&state, "arch-record-stop", &[]);
                }
                // Ask the editor to save before closing through its existing close guard.
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.close();
                }
            } else if let Err(error) = rpc::dispatch(&state, channel, &[]) {
                let mut s = state.lock().unwrap();
                s.event("arch-error", json!([error.to_string()]));
            }
        });
    })
    .build(app)?;
    Ok(())
}
