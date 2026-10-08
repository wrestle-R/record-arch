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
    if channel == "arch-smoke-report" && std::env::var_os("RECORD_ARCH_SMOKE_VIDEO").is_some() {
        crate::core::files::write_json(
            &crate::core::files::root().join(".cache/native-smoke.json"),
            args.first().unwrap_or(&Value::Null),
        )
        .map_err(|e| e.to_string())?;
        eprintln!(
            "Native smoke result: {}",
            args.first().unwrap_or(&Value::Null)
        );
        app.exit(if args.first().is_some_and(|v| v["success"] == true) {
            0
        } else {
            1
        });
        return Ok(json!({"success":true}));
    }
    if channel == "arch-smoke-diagnostic" && std::env::var_os("RECORD_ARCH_SMOKE_VIDEO").is_some() {
        eprintln!(
            "Webview diagnostic: {}",
            args.first().unwrap_or(&Value::Null)
        );
        return Ok(json!({"success":true}));
    }
    if channel == "arch-close-approved" {
        state.lock().map_err(|e| e.to_string())?.unsaved = false;
        app.exit(0);
        return Ok(json!({"success":true}));
    }
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
            let capture_app = app.handle().clone();
            state.lock().unwrap().before_capture = Some(Box::new(move || {
                let (sender, receiver) = std::sync::mpsc::channel();
                let handle = capture_app.clone();
                capture_app.run_on_main_thread(move || {
                    for window in handle.webview_windows().values() { let _ = window.hide(); }
                    let _ = sender.send(());
                })?;
                receiver.recv_timeout(std::time::Duration::from_secs(5))?;
                // Let the compositor present the hidden windows before requesting frames.
                std::thread::sleep(std::time::Duration::from_millis(100));
                Ok(())
            }));
            if let Some(path) = std::env::var_os("RECORD_ARCH_SMOKE_VIDEO") {
                state
                    .lock()
                    .unwrap()
                    .select_video(std::path::Path::new(&path))?;
            }
            rpc::server::start(&state)?;
            if let Err(error) = crate::tray::install(app) {
                eprintln!("Tray unavailable: {error}. Use the CLI to control recording.");
            }
            let handle = app.handle().clone();
            let shared = state.clone();
            if std::env::var_os("RECORD_ARCH_SMOKE_VIDEO").is_some() {
                let diagnostic = state.clone();
                let diagnostic_app=app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(8));
                    if let Some(w)=diagnostic_app.get_webview_window("main"){let _=w.eval("window.__TAURI_INTERNALS__.invoke('command',{channel:'arch-smoke-diagnostic',args:[{url:location.href,body:document.body.innerText.slice(0,600),readyState:document.readyState,errors:window.recordArchStartupErrors??[],videoFrame:typeof VideoFrame,videoEncoder:typeof VideoEncoder}]})");}
                    diagnostic
                        .lock()
                        .unwrap()
                        .event("arch-native-smoke", json!([]));
                });
            }
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
                                .inner_size(560., 780.)
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
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let shared = window.state::<Shared>();
                    let mut s = shared.lock().unwrap();
                    if s.unsaved {
                        api.prevent_close();
                        let _ = window.emit(
                            "request-save-before-close",
                            json!([uuid::Uuid::new_v4().to_string()]),
                        );
                    } else if s.recorder.status()["recording"] == true {
                        api.prevent_close();
                        let _ = window.hide();
                    } else {
                        // A hidden recorder window must not keep the process alive.
                        api.prevent_close();
                        window.app_handle().exit(0);
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Record Arch failed")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                let shared = app.state::<Shared>();
                let mut s = shared.lock().unwrap();
                if s.recorder.status()["recording"] == true {
                    let _ = s.recorder.stop();
                }
                s.encoders.clear();
            }
        });
}
