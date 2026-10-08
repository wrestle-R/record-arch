use super::{files, recording_sidecars};
use anyhow::Result;
use serde_json::{json, Value};
use std::{fs, path::Path};

const MAX_SAMPLES: usize = 60 * 60 * 30;

pub fn normalize(raw: &Value) -> Vec<Value> {
    let mut points: Vec<Value> = raw
        .as_array()
        .or_else(|| raw["samples"].as_array())
        .into_iter()
        .flatten()
        .take(MAX_SAMPLES)
        .filter(|p| p.is_object())
        .map(|p| {
            let mut point = json!({
                "timeMs": p["timeMs"].as_f64().unwrap_or(0.).max(0.),
                "cx": p["cx"].as_f64().unwrap_or(0.5).clamp(0., 1.),
                "cy": p["cy"].as_f64().unwrap_or(0.5).clamp(0., 1.)
            });
            for (key, allowed) in [
                (
                    "interactionType",
                    &[
                        "click",
                        "double-click",
                        "right-click",
                        "middle-click",
                        "move",
                        "mouseup",
                    ][..],
                ),
                (
                    "cursorType",
                    &[
                        "arrow",
                        "text",
                        "pointer",
                        "crosshair",
                        "open-hand",
                        "closed-hand",
                        "resize-ew",
                        "resize-ns",
                        "not-allowed",
                    ][..],
                ),
            ] {
                if p[key].as_str().is_some_and(|s| allowed.contains(&s)) {
                    point[key] = p[key].clone();
                }
            }
            point
        })
        .collect();
    points.sort_by(|a, b| {
        a["timeMs"]
            .as_f64()
            .unwrap()
            .total_cmp(&b["timeMs"].as_f64().unwrap())
    });
    points
}

pub fn load(video: &Path) -> Result<Vec<Value>> {
    for path in recording_sidecars::telemetry_paths(video) {
        if recording_sidecars::regular_file(&path) {
            let text = fs::read_to_string(path)?;
            let value = serde_json::from_str(text.trim_start_matches('\u{feff}'))?;
            return Ok(normalize(&value));
        }
    }
    Ok(Vec::new())
}

pub fn save(video: &Path, raw: &Value) -> Result<Vec<Value>> {
    let points = normalize(raw);
    let [path, legacy] = recording_sidecars::telemetry_paths(video);
    if points.is_empty() {
        for path in [&path, &legacy] {
            match fs::remove_file(path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
    } else {
        files::write_json(&path, &json!({"version":2,"samples":points}))?;
    }
    Ok(points)
}
