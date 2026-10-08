use super::media;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
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
pub fn spawn(source: &str, options: &Value, output: &Path) -> Result<Child> {
    let mut command = Command::new("wf-recorder");
    command.args([
        "-f",
        output.to_str().context("Invalid output path")?,
        "--no-damage",
        "-r",
        "30",
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
    Ok(child)
}
