use super::{arg, string};
use crate::core::{files, media, state::State};
use anyhow::Result;
use serde_json::{json, Value};
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    let value = match c {
        "arch-show-recorder"
        | "arch-hide-recording-windows"
        | "arch-show-recording-windows"
        | "arch-open-editor" => {
            s.event(c, json!([]));
            json!({"success":true})
        }
        "set-has-unsaved-changes" => {
            s.unsaved = arg(a, 0) == true;
            json!({"success":true})
        }
        "arch-doctor" => media::doctor(),
        "get-platform" => json!("linux"),
        "app:getVersion" => json!(env!("CARGO_PKG_VERSION")),
        "get-editor-mode" => json!(true),
        "get-window-fullscreen" => json!(false),
        "get-window-chrome" => json!({"trafficLightsVisible":false}),
        "get-linux-window-system" => json!(if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            "wayland"
        } else {
            "x11"
        }),
        "get-hud-overlay-capture-protection" | "set-hud-overlay-capture-protection" => {
            json!({"supported":false,"enabled":false})
        }
        "get-hud-overlay-mouse-passthrough-supported" => {
            json!({"supported":false,"resizeAnchor":"center"})
        }
        "get-countdown-delay" => {
            json!({"success":true,"delay":files::setting("countdown").as_u64().unwrap_or(3)})
        }
        "get-active-countdown" => json!({"success":true,"seconds":null}),
        "get-shortcuts" => files::setting("shortcuts"),
        "get-recording-preferences" => {
            let mut prefs = files::setting("recordingPreferences");
            if !prefs.is_object() {
                prefs = json!({});
            }
            prefs["success"] = true.into();
            prefs
        }
        "get-recording-audio-lab-config" => json!({"browserMicrophoneProfile":"processed"}),
        "arch-settings" => {
            files::read_json(&files::root().join("settings.json")).unwrap_or(json!({}))
        }
        "arch-set-setting" => {
            return Some((|| {
                files::set_setting(string(a, 0)?, arg(a, 1).clone())?;
                Ok(json!({"success":true}))
            })())
        }
        "set-countdown-delay" | "save-shortcuts" | "set-recording-preferences" => {
            let key = match c {
                "save-shortcuts" => "shortcuts",
                "set-countdown-delay" => "countdown",
                _ => "recordingPreferences",
            };
            return Some(
                files::set_setting(key, arg(a, 0).clone()).map(|_| json!({"success":true})),
            );
        }
        "get-native-export-capabilities" => {
            json!({"success":true,"capabilities":{"platform":"linux","nvidiaCuda":{"available":false,"skipReason":"Rust FFmpeg export uses software rendering","hasNvidiaGpu":null,"hasWrapper":false,"explicitEnabled":false}}})
        }
        "get-export-hardware-info" => {
            json!({"success":true,"hardware":{"platform":"linux","release":"","arch":std::env::consts::ARCH,"cpuModel":null,"logicalProcessors":std::thread::available_parallelism().map(|p|p.get()).unwrap_or(1),"totalMemoryGb":0,"machineModel":null,"gpus":[],"gpuFeatures":{"webgl":"enabled","webgpu":null}}})
        }
        "get-accessibility-permission-status" | "request-accessibility-permission" => {
            json!({"success":true,"trusted":false,"granted":false})
        }
        "get-screen-recording-permission-status" => json!({"success":true,"status":"unknown"}),
        "is-native-windows-capture-available" => json!(false),
        "get-whisper-small-model-status" => crate::core::models::status(),
        "get-system-cursor-assets" => json!({"success":true,"assets":{}}),
        "get-video-audio-fallback-paths" => {
            let paths = string(a, 0)
                .ok()
                .map(|p| crate::core::devices::audio_sidecars(std::path::Path::new(p)))
                .unwrap_or_default();
            json!({"success":true,"paths":paths})
        }
        "announcements:get" => json!({"success":true,"announcements":[]}),
        "auth:get-pending-callback" => Value::Null,
        _ => return None,
    };
    Some(Ok(value))
}
