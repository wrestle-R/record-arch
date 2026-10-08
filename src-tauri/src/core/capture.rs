use super::{capture_source, files, media, recording_segments, sidecar_capture::Sidecar};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Child, time::Instant};

pub use capture_source::sources;
#[derive(Default)]
pub struct Recorder {
    pub sidecars: Vec<Sidecar>,
    pub child: Option<Child>,
    pub output: Option<PathBuf>,
    pub paused: bool,
    active_path: Option<PathBuf>,
    staging: Option<PathBuf>,
    segments: Vec<PathBuf>,
    source: String,
    options: Value,
    started: Option<Instant>,
    elapsed: f64,
    error: Option<String>,
}
impl Recorder {
    pub fn status(&mut self) -> Value {
        let failed_device = self.sidecars.iter_mut().find_map(|sidecar| {
            sidecar.child.try_wait().ok().flatten().map(|status| {
                format!(
                    "Capture device exited ({status}); inspect {}",
                    sidecar.path.with_extension("log").display()
                )
            })
        });
        if let Some(error) = failed_device {
            self.error = Some(error);
            let _ = self.finish_segment();
            self.paused = false;
        }
        let exited = self
            .child
            .as_mut()
            .and_then(|c| c.try_wait().ok().flatten());
        if let Some(status) = exited {
            self.error = Some(format!(
                "Recorder exited ({status}); finalized segments are available for recovery"
            ));
            let _ = self.finish_segment();
            self.paused = false;
        }
        let elapsed = self.elapsed
            + self
                .started
                .map(|i| i.elapsed().as_secs_f64())
                .unwrap_or(0.);
        json!({"success":true,"recording":self.child.is_some() || self.paused,"paused":self.paused,"output":self.output,"pid":self.child.as_ref().map(Child::id),"elapsed":elapsed,"error":self.error,"recoveryManifest":self.staging.as_ref().map(|p|p.join("session.json"))})
    }
    pub fn start(&mut self, source: &str, options: &Value) -> Result<Value> {
        if self.status()["recording"] == true {
            bail!("A recording is already active");
        }
        let output = options["output"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                files::root().join("recordings").join(format!(
                    "recording-{}-{}.mp4",
                    chrono::Local::now().format("%Y%m%d-%H%M%S"),
                    &uuid::Uuid::new_v4().to_string()[..8]
                ))
            });
        if output.exists() {
            bail!("Output already exists");
        }
        if output.extension().and_then(|s| s.to_str()) != Some("mp4") {
            bail!("Recording output must end in .mp4");
        }
        fs::create_dir_all(output.parent().context("Output directory missing")?)?;
        self.output = Some(output.clone());
        self.staging = Some(
            files::root()
                .join(".cache")
                .join(format!("capture-{}", uuid::Uuid::new_v4())),
        );
        fs::create_dir_all(self.staging.as_ref().unwrap())?;
        self.segments.clear();
        self.source = source.to_string();
        self.options = options.clone();
        self.elapsed = 0.;
        self.error = None;
        self.paused = false;
        self.begin_segment()?;
        Ok(
            json!({"success":true,"path":output,"sessionId":uuid::Uuid::new_v4().to_string(),"hudCaptureExclusion":false}),
        )
    }
    fn begin_segment(&mut self) -> Result<()> {
        let path = self
            .staging
            .as_ref()
            .context("Capture staging missing")?
            .join(format!("segment-{}.mp4", self.segments.len()));
        self.child = Some(capture_source::spawn(&self.source, &self.options, &path)?);
        self.active_path = Some(path.clone());
        self.started = Some(Instant::now());
        let result = (|| -> Result<()> {
            if let Some(source) = self.options["microphone"].as_str() {
                self.sidecars.push(Sidecar::microphone(&path, source)?);
            }
            if let Some(device) = self.options["webcam"].as_str() {
                self.sidecars.push(Sidecar::webcam(&path, device)?);
            }
            self.write_manifest()
        })();
        if let Err(error) = result {
            let _ = self.finish_segment();
            self.paused = false;
            return Err(error);
        }
        Ok(())
    }
    fn write_manifest(&self) -> Result<()> {
        let mut segments = self.segments.clone();
        if let Some(path) = &self.active_path {
            segments.push(path.clone());
        }
        files::write_json(
            &self
                .staging
                .as_ref()
                .context("Capture staging missing")?
                .join("session.json"),
            &json!({"version":1,"output":self.output,"segments":segments}),
        )
    }
    fn finish_segment(&mut self) -> Result<()> {
        let mut first_error = None;
        if let Some(mut child) = self.child.take() {
            if let Err(error) = super::capture_process::stop(&mut child) {
                first_error = Some(error);
            }
        }
        for sidecar in &mut self.sidecars {
            if let Err(error) = sidecar.stop() {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        self.sidecars.clear();
        if let Some(started) = self.started.take() {
            self.elapsed += started.elapsed().as_secs_f64();
        }
        if let Some(path) = self.active_path.take() {
            if media::probe(&path).is_ok() {
                self.segments.push(path);
            } else if first_error.is_none() {
                first_error = Some(anyhow::anyhow!(
                    "Recording segment is not playable; inspect {}",
                    path.with_extension("capture.log").display()
                ));
            }
        }
        self.write_manifest()?;
        first_error.map_or(Ok(()), Err)
    }
    pub fn pause(&mut self, paused: bool) -> Result<Value> {
        if self.status()["recording"] != true {
            bail!("No active recording");
        }
        if self.paused == paused {
            return Ok(json!({"success":true}));
        }
        if paused {
            self.finish_segment()?;
            self.paused = true;
        } else {
            self.begin_segment()?;
            self.paused = false;
        }
        Ok(json!({"success":true}))
    }
    pub fn stop(&mut self) -> Result<Value> {
        if self.child.is_none() && !self.paused && self.segments.is_empty() {
            bail!("No active recording");
        }
        if self.child.is_some() {
            self.finish_segment()?;
        }
        self.paused = false;
        let manifest = self
            .staging
            .as_ref()
            .context("Capture staging missing")?
            .join("session.json");
        let result = recording_segments::recover(&manifest)?;
        self.segments.clear();
        self.staging = None;
        Ok(result)
    }
}
impl Drop for Recorder {
    fn drop(&mut self) {
        if self.child.is_some() || self.paused {
            let _ = self.stop();
        }
    }
}
