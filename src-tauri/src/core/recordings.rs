use super::{files, media, state::State};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{fs, path::Path};
pub fn list(s: &State, include_removed: bool) -> Result<Vec<Value>> {
    let removed = files::setting("removedRecordings");
    let mut entries = Vec::new();
    for entry in fs::read_dir(files::root().join("recordings"))? {
        let p = entry?.path();
        let paths = if p.is_dir() {
            fs::read_dir(p)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .collect()
        } else {
            vec![p]
        };
        for p in paths {
            if super::recording_sidecars::is_media_sidecar(&p)
                || !matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("mp4" | "mkv" | "webm" | "mov")
                )
            {
                continue;
            }
            if !include_removed
                && removed
                    .as_array()
                    .is_some_and(|paths| paths.iter().any(|v| v.as_str() == p.to_str()))
            {
                continue;
            }
            let meta = fs::metadata(&p)?;
            let created = meta
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64;
            entries.push(json!({"path":p,"name":p.file_name().unwrap().to_string_lossy(),"createdAt":created,"updatedAt":created,"bytes":meta.len(),"url":s.media_url(&p)}));
        }
    }
    entries.sort_by_key(|v| std::cmp::Reverse(v["createdAt"].as_u64().unwrap_or(0)));
    Ok(entries)
}
pub fn removed(paths: &Value, remove: bool) -> Result<()> {
    let mut values = files::setting("removedRecordings")
        .as_array()
        .cloned()
        .unwrap_or_default();
    for path in paths.as_array().context("Expected recording paths")? {
        let path = json!(files::managed(Path::new(
            path.as_str().context("Invalid recording path")?
        ))?);
        values.retain(|p| p != &path);
        if remove {
            values.push(path);
        }
    }
    files::set_setting("removedRecordings", json!(values))
}
/// Produce a shared source file, preserving timeline source offsets without changing originals.
pub fn append(s: &mut State, current: &Path, recording: &Path) -> Result<Value> {
    let current = s.readable(current)?;
    let recording = s.readable(recording)?;
    let first = media::probe(&current)?;
    let second = media::probe(&recording)?;
    let output = files::root()
        .join("recordings")
        .join(format!("sequence-{}.mp4", uuid::Uuid::new_v4()));
    let w = first["width"].as_u64().context("Width missing")?;
    let h = first["height"].as_u64().context("Height missing")?;
    let d1 = first["duration"].as_f64().unwrap();
    let d2 = second["duration"].as_f64().unwrap();
    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.args(["-v", "error", "-i"])
        .arg(current)
        .arg("-i")
        .arg(recording);
    let mut filters = Vec::new();
    for (i, meta) in [first, second].iter().enumerate() {
        filters.push(format!("[{i}:v]scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps=30,setpts=PTS-STARTPTS[v{i}]"));
        if meta["hasAudio"] == true {
            filters.push(format!(
                "[{i}:a]aresample=48000,aformat=channel_layouts=stereo,asetpts=PTS-STARTPTS[a{i}]"
            ));
        } else {
            filters.push(format!(
                "anullsrc=r=48000:cl=stereo:d={}[a{i}]",
                meta["duration"]
            ));
        }
    }
    filters.push("[v0][a0][v1][a1]concat=n=2:v=1:a=1[v][a]".into());
    let result = cmd
        .args([
            "-filter_complex",
            &filters.join(";"),
            "-map",
            "[v]",
            "-map",
            "[a]",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-movflags",
            "+faststart",
        ])
        .arg(&output)
        .output()?;
    if !result.status.success() {
        anyhow::bail!(
            "Could not append media: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    s.approve(&output)?;
    Ok(
        json!({"success":true,"value":{"path":output,"url":s.media_url(&output),"sourceStartMs":d1*1000.,"durationMs":d2*1000.,"totalDurationMs":(d1+d2)*1000.}}),
    )
}
