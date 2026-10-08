use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub fn root() -> PathBuf {
    std::env::var_os("RECORD_ARCH_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("Home directory unavailable")
                .join("Record-Arch")
        })
}
pub fn init() -> Result<()> {
    for dir in ["projects", "recordings", "exports", ".cache", ".trash"] {
        fs::create_dir_all(root().join(dir))?;
    }
    Ok(())
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("File must have a parent")?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
pub fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(
        fs::read_to_string(path)?.trim_start_matches('\u{feff}'),
    )?)
}
pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    atomic_write(path, &serde_json::to_vec_pretty(value)?)
}
pub fn safe_name(name: &str) -> String {
    let result: String = name
        .chars()
        .filter(|c| !c.is_control() && !"/\\:*?\"<>|".contains(*c))
        .collect();
    let result = result.trim().trim_matches('.');
    if result.is_empty() {
        "Untitled".into()
    } else {
        result.chars().take(100).collect()
    }
}
pub fn managed(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize()?;
    if !path.starts_with(root().canonicalize()?) {
        bail!("Operation restricted to the Record Arch library");
    }
    Ok(path)
}
pub fn setting(key: &str) -> Value {
    read_json(&root().join("settings.json"))
        .ok()
        .and_then(|v| v.get(key).cloned())
        .unwrap_or(Value::Null)
}
pub fn set_setting(key: &str, value: Value) -> Result<()> {
    let path = root().join("settings.json");
    let mut settings = read_json(&path).unwrap_or_else(|_| serde_json::json!({}));
    settings[key] = value;
    write_json(&path, &settings)
}
