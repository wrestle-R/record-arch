use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{path::Path, process::Command};

pub fn command(program: &str, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("Install {program} to use this feature"))?;
    if !output.status.success() {
        bail!("{program}: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(output.stdout)
}
pub fn probe(path: &Path) -> Result<Value> {
    let output = command(
        "ffprobe",
        &[
            "-v",
            "error",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            path.to_str().context("Invalid path")?,
        ],
    )?;
    let data: Value = serde_json::from_slice(&output)?;
    let stream = data["streams"]
        .as_array()
        .context("No streams")?
        .iter()
        .find(|s| s["codec_type"] == "video")
        .context("No video stream")?;
    let fps = stream["avg_frame_rate"]
        .as_str()
        .unwrap_or("30/1")
        .split('/')
        .map(|s| s.parse::<f64>().unwrap_or(1.))
        .collect::<Vec<_>>();
    let duration = data["format"]["duration"]
        .as_str()
        .unwrap_or("0")
        .parse::<f64>()?;
    let audio = data["streams"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["codec_type"] == "audio");
    Ok(
        json!({"width":stream["width"],"height":stream["height"],"duration":duration,"frameRate":fps[0]/fps.get(1).copied().unwrap_or(1.).max(1.),"codec":stream["codec_name"],"hasAudio":audio.is_some(),"audioCodec":audio.map(|s|s["codec_name"].clone()),"mediaStartTime":0,"streamStartTime":0,"streamDuration":duration}),
    )
}
pub fn frame(path: &Path, seconds: f64) -> Result<Vec<u8>> {
    if !seconds.is_finite() || seconds < 0. {
        bail!("Invalid frame timestamp");
    }
    command(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-ss",
            &seconds.to_string(),
            "-i",
            path.to_str().context("Invalid path")?,
            "-frames:v",
            "1",
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "-",
        ],
    )
}
pub fn available(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}
pub fn doctor() -> Value {
    json!({"success":true,"platform":"linux","session":std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),"storage":super::files::root(),"ffmpeg":available("ffmpeg"),"ffprobe":available("ffprobe"),"wfRecorder":available("wf-recorder"),"hyprland":std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some(),"hudCaptureExclusion":false})
}
