use super::{files, state::State};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{fs, path::Path};

pub fn entries() -> Result<Vec<Value>> {
    let mut entries = Vec::new();
    for file in fs::read_dir(files::root().join("projects"))? {
        let path = file?.path();
        if !matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("recordarch" | "recordly")
        ) {
            continue;
        }
        if let Ok(project) = files::read_json(&path) {
            let modified = fs::metadata(&path)?
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64;
            entries.push(json!({"path":path,"name":path.file_stem().unwrap_or_default().to_string_lossy(),"updatedAt":modified,"modifiedAt":modified,"videoPath":project["videoPath"],"thumbnailUrl":null,"projectId":project["projectId"]}));
        }
    }
    entries.sort_by_key(|v| std::cmp::Reverse(v["updatedAt"].as_u64().unwrap_or(0)));
    Ok(entries)
}
pub fn load(state: &mut State, path: &Path) -> Result<Value> {
    let project = files::read_json(path)?;
    if !project.is_object() || !project["version"].is_number() || !project["videoPath"].is_string()
    {
        bail!("Invalid project: expected version and videoPath");
    }
    let video = Path::new(project["videoPath"].as_str().unwrap());
    state.select_video(video)?;
    if let Some(webcam) = project
        .pointer("/editor/webcam/sourcePath")
        .and_then(Value::as_str)
    {
        state.approve(Path::new(webcam))?;
        state.session["webcamPath"] = webcam.into();
    }
    if let Some(clips) = project
        .pointer("/editor/clipRegions")
        .and_then(Value::as_array)
    {
        for clip in clips {
            if let Some(p) = clip["sourcePath"].as_str() {
                let _ = state.approve(Path::new(p));
            }
        }
    }
    if let Some(audio) = project
        .pointer("/editor/audioRegions")
        .and_then(Value::as_array)
    {
        for a in audio {
            if let Some(p) = a["audioPath"].as_str() {
                let _ = state.approve(Path::new(p));
            }
        }
    }
    state.project = Some(path.canonicalize()?);
    files::set_setting("lastProject", json!(state.project))?;
    Ok(json!({"success":true,"project":project,"path":state.project}))
}
pub fn save(
    state: &mut State,
    mut data: Value,
    path: Option<&str>,
    name: Option<&str>,
) -> Result<Value> {
    if !data.is_object() || !data["videoPath"].is_string() {
        bail!("Project must contain a videoPath");
    }
    if data["projectId"].is_null() {
        data["projectId"] = uuid::Uuid::new_v4().to_string().into();
    }
    let path = if let Some(path) = path {
        Path::new(path).to_path_buf()
    } else if let Some(name) = name {
        files::root()
            .join("projects")
            .join(format!("{}.recordarch", files::safe_name(name)))
    } else if let Some(path) = &state.project {
        path.clone()
    } else {
        files::root().join("projects").join(format!(
            "{}-{}.recordarch",
            files::safe_name(
                Path::new(data["videoPath"].as_str().unwrap())
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref()
            ),
            &uuid::Uuid::new_v4().to_string()[..8]
        ))
    };
    if path.exists() {
        let old = files::read_json(&path)?;
        if old["projectId"] != data["projectId"] {
            bail!("A different project already uses this name");
        }
        files::atomic_write(&path.with_extension("recordarch.bak"), &fs::read(&path)?)?;
    }
    files::write_json(&path, &data)?;
    state.project = Some(path.clone());
    files::set_setting("lastProject", json!(path))?;
    Ok(
        json!({"success":true,"path":path,"project":data,"projectId":data["projectId"],"projectName":path.file_stem().unwrap_or_default().to_string_lossy()}),
    )
}
pub fn latest(state: &mut State) -> Result<Value> {
    if let Some(path) = state.project.clone() {
        return load(state, &path);
    }
    if state.video.is_some() {
        return Ok(json!({"success":false}));
    }
    let saved = files::setting("lastProject");
    if let Some(path) = saved.as_str() {
        if let Ok(value) = load(state, Path::new(path)) {
            return Ok(value);
        }
    }
    for entry in entries()? {
        if let Some(path) = entry["path"].as_str() {
            if let Ok(value) = load(state, Path::new(path)) {
                return Ok(value);
            }
        }
    }
    Ok(json!({"success":false}))
}
pub fn import(state: &mut State, path: &Path) -> Result<Value> {
    let source = state.approve(path)?;
    let parent = files::root()
        .join("recordings")
        .join(uuid::Uuid::new_v4().to_string());
    fs::create_dir_all(&parent)?;
    let target = parent.join(source.file_name().context("Missing file name")?);
    fs::copy(source, &target)?;
    state.select_video(&target)?;
    state.project = None;
    Ok(json!({"success":true,"path":target,"kind":"video"}))
}
