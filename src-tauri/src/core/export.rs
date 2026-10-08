use super::{files, media};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
};

pub struct Encoder {
    child: Child,
    input: Option<ChildStdin>,
    pub path: PathBuf,
    pub frames: usize,
}
impl Encoder {
    pub fn start(options: &Value) -> Result<Self> {
        let width = options["width"].as_u64().context("Export width missing")?;
        let height = options["height"]
            .as_u64()
            .context("Export height missing")?;
        let fps = options["frameRate"].as_f64().unwrap_or(30.);
        if width == 0
            || height == 0
            || width > 7680
            || height > 7680
            || !(1. ..=120.).contains(&fps)
        {
            bail!("Unsupported export dimensions or frame rate");
        }
        let path = files::root()
            .join(".cache")
            .join(format!("export-{}.mp4", uuid::Uuid::new_v4()));
        let log = fs::File::create(path.with_extension("log"))?;
        let mut child = Command::new("ffmpeg")
            .args([
                "-y",
                "-v",
                "error",
                "-f",
                "image2pipe",
                "-framerate",
                &fps.to_string(),
                "-vcodec",
                "png",
                "-i",
                "-",
                "-an",
                "-c:v",
                "libx264",
                "-preset",
                "fast",
                "-crf",
                "18",
                "-pix_fmt",
                "yuv420p",
                "-vf",
                "pad=ceil(iw/2)*2:ceil(ih/2)*2",
                "-movflags",
                "+faststart",
                path.to_str().unwrap(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()?;
        let input = child.stdin.take();
        Ok(Self {
            child,
            input,
            path,
            frames: 0,
        })
    }
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        if !data.starts_with(b"\x89PNG\r\n\x1a\n") {
            bail!("Expected a PNG frame");
        }
        self.input
            .as_mut()
            .context("Export session closed")?
            .write_all(data)?;
        self.frames += 1;
        Ok(())
    }
    pub fn finish(mut self, options: &Value) -> Result<Value> {
        self.input.take();
        let status = self.child.wait()?;
        if !status.success() {
            bail!(
                "FFmpeg encoding failed: {}",
                fs::read_to_string(self.path.with_extension("log")).unwrap_or_default()
            );
        }
        if let Some(audio) = options["audioPath"].as_str() {
            let muxed = self.path.with_extension("muxed.mp4");
            media::command(
                "ffmpeg",
                &[
                    "-y",
                    "-v",
                    "error",
                    "-i",
                    self.path.to_str().unwrap(),
                    "-i",
                    audio,
                    "-map",
                    "0:v:0",
                    "-map",
                    "1:a:0",
                    "-c:v",
                    "copy",
                    "-c:a",
                    "aac",
                    "-shortest",
                    "-movflags",
                    "+faststart",
                    muxed.to_str().unwrap(),
                ],
            )?;
            fs::rename(muxed, &self.path)?;
        }
        Ok(
            json!({"success":true,"tempPath":self.path,"tempFilePath":self.path,"encoderName":"libx264","frames":self.frames}),
        )
    }
}
impl Drop for Encoder {
    fn drop(&mut self) {
        self.input.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

pub fn mix_audio(options: &Value) -> Result<PathBuf> {
    let duration = options["duration"]
        .as_f64()
        .context("Audio duration missing")?;
    if !duration.is_finite() || duration <= 0. {
        bail!("Invalid audio duration");
    }
    let path = files::root()
        .join(".cache")
        .join(format!("audio-{}.wav", uuid::Uuid::new_v4()));
    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-v",
        "error",
        "-f",
        "lavfi",
        "-i",
        &format!("anullsrc=r=48000:cl=stereo:d={duration}"),
    ]);
    let mut filters = vec!["[0:a]anull[base]".to_string()];
    let mut labels = vec!["[base]".to_string()];
    let mut index = 1;
    for track in options["tracks"]
        .as_array()
        .context("Expected audio tracks")?
    {
        let source = track["path"].as_str().context("Audio path missing")?;
        let start = track["sourceStart"].as_f64().unwrap_or(0.);
        let end = track["sourceEnd"].as_f64().unwrap_or(duration);
        let output_start = track["outputStart"].as_f64().unwrap_or(0.);
        let speed = track["speed"].as_f64().unwrap_or(1.);
        let volume = track["volume"].as_f64().unwrap_or(1.);
        if [start, end, output_start, speed, volume]
            .iter()
            .any(|v| !v.is_finite())
            || start < 0.
            || end <= start
            || output_start < 0.
            || speed <= 0.
            || volume < 0.
        {
            bail!("Invalid audio track");
        }
        // Ignore video tracks with no audio; imported sound files are detected by ffprobe.
        let info: Value = serde_json::from_slice(&media::command(
            "ffprobe",
            &[
                "-v",
                "error",
                "-select_streams",
                "a",
                "-show_streams",
                "-of",
                "json",
                source,
            ],
        )?)?;
        if info["streams"].as_array().map_or(true, |s| s.is_empty()) {
            continue;
        }
        cmd.args(["-i", source]);
        let mut tempo = speed;
        let mut chain = Vec::new();
        while tempo > 2. {
            chain.push("atempo=2".to_string());
            tempo /= 2.;
        }
        while tempo < 0.5 {
            chain.push("atempo=0.5".to_string());
            tempo /= 0.5;
        }
        chain.push(format!("atempo={tempo}"));
        filters.push(format!("[{index}:a]atrim=start={start}:end={end},asetpts=PTS-STARTPTS,{},volume={volume},adelay={}|{}[a{index}]",chain.join(","),(output_start*1000.).round(),(output_start*1000.).round()));
        labels.push(format!("[a{index}]"));
        index += 1;
    }
    filters.push(format!(
        "{}amix=inputs={}:duration=first:normalize=0[out]",
        labels.join(""),
        labels.len()
    ));
    let result = cmd
        .args([
            "-filter_complex",
            &filters.join(";"),
            "-map",
            "[out]",
            "-t",
            &duration.to_string(),
            "-c:a",
            "pcm_s16le",
            path.to_str().unwrap(),
        ])
        .output()?;
    if !result.status.success() {
        bail!(
            "Audio mix failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(path)
}
