use super::arg;
use crate::core::{cursor_telemetry, state::State};
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;

pub fn handle(s: &mut State, channel: &str, args: &[Value]) -> Option<Result<Value>> {
    if !matches!(channel, "get-cursor-telemetry" | "set-cursor-telemetry") {
        return None;
    }
    Some((|| {
        let path = arg(args, 0).as_str().map(Path::new).or(s.video.as_deref());
        let Some(path) = path else {
            return Ok(
                json!({"success":channel == "get-cursor-telemetry","samples":[],"message":"No video path available for cursor telemetry"}),
            );
        };
        let path = s.readable(path)?;
        let samples = if channel == "get-cursor-telemetry" {
            cursor_telemetry::load(&path)?
        } else {
            cursor_telemetry::save(&path, arg(args, 1))?
        };
        Ok(json!({"success":true,"samples":samples}))
    })())
}
