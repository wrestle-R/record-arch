use crate::{
    core::state::{Shared, State},
    rpc,
};
use serde_json::{json, Value};
use tauri::{Emitter, Manager};

#[tauri::command]
async fn command(
    app: tauri::AppHandle,
    state: tauri::State<'_, Shared>,
    channel: String,
    args: Vec<Value>,
) -> Result<Value, String> {
    if channel == "arch-hide-recording-windows" {
        for w in app.webview_windows().values() {
            let _ = w.hide();
        }
        return Ok(json!({"success":true}));
    }
    if channel == "arch-show-recording-windows" {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.show();
        }
        return Ok(json!({"success":true}));
    }
    let state = state.inner().clone();
    Ok(
        tauri::async_runtime::spawn_blocking(move || {
            match rpc::dispatch(&state, &channel, &args) {
                Ok(v) => v,
                Err(e) => json!({"success":false,"error":e.to_string(),"message":e.to_string()}),
            }
        })
        .await
        .unwrap_or_else(|e| json!({"success":false,"error":e.to_string()})),
    )
}
pub fn run() {
    let state = State::new().expect("Could not initialize project library");
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![command])
        .setup(move |app| {
            rpc::server::start(&state)?;
            let handle = app.handle().clone();
            let shared = state.clone();
            std::thread::spawn(move || loop {
                let events = {
                    let mut state = shared.lock().unwrap();
                    std::mem::take(&mut state.events)
                };
                for event in events {
                    let channel = event["channel"].as_str().unwrap_or("arch-event");
                    match channel {
                        "arch-show-recorder" => {
                            if let Some(w) = handle.get_webview_window("recorder") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            } else {
                                let _ = tauri::WebviewWindowBuilder::new(
                                    &handle,
                                    "recorder",
                                    tauri::WebviewUrl::App("index.html?windowType=recorder".into()),
                                )
                                .title("Record Arch · Recorder")
                                .inner_size(560., 620.)
                                .resizable(false)
                                .build();
                            }
                        }
                        "arch-hide-recording-windows" => {
                            for w in handle.webview_windows().values() {
                                let _ = w.hide();
                            }
                        }
                        "arch-open-editor"
                        | "arch-recording-complete"
                        | "arch-show-recording-windows" => {
                            if let Some(w) = handle.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        _ => {}
                    }
                    let _ = handle.emit(channel, event["args"].clone());
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Record Arch failed");
}
