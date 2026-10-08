use super::{arg, string};
use crate::core::{capture, state::State};
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some(match c {
        "get-sources" | "arch-sources" => capture::sources(),
        "arch-devices" => crate::core::devices::list(),
        "arch-record-status" => Ok(s.recorder.status()),
        "arch-record-recover" => string(a, 0).and_then(|p| {
            anyhow::ensure!(
                s.recorder.status()["recording"] != true,
                "Stop recording before recovery"
            );
            crate::core::recording_segments::recover(Path::new(p))
        }),
        "arch-record-start" => string(a, 0).and_then(|source| {
            anyhow::ensure!(
                s.recorder.status()["recording"] != true,
                "A recording is already active"
            );
            if let Some(hide_windows) = &s.before_capture {
                hide_windows()?;
            }
            let result = s.recorder.start(source, arg(a, 1));
            if result.is_err() {
                s.event("arch-show-recording-windows", json!([]));
            }
            result
        }),
        "arch-record-stop" | "stop-ffmpeg-recording" => s.recorder.stop().and_then(|v| {
            if let Some(p) = v["path"].as_str() {
                s.select_video(Path::new(p))?;
                s.project = None;
                if let Some(webcam) = v["webcamPath"].as_str() {
                    s.approve(Path::new(webcam))?;
                    s.session["webcamPath"] = webcam.into();
                }
            }
            s.event("arch-recording-complete", json!([v]));
            Ok(json!({"success":true,"session":s.session,"path":v["path"]}))
        }),
        "arch-record-pause" => s.recorder.pause(true),
        "arch-record-resume" => (|| {
            if s.recorder.paused {
                if let Some(hide_windows) = &s.before_capture {
                    hide_windows()?;
                }
            }
            let result = s.recorder.pause(false);
            if result.is_err() && s.recorder.status()["recording"] != true {
                s.event("arch-show-recording-windows", json!([]));
            }
            result
        })(),
        "select-source" => crate::core::files::set_setting("selectedSource", arg(a, 0).clone())
            .map(|_| json!({"success":true})),
        "get-selected-source" => Ok(crate::core::files::setting("selectedSource")),
        _ => return None,
    })
}
