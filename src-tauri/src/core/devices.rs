use super::media;
use anyhow::Result;
use serde_json::{json, Value};
use std::{fs, path::Path};
pub fn list() -> Result<Value> {
    let microphones = media::command("pactl", &["--format=json", "list", "sources"])
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .unwrap_or(json!([]));
    let mut cameras = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/video4linux") {
        for entry in entries.flatten() {
            let id = format!("/dev/{}", entry.file_name().to_string_lossy());
            let name = fs::read_to_string(entry.path().join("name")).unwrap_or(id.clone());
            cameras.push(json!({"id":id,"name":name.trim()}));
        }
    }
    Ok(json!({"success":true,"microphones":microphones,"cameras":cameras}))
}
pub fn audio_sidecars(video: &Path) -> Vec<String> {
    super::recording_sidecars::audio(video)
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}
