use super::{arg, string};
use crate::core::{export, files, media, state::State};
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Seek, SeekFrom, Write},
    path::Path,
};
fn bytes(value: &Value) -> Result<Vec<u8>> {
    if let Some(b64) = value["__bytes"].as_str() {
        return Ok(STANDARD.decode(b64)?);
    }
    if let Some(b64) = value.as_str() {
        return Ok(STANDARD.decode(b64.split(',').next_back().unwrap_or(b64))?);
    }
    value
        .as_array()
        .context("Expected binary data")?
        .iter()
        .map(|v| {
            v.as_u64()
                .filter(|b| *b < 256)
                .map(|b| b as u8)
                .context("Invalid byte")
        })
        .collect()
}
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some((|| {
        Ok(match c {
            "arch-audio-peaks" => {
                let path = s.readable(Path::new(string(a, 0)?))?;
                crate::core::waveform::peaks(&path, arg(a, 1).as_u64().unwrap_or(1000) as usize)?
            }
            "download-whisper-small-model" => crate::core::models::download()?,
            "delete-whisper-small-model" => crate::core::models::delete()?,
            "generate-auto-captions" => {
                let options = arg(a, 0);
                let video = s.readable(Path::new(
                    options["videoPath"]
                        .as_str()
                        .context("Video path missing")?,
                ))?;
                let model = s.readable(Path::new(
                    options["whisperModelPath"]
                        .as_str()
                        .context("Whisper model missing")?,
                ))?;
                crate::core::captions::generate(
                    &video,
                    &model,
                    options["whisperExecutablePath"]
                        .as_str()
                        .unwrap_or("whisper-cli"),
                    options["language"].as_str().unwrap_or("auto"),
                )?
            }
            "probe-native-video-metadata" => {
                let path = s.readable(Path::new(string(a, 0)?))?;
                json!({"success":true,"metadata":media::probe(&path)?})
            }
            "arch-decoder-open" => {
 let path=s.readable(Path::new(string(a,0)?))?;let decoder=crate::core::decoder::Decoder::start(&path,arg(a,1).as_f64().unwrap_or(0.),arg(a,2).as_f64().unwrap_or(30.))?;
 let id=uuid::Uuid::new_v4().to_string();s.decoders.insert(id.clone(),decoder);json!({"success":true,"sessionId":id})
 },
 "arch-decoder-next" => {let image=s.decoders.get_mut(string(a,0)?).context("Unknown decoder")?.frame()?;json!({"success":true,"dataUrl":format!("data:image/png;base64,{}",STANDARD.encode(image))})},
 "arch-decoder-close" => {s.decoders.remove(string(a,0)?);json!({"success":true})},
 "arch-frame" | "get-recording-thumbnail" | "generate-wallpaper-thumbnail" => {
                let path = s.readable(Path::new(string(a, 0)?))?;
                let image = media::frame(&path, arg(a, 1).as_f64().unwrap_or(0.))?;
                let url = format!("data:image/png;base64,{}", STANDARD.encode(image));
                json!({"success":true,"dataUrl":url,"thumbnailUrl":url,"value":url})
            }
            "read-local-file" => {
                let path = s.readable(Path::new(string(a, 0)?))?;
                let data = fs::read(path)?;
                if data.len() > 64 * 1024 * 1024 {
                    bail!("Use the media stream for files over 64 MiB");
                }
                json!({"success":true,"data":data})
            }
            "arch-convert-gif" => {
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
                        path.to_str().unwrap(),
                    ],
                )?;
                json!({"success":true,"path":path})
            }
            "arch-encode-start" => {
                let encoder = export::Encoder::start(arg(a, 0))?;
                let id = uuid::Uuid::new_v4().to_string();
                s.encoders.insert(id.clone(), encoder);
                json!({"success":true,"sessionId":id})
            }
            "arch-encode-frame" => {
                let encoder = s
                    .encoders
                    .get_mut(string(a, 0)?)
                    .context("Unknown encoder session")?;
                encoder.write(&bytes(arg(a, 1))?)?;
                json!({"success":true})
            }
            "arch-encode-finish" => {
                let encoder = s
                    .encoders
                    .remove(string(a, 0)?)
                    .context("Unknown encoder session")?;
                encoder.finish(arg(a, 1))?
            }
            "arch-encode-cancel" => {
                s.encoders.remove(string(a, 0)?);
                json!({"success":true})
            }
            "arch-mix-audio" => {
                for track in arg(a, 0)["tracks"].as_array().context("Missing tracks")? {
                    s.readable(Path::new(
                        track["path"].as_str().context("Missing track path")?,
                    ))?;
                }
                let path = export::mix_audio(arg(a, 0))?;
                json!({"success":true,"path":path})
            }
            "export-stream-open" => {
                let id = uuid::Uuid::new_v4().to_string();
                let ext = arg(a, 0)["extension"].as_str().unwrap_or("mp4");
                if !["mp4", "gif", "webm", "wav"].contains(&ext) {
                    bail!("Invalid stream extension");
                }
                let path = files::root()
                    .join(".cache")
                    .join(format!("stream-{id}.{ext}"));
                fs::File::create(&path)?;
                s.streams.insert(id.clone(), path.clone());
                json!({"success":true,"streamId":id,"tempPath":path})
            }
            "export-stream-write" => {
                let path = s.streams.get(string(a, 0)?).context("Unknown stream")?;
                let mut f = fs::OpenOptions::new().write(true).open(path)?;
                f.seek(SeekFrom::Start(
                    arg(a, 1).as_u64().context("Missing position")?,
                ))?;
                f.write_all(&bytes(arg(a, 2))?)?;
                json!({"success":true})
            }
            "export-stream-close" => {
                let path = s.streams.remove(string(a, 0)?).context("Unknown stream")?;
                if arg(a, 1)["abort"] == true {
                    fs::remove_file(&path)?;
                    return Ok(json!({"success":true}));
                }
                json!({"success":true,"tempPath":path,"tempFilePath":path})
            }
            "store-recorded-video" | "store-microphone-sidecar" => {
                let name = files::safe_name(arg(a, 1).as_str().unwrap_or("recording.webm"));
                let path = files::root()
                    .join("recordings")
                    .join(format!("{}-{name}", uuid::Uuid::new_v4()));
                files::atomic_write(&path, &bytes(arg(a, 0))?)?;
                if c == "store-recorded-video" {
                    s.select_video(&path)?;
                }
                json!({"success":true,"path":path})
            }
            "write-exported-video-to-path" => {
                let path = Path::new(string(a, 1)?);
                files::atomic_write(path, &bytes(arg(a, 0))?)?;
                crate::core::captions::sidecar(path, arg(a, 2))?;
                json!({"success":true,"path":path})
            }
            "finalize-exported-video" => {
                let temp = files::managed(Path::new(
                    arg(a, 0)["tempPath"].as_str().context("Missing tempPath")?,
                ))?;
                let output = arg(a, 0)["outputPath"]
                    .as_str()
                    .map(|p| Path::new(p).to_path_buf())
                    .unwrap_or_else(|| {
                        files::root().join("exports").join(files::safe_name(
                            arg(a, 0)["fileName"].as_str().unwrap_or("export.mp4"),
                        ))
                    });
                if output.exists() {
                    bail!("Export destination already exists");
                }
                fs::create_dir_all(output.parent().context("No output directory")?)?;
                files::publish(&temp, &output)?;
                crate::core::captions::sidecar(&output, &arg(a, 0)["captionSidecar"])?;
                fs::remove_file(temp)?;
                json!({"success":true,"path":output})
            }
            "discard-exported-temp" => {
                let path = files::managed(Path::new(string(a, 0)?))?;
                if !path.starts_with(files::root().join(".cache")) {
                    bail!("Not a temporary export");
                }
                fs::remove_file(path)?;
                json!({"success":true})
            }
            _ => return Err(anyhow::anyhow!("__unhandled")),
        })
    })())
    .filter(|r| {
        !r.as_ref()
            .err()
            .is_some_and(|e| e.to_string() == "__unhandled")
    })
}
