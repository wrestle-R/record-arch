use super::{files, media};
use anyhow::{bail, Context, Result};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
#[derive(Default)]
pub struct Recorder {
    pub child: Option<Child>,
    pub output: Option<PathBuf>,
    pub paused: bool,
    pub started: Option<Instant>,
}
pub fn sources() -> Result<Value> {
    let monitors: Value = serde_json::from_slice(&media::command("hyprctl", &["monitors", "-j"])?)?;
    let mut list = Vec::new();
    for monitor in monitors.as_array().context("Invalid monitor response")? {
        list.push(json!({"id":format!("screen:{}",monitor["name"].as_str().unwrap_or_default()),"name":monitor["name"],"sourceType":"screen","display_id":monitor["id"].to_string(),"thumbnail":"","bounds":monitor}));
    }
    let windows: Value = serde_json::from_slice(&media::command("hyprctl", &["clients", "-j"])?)?;
    for window in windows.as_array().context("Invalid window response")? {
        list.push(json!({"id":format!("window:{}",window["address"].as_str().unwrap_or_default()),"name":window["title"],"appName":window["class"],"sourceType":"window","bounds":window,"thumbnail":""}));
    }
    Ok(json!(list))
}
impl Recorder {
    pub fn status(&mut self) -> Value {
        if let Some(child) = &mut self.child {
            if child.try_wait().ok().flatten().is_some() {
                self.child = None;
            }
        }
        json!({"success":true,"recording":self.child.is_some(),"paused":self.paused,"output":self.output,"pid":self.child.as_ref().map(Child::id),"elapsed":self.started.map(|i|i.elapsed().as_secs_f64()).unwrap_or(0.)})
    }
    pub fn start(&mut self, source: &str, options: &Value) -> Result<Value> {
        if self.status()["recording"] == true {
            bail!("A recording is already active");
        }
        let output = if let Some(p) = options["output"].as_str() {
            PathBuf::from(p)
        } else {
            files::root().join("recordings").join(format!(
                "recording-{}.mp4",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            ))
        };
        if output.exists() {
            bail!("Output already exists");
        }
        fs::create_dir_all(output.parent().context("Output directory missing")?)?;
        let mut command = Command::new("wf-recorder");
        command.args([
            "-f",
            output.to_str().context("Invalid output path")?,
            "-c",
            "libx264",
            "-p",
            "preset=ultrafast",
        ]);
        if let Some(monitor) = source.strip_prefix("screen:") {
            command.args(["-o", monitor]);
        } else if source.starts_with("window:") {
            let selected = sources()?
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == source)
                .cloned()
                .context("Window is no longer available")?;
            let b = &selected["bounds"];
            let x = b["at"][0].as_i64().context("Window position unavailable")?;
            let y = b["at"][1].as_i64().context("Window position unavailable")?;
            let w = b["size"][0].as_i64().context("Window size unavailable")?;
            let h = b["size"][1].as_i64().context("Window size unavailable")?;
            command.args(["-g", &format!("{x},{y} {w}x{h}")]);
        } else {
            bail!("Select a screen or window from sources list");
        }
        if options["systemAudio"].as_bool().unwrap_or(false) {
            command.arg("--audio");
        }
        if options["hideCursor"].as_bool().unwrap_or(false) {
            command.arg("--no-cursor");
        }
        let log = fs::File::create(output.with_extension("capture.log"))?;
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(log);
        let mut child = command.spawn().context("wf-recorder could not start")?;
        std::thread::sleep(Duration::from_millis(350));
        if let Some(status) = child.try_wait()? {
            bail!(
                "Recorder exited ({status}); inspect {}",
                output.with_extension("capture.log").display()
            );
        }
        self.child = Some(child);
        self.output = Some(output.clone());
        self.started = Some(Instant::now());
        self.paused = false;
        Ok(
            json!({"success":true,"path":output,"sessionId":uuid::Uuid::new_v4().to_string(),"hudCaptureExclusion":false}),
        )
    }
    pub fn pause(&mut self, paused: bool) -> Result<Value> {
        let child = self.child.as_ref().context("No active recording")?;
        kill(
            Pid::from_raw(child.id() as i32),
            if paused {
                Signal::SIGSTOP
            } else {
                Signal::SIGCONT
            },
        )?;
        self.paused = paused;
        Ok(json!({"success":true}))
    }
    pub fn stop(&mut self) -> Result<Value> {
        let mut child = self.child.take().context("No active recording")?;
        if self.paused {
            kill(Pid::from_raw(child.id() as i32), Signal::SIGCONT)?;
        }
        kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)?;
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if child.try_wait()?.is_some() {
                break;
            }
            if Instant::now() > deadline {
                child.kill()?;
                child.wait()?;
                bail!("Recording finalization timed out; media may need recovery");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        self.paused = false;
        let output = self.output.clone().context("Recording path missing")?;
        media::probe(&output)?;
        Ok(json!({"success":true,"path":output,"videoPath":output}))
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        if self.child.is_some() {
            let _ = self.stop();
        }
    }
}
