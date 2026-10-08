use anyhow::{bail, Context, Result};
use std::{
    io::Read,
    path::Path,
    process::{Child, ChildStdout, Command, Stdio},
};
pub struct Decoder {
    child: Child,
    output: ChildStdout,
}
impl Decoder {
    pub fn start(path: &Path, start: f64, fps: f64) -> Result<Self> {
        if !start.is_finite() || start < 0. || !fps.is_finite() || !(0.1..=240.).contains(&fps) {
            bail!("Invalid decoder timing");
        }
        let mut child = Command::new("ffmpeg")
            .args(["-v", "error", "-ss", &start.to_string(), "-i"])
            .arg(path)
            .args([
                "-an",
                "-vf",
                &format!("fps={fps}"),
                "-f",
                "image2pipe",
                "-vcodec",
                "png",
                "-",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let output = child.stdout.take().context("Decoder stream unavailable")?;
        Ok(Self { child, output })
    }
    pub fn frame(&mut self) -> Result<Vec<u8>> {
        let mut signature = [0; 8];
        self.output
            .read_exact(&mut signature)
            .context("Video decoder reached the end of the source")?;
        if &signature != b"\x89PNG\r\n\x1a\n" {
            bail!("Invalid frame stream");
        }
        let mut image = signature.to_vec();
        loop {
            let mut header = [0; 8];
            self.output.read_exact(&mut header)?;
            let length = u32::from_be_bytes(header[..4].try_into().unwrap()) as usize;
            if length > 64 * 1024 * 1024 || image.len() + length > 128 * 1024 * 1024 {
                bail!("Decoded frame exceeds size limit");
            }
            image.extend(header);
            let offset = image.len();
            image.resize(offset + length + 4, 0);
            self.output.read_exact(&mut image[offset..])?;
            if &header[4..] == b"IEND" {
                return Ok(image);
            }
        }
    }
}
impl Drop for Decoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
