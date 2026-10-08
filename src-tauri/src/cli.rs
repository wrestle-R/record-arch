mod cli_export;
use clap::{Parser, Subcommand};
use record_arch::{
    core::{files, media, state::State},
    rpc::server,
};
use serde_json::{json, Value};
#[derive(Parser)]
#[command(
    name = "record-arch",
    version,
    about = "Record Arch: local recording and editing for humans and agents"
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    Export {
        #[command(subcommand)]
        command: cli_export::ExportCommand,
    },
    Recorder,
    Devices,
    Doctor,
    Serve,
    Sources {
        #[command(subcommand)]
        command: SourceCommand,
    },
    Record {
        #[command(subcommand)]
        command: RecordCommand,
    },
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    Editor {
        #[arg(value_name = "VIDEO_OR_PROJECT")]
        path: Option<String>,
    },
    Rpc {
        channel: String,
        #[arg(default_value = "[]")]
        args: String,
    },
}
#[derive(Subcommand)]
enum SourceCommand {
    List,
}
#[derive(Subcommand)]
enum RecordCommand {
    Start {
        #[arg(long)]
        source: String,
        #[arg(long)]
        output: Option<String>,
        #[arg(long)]
        system_audio: bool,
        #[arg(long)]
        hide_cursor: bool,
        #[arg(long)]
        microphone: Option<String>,
        #[arg(long)]
        webcam: Option<String>,
    },
    Status,
    Stop,
    Pause,
    Resume,
}
#[derive(Subcommand)]
enum ProjectCommand {
    List,
    Inspect {
        path: String,
    },
    Create {
        #[arg(long)]
        video: String,
        #[arg(long)]
        name: Option<String>,
    },
    Apply {
        path: String,
        #[arg(long)]
        edits: String,
    },
}
fn run(cli: &Cli) -> anyhow::Result<Value> {
    Ok(match &cli.command {
        Commands::Export { command } => cli_export::run(command)?,
        Commands::Recorder => server::call("arch-show-recorder", json!([]))?,
        Commands::Devices => server::call("arch-devices", json!([]))?,
        Commands::Doctor => media::doctor(),
        Commands::Serve => {
            let state = State::new()?;
            server::start(&state)?;
            println!("{}", json!({"success":true,"socket":server::socket_path()}));
            loop {
                std::thread::park();
            }
        }
        Commands::Sources {
            command: SourceCommand::List,
        } => server::call("arch-sources", json!([]))?,
        Commands::Record { command } => match command {
            RecordCommand::Start {
                source,
                output,
                system_audio,
                hide_cursor,
                microphone,
                webcam,
            } => server::call(
                "arch-record-start",
                json!([source,{"output":output,"systemAudio":system_audio,"hideCursor":hide_cursor,"microphone":microphone,"webcam":webcam}]),
            )?,
            RecordCommand::Status => server::call("arch-record-status", json!([]))?,
            RecordCommand::Stop => server::call("arch-record-stop", json!([]))?,
            RecordCommand::Pause => server::call("arch-record-pause", json!([]))?,
            RecordCommand::Resume => server::call("arch-record-resume", json!([]))?,
        },
        Commands::Project { command } => match command {
            ProjectCommand::List => server::call("list-project-files", json!([]))?,
            ProjectCommand::Inspect { path } => {
                json!({"success":true,"project":files::read_json(std::path::Path::new(path))?})
            }
            ProjectCommand::Create { video, name } => server::call(
                "save-project-file-named",
                json!([{"version":2,"videoPath":std::fs::canonicalize(video)?,"editor":{}},name]),
            )?,
            ProjectCommand::Apply { path, edits } => {
                let mut data = files::read_json(std::path::Path::new(path))?;
                let patch = files::read_json(std::path::Path::new(edits))?;
                if !patch.is_object() {
                    anyhow::bail!("Edits must be an object of editor properties");
                }
                for (k, v) in patch.as_object().unwrap() {
                    data["editor"][k] = v.clone();
                }
                server::call("save-project-file", json!([data, null, path]))?
            }
        },
        Commands::Editor { path } => {
            if let Some(path) = path {
                if matches!(
                    std::path::Path::new(path)
                        .extension()
                        .and_then(|s| s.to_str()),
                    Some("recordarch" | "recordly")
                ) {
                    server::call("open-project-file-at-path", json!([path]))?;
                } else {
                    server::call("arch-import", json!([path]))?;
                }
            }
            server::call("arch-open-editor", json!([]))?
        }
        Commands::Rpc { channel, args } => server::call(channel, serde_json::from_str(args)?)?,
    })
}
fn main() {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(value) => {
            println!(
                "{}",
                if cli.json {
                    value.to_string()
                } else {
                    serde_json::to_string_pretty(&value).unwrap()
                }
            );
            if value["success"] == false {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!(
                "{}",
                json!({"success":false,"error":error.to_string(),"code":"COMMAND_FAILED"})
            );
            std::process::exit(1);
        }
    }
}
