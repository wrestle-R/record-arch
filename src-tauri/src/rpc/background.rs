//! Long media operations validate inputs under the state lock, then release it.
//! Recording controls remain responsive while FFmpeg or Whisper works.
use super::{arg, string};
use crate::core::{captions, export, files, media, models, state::Shared, waveform};
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn readable(state: &Shared, path: &str) -> Result<PathBuf> {
    state
        .lock()
        .map_err(|_| anyhow::anyhow!("Application state unavailable"))?
        .readable(Path::new(path))
}
pub fn handle(state: &Shared, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some(match c {
        "open-projects-directory" | "open-recordings-folder" => {
            let folder = files::root().join(if c == "open-projects-directory" {
                "projects"
            } else {
                "recordings"
            });
            media::command("xdg-open", &[folder.to_string_lossy().as_ref()])
                .map(|_| json!({"success":true}))
        }
        "reveal-in-folder" => (|| {
            let path = readable(state, string(a, 0)?)?;
            media::command(
                "xdg-open",
                &[path
                    .parent()
                    .context("Parent folder missing")?
                    .to_string_lossy()
                    .as_ref()],
            )?;
            Ok(json!({"success":true}))
        })(),
        "open-external-url" => (|| {
            let url = url::Url::parse(string(a, 0)?)?;
            if !matches!(url.scheme(), "http" | "https") {
                bail!("Only HTTP and HTTPS links can be opened");
            }
            media::command("xdg-open", &[url.as_str()])?;
            Ok(json!({"success":true}))
        })(),
        "download-whisper-small-model" => models::download(),
        "delete-whisper-small-model" => models::delete(),
        "arch-encode-finish" => (|| {
            let encoder = state
                .lock()
                .map_err(|_| anyhow::anyhow!("Application state unavailable"))?
                .encoders
                .remove(string(a, 0)?)
                .context("Unknown encoder session")?;
            if let Some(path) = arg(a, 1)["audioPath"].as_str() {
                readable(state, path)?;
            }
            encoder.finish(arg(a, 1))
        })(),
        "probe-native-video-metadata" => (|| {
            let path = readable(state, string(a, 0)?)?;
            Ok(json!({"success":true,"metadata":media::probe(&path)?}))
        })(),
        "arch-frame" | "get-recording-thumbnail" | "generate-wallpaper-thumbnail" => (|| {
            let path = readable(state, string(a, 0)?)?;
            let bytes = media::frame(&path, arg(a, 1).as_f64().unwrap_or(0.))?;
            let url = format!("data:image/png;base64,{}", STANDARD.encode(bytes));
            Ok(json!({"success":true,"dataUrl":url,"thumbnailUrl":url,"value":url}))
        })(),
        "read-local-file" => (|| {
            let path = readable(state, string(a, 0)?)?;
            if fs::metadata(&path)?.len() > 64 * 1024 * 1024 {
                bail!("Use the media stream for files over 64 MiB");
            }
            Ok(json!({"success":true,"data":fs::read(path)?}))
        })(),
        "arch-audio-peaks" => (|| {
            let path = readable(state, string(a, 0)?)?;
            waveform::peaks(&path, arg(a, 1).as_u64().unwrap_or(1000) as usize)
        })(),
        "arch-mix-audio" => (|| {
            for track in arg(a, 0)["tracks"].as_array().context("Missing tracks")? {
                readable(state, track["path"].as_str().context("Missing track path")?)?;
            }
            let path = export::mix_audio(arg(a, 0))?;
            Ok(json!({"success":true,"path":path}))
        })(),
        "generate-auto-captions" => (|| {
            let options = arg(a, 0);
            let video = readable(
                state,
                options["videoPath"]
                    .as_str()
                    .context("Video path missing")?,
            )?;
            let model = readable(
                state,
                options["whisperModelPath"]
                    .as_str()
                    .context("Whisper model missing")?,
            )?;
            captions::generate(
                &video,
                &model,
                options["whisperExecutablePath"]
                    .as_str()
                    .unwrap_or("whisper-cli"),
                options["language"].as_str().unwrap_or("auto"),
            )
        })(),
        "arch-convert-gif" => (|| {
            let source = files::managed(Path::new(string(a, 0)?))?;
            let path = files::root()
                .join(".cache")
                .join(format!("export-{}.gif", uuid::Uuid::new_v4()));
            media::command(
                "ffmpeg",
                &[
                    "-v",
                    "error",
                    "-i",
                    source.to_str().context("Invalid path")?,
                    "-filter_complex",
                    "[0:v]split[a][b];[a]palettegen[p];[b][p]paletteuse",
                    "-loop",
                    if arg(a, 1)["loop"] == true { "0" } else { "-1" },
                    path.to_str().context("Invalid path")?,
                ],
            )?;
            Ok(json!({"success":true,"path":path}))
        })(),
        _ => return None,
    })
}
