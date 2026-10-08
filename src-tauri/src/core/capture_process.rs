use anyhow::{bail, Result};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use std::{
    process::Child,
    time::{Duration, Instant},
};

/// Finalize every child on success and failure; never leave an unreaped encoder.
pub fn stop(child: &mut Child) -> Result<()> {
    if child.try_wait()?.is_some() {
        return Ok(());
    }
    let pid = Pid::from_raw(child.id() as i32);
    let _ = kill(pid, Signal::SIGCONT);
    let _ = kill(pid, Signal::SIGINT);
    let deadline = Instant::now() + Duration::from_secs(15);
    while child.try_wait()?.is_none() {
        if Instant::now() >= deadline {
            child.kill()?;
            child.wait()?;
            bail!("Recording finalization timed out; finalized segments remain available for recovery");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
