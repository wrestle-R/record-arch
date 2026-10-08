use super::{files, media};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::Path;
pub fn sidecar(output: &Path, options: &Value) -> Result<()> {
    if options.is_null() {
        return Ok(());
    }
    let cues = options["cues"].as_array().context("Caption cues missing")?;
    for format in ["srt", "vtt"] {
        if options["format"] != format && options["format"] != "both" {
            continue;
        }
        let mut text = if format == "vtt" {
            "WEBVTT\n\n".to_string()
        } else {
            String::new()
        };
        for (i, cue) in cues.iter().enumerate() {
            let start = cue["startMs"].as_f64().context("Caption start missing")?;
            let end = cue["endMs"].as_f64().context("Caption end missing")?;
            if !start.is_finite() || !end.is_finite() || start < 0. || end <= start {
                bail!("Invalid caption time");
            }
            if format == "srt" {
                text.push_str(&format!("{}\n", i + 1));
            }
            text.push_str(&format!(
                "{} --> {}\n{}\n\n",
                timestamp(start, format),
                timestamp(end, format),
                cue["text"].as_str().context("Caption text missing")?
            ));
        }
        files::atomic_write(&output.with_extension(format), text.as_bytes())?;
    }
    Ok(())
}
fn timestamp(ms: f64, format: &str) -> String {
    let ms = ms.round() as u64;
    format!(
        "{:02}:{:02}:{:02}{}{:03}",
        ms / 3600000,
        ms / 60000 % 60,
        ms / 1000 % 60,
        if format == "srt" { "," } else { "." },
        ms % 1000
    )
}
pub fn generate(video: &Path, model: &Path, executable: &str, language: &str) -> Result<Value> {
    let id = uuid::Uuid::new_v4();
    let audio = files::root()
        .join(".cache")
        .join(format!("caption-{id}.wav"));
    let output = audio.with_extension("transcript");
    media::command(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-i",
            video.to_str().unwrap(),
            "-vn",
            "-ac",
            "1",
            "-ar",
            "16000",
            audio.to_str().unwrap(),
        ],
    )?;
    let result = (|| {
        media::command(
            executable,
            &[
                "-m",
                model.to_str().unwrap(),
                "-f",
                audio.to_str().unwrap(),
                "-l",
                language,
                "-oj",
                "-of",
                output.to_str().unwrap(),
            ],
        )?;
        let data = files::read_json(&output.with_extension("transcript.json"))?;
        let cues:Vec<Value>=data["transcription"].as_array().context("Whisper returned no transcription")?.iter().enumerate().map(|(i,c)|json!({"id":format!("caption-{i}"),"startMs":c["offsets"]["from"],"endMs":c["offsets"]["to"],"text":c["text"].as_str().unwrap_or_default().trim()})).collect();
        Ok(json!({"success":true,"cues":cues}))
    })();
    let _ = std::fs::remove_file(audio);
    let _ = std::fs::remove_file(output.with_extension("transcript.json"));
    result
}
