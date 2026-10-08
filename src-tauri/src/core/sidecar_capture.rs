use anyhow::{bail, Context, Result};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct Sidecar {
    pub child: Child,
    pub path: PathBuf,
}
impl Sidecar {
    pub fn microphone(video: &Path, source: &str) -> Result<Self> {
        let available = super::devices::list()?;
        if !available["microphones"]
            .as_array()
            .is_some_and(|devices| devices.iter().any(|d| d["name"] == source))
        {
            bail!("Select an available microphone from the device list");
        }
        let path = video.with_extension("mic.wav");
        Self::start(
            path,
            &[
                "-f",
                "pulse",
                "-i",
                source,
                "-ac",
                "2",
                "-ar",
                "48000",
                "-c:a",
                "pcm_s16le",
            ],
        )
    }
    pub fn webcam(video: &Path, device: &str) -> Result<Self> {
        if !device
            .strip_prefix("/dev/video")
            .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        {
            bail!("Select a camera device from the device list");
        }
        Self::start(
            video.with_extension("webcam.mp4"),
            &[
                "-f",
                "v4l2",
                "-framerate",
                "30",
                "-i",
                device,
                "-an",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-pix_fmt",
                "yuv420p",
            ],
        )
    }
    fn start(path: PathBuf, args: &[&str]) -> Result<Self> {
        if path.exists() {
            bail!("Sidecar output already exists");
        }
        let log = fs::File::create(path.with_extension("log"))?;
        let mut child = Command::new("ffmpeg")
            .args(["-v", "error"])
            .args(args)
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(log)
            .spawn()
            .context("FFmpeg sidecar could not start")?;
        std::thread::sleep(Duration::from_millis(150));
        if child.try_wait()?.is_some() {
            bail!(
                "Capture device failed; see {}",
                path.with_extension("log").display()
            );
        }
        Ok(Self { child, path })
    }
    pub fn signal(&self, signal: Signal) -> Result<()> {
        kill(Pid::from_raw(self.child.id() as i32), signal)?;
        Ok(())
    }
    pub fn stop(&mut self) -> Result<()> {
        if let Some(status) = self.child.try_wait()? {
            if !status.success() {
                bail!(
                    "Capture device exited ({status}); inspect {}",
                    self.path.with_extension("log").display()
                );
            }
            return Ok(());
        }
        self.signal(Signal::SIGCONT)?;
        self.signal(Signal::SIGINT)?;
        let until = Instant::now() + Duration::from_secs(10);
        while self.child.try_wait()?.is_none() {
            if Instant::now() > until {
                self.child.kill()?;
                self.child.wait()?;
                bail!("Device finalization timed out");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }
}
impl Drop for Sidecar {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
