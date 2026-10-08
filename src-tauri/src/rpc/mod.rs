mod background;
pub mod capture;
pub mod jobs;
pub mod library;
pub mod media;
pub mod server;
pub mod settings;
use crate::core::state::Shared;
use anyhow::Result;
use serde_json::{json, Value};

pub fn dispatch(state: &Shared, channel: &str, args: &[Value]) -> Result<Value> {
    if let Some(result) = background::handle(state, channel, args) {
        return result;
    }
    let mut state = state
        .lock()
        .map_err(|_| anyhow::anyhow!("Application state unavailable"))?;
    for handler in [
        jobs::handle,
        library::handle,
        settings::handle,
        media::handle,
        capture::handle,
    ] {
        if let Some(result) = handler(&mut state, channel, args) {
            return result;
        }
    }
    Ok(
        json!({"success":false,"code":"UNSUPPORTED_CAPABILITY","error":format!("{channel} has not been ported to this Linux backend"),"message":format!("{channel} is unavailable in Record Arch")}),
    )
}
pub fn arg(args: &[Value], i: usize) -> &Value {
    args.get(i).unwrap_or(&Value::Null)
}
pub fn string(args: &[Value], i: usize) -> Result<&str> {
    arg(args, i)
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Expected string argument {i}"))
}
