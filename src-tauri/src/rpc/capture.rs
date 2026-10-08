use super::{arg, string};
use crate::core::{capture, state::State};
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some(match c {
        "get-sources" | "arch-sources" => capture::sources(),
        "arch-record-status" => Ok(s.recorder.status()),
        "arch-record-start" => string(a, 0).and_then(|source| s.recorder.start(source, arg(a, 1))),
        "arch-record-stop" | "stop-ffmpeg-recording" => s.recorder.stop().and_then(|v| {
            if let Some(p) = v["path"].as_str() {
                s.select_video(Path::new(p))?;
                s.project = None;
            }
            s.event("arch-recording-complete", json!([v]));
            Ok(json!({"success":true,"session":s.session,"path":v["path"]}))
        }),
        "arch-record-pause" => s.recorder.pause(true),
        "arch-record-resume" => s.recorder.pause(false),
        "select-source" => crate::core::files::set_setting("selectedSource", arg(a, 0).clone())
            .map(|_| json!({"success":true})),
        "get-selected-source" => Ok(crate::core::files::setting("selectedSource")),
        _ => return None,
    })
}
