use super::media;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::Path;

/// Decode a bounded mono stream in FFmpeg; WebKit need not decode the video container.
pub fn peaks(path: &Path, count: usize) -> Result<Value> {
    if !(16..=8192).contains(&count) {
        bail!("Peak count must be between 16 and 8192");
    }
    let data = media::command(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-i",
            path.to_str().context("Invalid audio path")?,
            "-vn",
            "-ac",
            "1",
            "-ar",
            "1000",
            "-f",
            "f32le",
            "-",
        ],
    )?;
    let samples: Vec<f32> = data
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes(*b).abs())
        .collect();
    let mut peaks = vec![0.0_f32; count];
    for (i, sample) in samples.iter().enumerate() {
        let bin = i * count / samples.len().max(1);
        peaks[bin] = peaks[bin].max(*sample);
    }
    Ok(json!({"success":true,"peaks":peaks,"durationMs":samples.len()}))
}
