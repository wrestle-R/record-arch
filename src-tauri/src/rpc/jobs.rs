use super::{arg, string};
use crate::core::{files, state::State};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn renderer_alive(s: &mut State) -> bool {
    let alive = s
        .renderer_ready
        .is_some_and(|seen| seen.elapsed() < Duration::from_secs(30));
    if !alive {
        s.renderer_ready = None;
        let abandoned = s
            .jobs
            .values_mut()
            .filter(|j| matches!(j["status"].as_str(), Some("queued" | "running")));
        let mut had_active = false;
        for job in abandoned {
            job["status"] = "failed".into();
            job["error"] = "Desktop renderer disconnected".into();
            had_active = true;
        }
        if had_active {
            s.encoders.clear();
            s.decoders.clear();
        }
    }
    alive
}
fn submit(s: &mut State, a: &[Value]) -> Result<Value> {
    if !renderer_alive(s) {
        bail!("Open the Record Arch desktop editor before submitting a rendered export");
    }
    let project = files::read_json(Path::new(string(a, 0)?))?;
    let source = project["videoPath"]
        .as_str()
        .context("Project video missing")?;
    s.approve(Path::new(source))?;
    if let Some(audio) = project
        .pointer("/editor/audioRegions")
        .and_then(Value::as_array)
    {
        for track in audio {
            if let Some(path) = track["audioPath"].as_str() {
                s.approve(Path::new(path))?;
            }
        }
    }
    if let Some(webcam) = project
        .pointer("/editor/webcam/sourcePath")
        .and_then(Value::as_str)
    {
        s.approve(Path::new(webcam))?;
    }
    let options = arg(a, 1).clone();
    let output = options["outputPath"]
        .as_str()
        .context("Export output path missing")?;
    if Path::new(output).exists() {
        bail!("Export destination already exists");
    }
    if s.jobs
        .values()
        .any(|j| matches!(j["status"].as_str(), Some("queued" | "running")))
    {
        bail!("Another agent export is active");
    }
    let id = uuid::Uuid::new_v4().to_string();
    let job = json!({"id":id,"status":"queued","progress":0,"options":options});
    s.jobs.insert(id.clone(), job.clone());
    s.event(
        "arch-export-job",
        json!([{"id":id,"project":project,"options":options}]),
    );
    Ok(json!({"success":true,"job":job}))
}
fn complete(s: &mut State, a: &[Value]) -> Result<Value> {
    let id = string(a, 0)?;
    let result = arg(a, 1);
    let job = s.jobs.get_mut(id).context("Unknown export job")?;
    if !matches!(job["status"].as_str(), Some("queued" | "running")) {
        return Ok(json!({"success":false,"error":"Export already finished or was cancelled"}));
    }
    if result["success"] == true {
        let temp = files::managed(Path::new(
            result["tempFilePath"]
                .as_str()
                .context("Export temp path missing")?,
        ))?;
        if !temp.starts_with(files::root().join(".cache")) {
            bail!("Expected a temporary export");
        }
        let output = Path::new(job["options"]["outputPath"].as_str().unwrap()).to_path_buf();
        files::publish(&temp, &output)?;
        fs::remove_file(temp)?;
        job["status"] = "completed".into();
        job["path"] = json!(output);
        job["progress"] = 100.into();
    } else {
        job["status"] = "failed".into();
        job["error"] = result["error"].clone();
    }
    Ok(json!({"success":true,"job":job}))
}
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some(match c {
        "arch-renderer-ready" => {
            s.renderer_ready = if arg(a, 0) == false {
                None
            } else {
                Some(Instant::now())
            };
            Ok(json!({"success":true}))
        }
        "arch-export-submit" => submit(s, a),
        "arch-export-complete" => complete(s, a),
        "arch-export-list" => {
            renderer_alive(s);
            Ok(json!({"success":true,"jobs":s.jobs.values().collect::<Vec<_>>()}))
        }
        "arch-export-status" => (|| {
            renderer_alive(s);
            let job = s.jobs.get(string(a, 0)?).context("Unknown export job")?;
            Ok(json!({"success":true,"job":job}))
        })(),
        "arch-export-progress" => (|| {
            let job = s
                .jobs
                .get_mut(string(a, 0)?)
                .context("Unknown export job")?;
            if matches!(job["status"].as_str(), Some("queued" | "running")) {
                job["status"] = "running".into();
                job["progress"] = arg(a, 1).clone();
            }
            Ok(json!({"success":true}))
        })(),
        "arch-export-cancel" => (|| {
            let id = string(a, 0)?;
            let job = s.jobs.get_mut(id).context("Unknown export job")?;
            if matches!(job["status"].as_str(), Some("completed" | "failed")) {
                bail!("Export already finished");
            }
            job["status"] = "cancelled".into();
            s.event("arch-export-cancelled", json!([id]));
            Ok(json!({"success":true}))
        })(),
        _ => return None,
    })
}
