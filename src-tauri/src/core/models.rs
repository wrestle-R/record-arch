use super::{files, media};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
const SMALL_SHA256: &str = "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b";
fn path() -> PathBuf {
    files::root().join("models/ggml-small.bin")
}
pub fn status() -> Value {
    let path = path();
    json!({"success":true,"exists":path.is_file(),"path":path.is_file().then_some(path)})
}
pub fn download() -> Result<Value> {
    let path = path();
    if path.is_file() {
        return Ok(json!({"success":true,"path":path,"alreadyDownloaded":true}));
    }
    fs::create_dir_all(path.parent().unwrap())?;
    let partial = path.with_extension("bin.partial");
    let result = (|| {
        media::command(
            "curl",
            &[
                "--fail",
                "--location",
                "--silent",
                "--show-error",
                "--max-time",
                "900",
                "--output",
                partial.to_str().unwrap(),
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
            ],
        )?;
        let checksum = media::command("sha256sum", &[partial.to_str().unwrap()])?;
        let hash = std::str::from_utf8(&checksum)?
            .split_whitespace()
            .next()
            .context("Missing checksum")?;
        if hash != SMALL_SHA256 {
            bail!("Whisper model checksum mismatch");
        }
        fs::rename(&partial, &path)?;
        Ok(json!({"success":true,"path":path}))
    })();
    if result.is_err() {
        let _ = fs::remove_file(partial);
    }
    result
}
pub fn delete() -> Result<Value> {
    let path = path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(json!({"success":true}))
}
