use anyhow::{bail, Result};
use clap::Subcommand;
use record_arch::rpc::server;
use serde_json::{json, Value};
#[derive(Subcommand)]
pub enum ExportCommand {
    Start {
        project: String,
        #[arg(long)]
        output: String,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
        #[arg(long, default_value_t = 30)]
        fps: u32,
        #[arg(long,default_value="mp4",value_parser=["mp4","gif"])]
        format: String,
        #[arg(long)]
        wait: bool,
    },
    Status {
        id: String,
    },
    Cancel {
        id: String,
    },
}
pub fn run(command: &ExportCommand) -> Result<Value> {
    Ok(match command {
        ExportCommand::Start {
            project,
            output,
            width,
            height,
            fps,
            format,
            wait,
        } => {
            let output = std::path::PathBuf::from(output);
            let output = if output.is_absolute() {
                output
            } else {
                std::env::current_dir()?.join(output)
            };
            let result = server::call(
                "arch-export-submit",
                json!([std::fs::canonicalize(project)?,{"outputPath":output,"width":width,"height":height,"frameRate":fps,"format":format}]),
            )?;
            if !wait || result["success"] != true {
                return Ok(result);
            }
            let id = result["job"]["id"].as_str().unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3600);
            loop {
                let status = server::call("arch-export-status", json!([id]))?;
                match status["job"]["status"].as_str() {
                    Some("completed") => return Ok(status),
                    Some("failed" | "cancelled") => {
                        return Ok(json!({"success":false,"job":status["job"]}))
                    }
                    _ => {}
                }
                if std::time::Instant::now() > deadline {
                    bail!("Export wait timed out; inspect job {id} with export status");
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
        ExportCommand::Status { id } => server::call("arch-export-status", json!([id]))?,
        ExportCommand::Cancel { id } => server::call("arch-export-cancel", json!([id]))?,
    })
}
