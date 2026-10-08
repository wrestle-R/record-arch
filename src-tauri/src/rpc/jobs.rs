use super::{arg, string};
use crate::core::{files, state::State};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{fs, path::Path};
pub fn handle(s: &mut State, c: &str, a: &[Value]) -> Option<Result<Value>> {
    Some((||Ok(match c{
  "arch-renderer-ready"=>{s.renderer_ready=true;json!({"success":true})},
  "arch-export-submit"=>{
    if !s.renderer_ready{bail!("Open the Record Arch desktop editor before submitting a rendered export");}
    let project=files::read_json(Path::new(string(a,0)?))?;
    let source=project["videoPath"].as_str().context("Project video missing")?;
    s.approve(Path::new(source))?;
    if let Some(audio)=project.pointer("/editor/audioRegions").and_then(Value::as_array){for track in audio{if let Some(p)=track["audioPath"].as_str(){s.approve(Path::new(p))?;}}}
    if let Some(webcam)=project.pointer("/editor/webcam/sourcePath").and_then(Value::as_str){s.approve(Path::new(webcam))?;}
    let options=arg(a,1).clone();
    let output=options["outputPath"].as_str().context("Export output path missing")?;
    if Path::new(output).exists(){bail!("Export destination already exists");}
    if s.jobs.values().any(|j|matches!(j["status"].as_str(),Some("queued"|"running"))){bail!("Another agent export is active");}
    let id=uuid::Uuid::new_v4().to_string();
    let job=json!({"id":id,"status":"queued","progress":0,"options":options});
    s.jobs.insert(id.clone(),job.clone());
    s.event("arch-export-job",json!([{ "id":id,"project":project,"options":options}]));
    json!({"success":true,"job":job})
  },
  "arch-export-status"=>json!({"success":true,"job":s.jobs.get(string(a,0)?).context("Unknown export job")?}),
  "arch-export-progress"=>{let job=s.jobs.get_mut(string(a,0)?).context("Unknown export job")?;if job["status"]!="cancelled"{job["status"]="running".into();job["progress"]=arg(a,1).clone();}json!({"success":true})},
  "arch-export-cancel"=>{let id=string(a,0)?;let job=s.jobs.get_mut(id).context("Unknown export job")?;if matches!(job["status"].as_str(),Some("completed"|"failed")){bail!("Export already finished");}job["status"]="cancelled".into();s.event("arch-export-cancelled",json!([id]));json!({"success":true})},
  "arch-export-complete"=>{
    let id=string(a,0)?;let result=arg(a,1);
    let job=s.jobs.get_mut(id).context("Unknown export job")?;
    if job["status"]=="cancelled"{return Ok(json!({"success":false,"error":"Export was cancelled"}));}
    if result["success"]==true {
      let temp=files::managed(Path::new(result["tempFilePath"].as_str().context("Export temp path missing")?))?;
      if !temp.starts_with(files::root().join(".cache")){bail!("Expected a temporary export");}
      let output=Path::new(job["options"]["outputPath"].as_str().unwrap()).to_path_buf();
      if output.exists(){bail!("Export destination already exists");}
      fs::create_dir_all(output.parent().context("Output parent missing")?)?;
      // create_new prevents overwriting a file created while frames were rendering.
      let mut destination=fs::OpenOptions::new().write(true).create_new(true).open(&output)?;
      std::io::copy(&mut fs::File::open(&temp)?,&mut destination)?;destination.sync_all()?;
      fs::remove_file(temp)?;
      job["status"]="completed".into();job["path"]=json!(output);job["progress"]=100.into();
    }else{job["status"]="failed".into();job["error"]=result["error"].clone();}
    json!({"success":true,"job":job})
  },
  _=>return Err(anyhow::anyhow!("__unhandled"))
 }))()).filter(|r|!r.as_ref().err().is_some_and(|e|e.to_string()=="__unhandled"))
}
