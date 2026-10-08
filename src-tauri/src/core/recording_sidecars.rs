//! Exact adjacent filenames used by Recordly and Record Arch recording bundles.
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn telemetry_paths(video: &Path) -> [PathBuf; 2] {
    let mut appended = video.as_os_str().to_os_string();
    appended.push(".cursor.json");
    [PathBuf::from(appended), video.with_extension("cursor.json")]
}

fn candidates(video: &Path) -> Vec<PathBuf> {
    let mut paths = telemetry_paths(video).to_vec();
    let media = audio_paths(video).into_iter().chain(webcam_paths(video));
    for path in media {
        let mut metadata = path.as_os_str().to_os_string();
        metadata.push(".json");
        paths.push(path);
        paths.push(PathBuf::from(metadata));
    }
    for ext in ["recordly-session.json", "recording-session.json"] {
        paths.push(video.with_extension(ext));
    }
    paths
}

fn audio_paths(video: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for kind in ["system", "mic", "microphone", "audio"] {
        for ext in ["wav", "webm", "mp4", "m4a"] {
            paths.push(video.with_extension(format!("{kind}.{ext}")));
        }
    }
    paths
}

fn webcam_paths(video: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for ext in ["mp4", "webm", "mov", "mkv", "avi"] {
        paths.push(video.with_extension(format!("webcam.{ext}")));
        let mut name = video.file_stem().unwrap_or_default().to_os_string();
        name.push(format!("-webcam.{ext}"));
        paths.push(video.with_file_name(name));
    }
    paths
}

/// Do not follow adjacent symlinks into files outside the opened recording bundle.
pub fn regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_file())
}

pub fn copy(source: &Path, target: &Path) -> Result<()> {
    for (source, target) in candidates(source).into_iter().zip(candidates(target)) {
        if regular_file(&source) {
            fs::copy(source, target)?;
        }
    }
    Ok(())
}

pub fn webcam(video: &Path) -> Option<PathBuf> {
    webcam_paths(video).into_iter().find(|p| regular_file(p))
}

pub fn is_webcam(path: &Path) -> bool {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    stem.ends_with(".webcam") || stem.ends_with("-webcam")
}

pub fn is_media_sidecar(path: &Path) -> bool {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    is_webcam(path)
        || [".system", ".mic", ".microphone", ".audio"]
            .iter()
            .any(|kind| stem.ends_with(kind))
}

pub fn audio(video: &Path) -> Vec<PathBuf> {
    audio_paths(video)
        .into_iter()
        .filter(|p| regular_file(p))
        .collect()
}
