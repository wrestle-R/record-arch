use super::{files, media};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn combine(paths: &[PathBuf], destination: &Path, staging: &Path) -> Result<()> {
    if paths.len() == 1 {
        return files::publish(&paths[0], destination);
    }
    let list = staging.join("concat.txt");
    let contents = paths
        .iter()
        .map(|p| format!("file '{}'\n", p.to_string_lossy().replace('\'', "'\\''")))
        .collect::<String>();
    files::atomic_write(&list, contents.as_bytes())?;
    let temporary = staging.join(format!(
        "combined.{}",
        destination
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("mp4")
    ));
    media::command(
        "ffmpeg",
        &[
            "-y",
            "-v",
            "error",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            list.to_str().context("Invalid concat path")?,
            "-c",
            "copy",
            temporary.to_str().context("Invalid output path")?,
        ],
    )?;
    files::publish(&temporary, destination)
}
pub fn recover(manifest: &Path) -> Result<Value> {
    let manifest = files::managed(manifest)?;
    let staging = manifest.parent().context("Manifest directory missing")?;
    if !staging.starts_with(files::root().join(".cache")) {
        bail!("Expected a capture manifest in the cache");
    }
    let data = files::read_json(&manifest)?;
    let output = PathBuf::from(data["output"].as_str().context("Manifest output missing")?);
    let paths = data["segments"]
        .as_array()
        .context("Manifest segments missing")?
        .iter()
        .map(|v| {
            let path = files::managed(Path::new(v.as_str().context("Invalid segment")?))?;
            if !path.starts_with(staging) {
                bail!("Segment is outside its capture session");
            }
            media::probe(&path)?;
            Ok(path)
        })
        .collect::<Result<Vec<_>>>()?;
    if paths.is_empty() {
        bail!("No playable capture segments");
    }
    // Publish sidecars first. Keep the manifest and source segments if any step fails.
    for extension in ["mic.wav", "webcam.mp4"] {
        let sidecars = paths
            .iter()
            .map(|p| p.with_extension(extension))
            .collect::<Vec<_>>();
        if sidecars.iter().any(|p| p.is_file()) {
            if !sidecars.iter().all(|p| p.is_file()) {
                bail!("A capture sidecar is missing; retain this session for recovery");
            }
            let destination = output.with_extension(extension);
            if !destination.exists() {
                combine(&sidecars, &destination, staging)?;
            }
        }
    }
    combine(&paths, &output, staging)?;
    media::probe(&output)?;
    fs::remove_dir_all(staging)?;
    Ok(
        json!({"success":true,"path":output,"videoPath":output,"webcamPath":output.with_extension("webcam.mp4").is_file().then(||output.with_extension("webcam.mp4"))}),
    )
}
