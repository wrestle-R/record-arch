use super::{arg, string};
use crate::core::{files, library, state::State};
use anyhow::{bail, Result};
use serde_json::{json, Value};
use std::{fs, path::Path};
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some((||Ok(match c {
        "get-current-video-path"|"get-recorded-video-path"=>json!({"success":s.video.is_some(),"path":s.video}),
        "get-current-recording-session"=>json!({"success":!s.session.is_null(),"session":s.session}),
        "clear-current-video-path"=>{s.video=None;s.session=Value::Null;json!({"success":true})},
        "set-current-video-path"=>{s.select_video(Path::new(string(a,0)?))?;if arg(a,1)["preserveProjectPath"]!=true{s.project=None;}json!({"success":true,"webcamPath":null})},
        "set-current-recording-session"=>{let session=arg(a,0);s.select_video(Path::new(session["videoPath"].as_str().ok_or_else(||anyhow::anyhow!("Missing videoPath"))?))?;if let Some(p)=session["webcamPath"].as_str(){s.approve(Path::new(p))?;}s.session=session.clone();json!({"success":true})},
        "arch-import"=>library::import(s,Path::new(string(a,0)?))?,
        "get-projects-directory"=>json!({"success":true,"path":files::root().join("projects"),"projectsDir":files::root().join("projects")}),
        "get-recordings-directory"=>json!({"success":true,"path":files::root().join("recordings")}),
        "list-project-files"=>json!({"success":true,"projectsDir":files::root().join("projects"),"entries":library::entries()?}),
        "load-current-project-file"=>library::latest(s)?,
        "open-project-file-at-path"=>library::load(s,Path::new(string(a,0)?))?,
        "save-project-file"=>library::save(s,arg(a,0).clone(),arg(a,1).as_str(),None)?,
        "save-project-file-named"=>library::save(s,arg(a,0).clone(),None,arg(a,1).as_str())?,
        "create-project-file"=>library::save(s,arg(a,0).clone(),None,None)?,
        "get-project-preview"=>{let project=files::read_json(Path::new(string(a,0)?))?;json!({"success":true,"project":project,"preview":project})},
        "rename-library-project"=>{let p=files::managed(Path::new(string(a,0)?))?;let target=p.with_file_name(format!("{}.recordarch",files::safe_name(string(a,1)?)));if target.exists(){bail!("Project name already exists");}fs::rename(&p,&target)?;if s.project.as_ref()==Some(&p){s.project=Some(target.clone());}json!({"success":true,"path":target})},
        "trash-project-files"|"delete-recording-file"=>{let paths=if c=="delete-recording-file"{vec![arg(a,0).clone()]}else{arg(a,0).as_array().cloned().unwrap_or_default()};let mut deleted=Vec::new();for p in paths {let path=files::managed(Path::new(p.as_str().ok_or_else(||anyhow::anyhow!("Invalid path"))?))?;let target=files::root().join(".trash").join(format!("{}-{}",uuid::Uuid::new_v4(),path.file_name().unwrap().to_string_lossy()));fs::rename(&path,target)?;deleted.push(path);}json!({"success":true,"deleted":deleted,"errors":[]})},
        "list-recordings"=>{let mut entries=Vec::new();for f in fs::read_dir(files::root().join("recordings"))? {let p=f?.path();let paths=if p.is_dir(){fs::read_dir(p)?.filter_map(|e|e.ok().map(|e|e.path())).collect()}else{vec![p]};for p in paths {if matches!(p.extension().and_then(|x|x.to_str()),Some("mp4"|"mkv"|"webm"|"mov")){entries.push(json!({"path":p,"name":p.file_name().unwrap().to_string_lossy(),"thumbnailUrl":null,"updatedAt":0,"removed":false}));}}}json!({"success":true,"entries":entries,"recordings":entries})},
        "get-local-media-url"=>{let path=s.readable(Path::new(string(a,0)?))?;json!({"success":true,"url":s.media_url(&path)})},
        "get-cursor-telemetry"=>{let p=s.readable(Path::new(string(a,0)?))?;let samples=files::read_json(&p.with_extension("cursor.json")).unwrap_or(json!([]));json!({"success":true,"samples":samples,"telemetry":samples})},
        "set-cursor-telemetry"=>{let p=s.readable(Path::new(string(a,0)?))?;files::write_json(&p.with_extension("cursor.json"),arg(a,1))?;json!({"success":true})},
        "arch-events"=>json!(std::mem::take(&mut s.events)),
        _=>return Err(anyhow::anyhow!("__unhandled"))
    }))()).filter(|r|!r.as_ref().err().is_some_and(|e|e.to_string()=="__unhandled"))
}
