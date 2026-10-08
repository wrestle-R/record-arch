use record_arch::{
    core::{files, state::State},
    rpc,
};
use serde_json::json;
#[test]
fn imported_media_survives_save_reopen_and_library_removal() {
    let root = std::env::temp_dir().join(format!("record-arch-test-{}", uuid::Uuid::new_v4()));
    std::env::set_var("RECORD_ARCH_HOME", &root);
    let source = root.join("input.mp4");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(&source, b"fixture media").unwrap();
    let cursor_path = std::path::PathBuf::from(format!("{}.cursor.json", source.display()));
    files::write_json(
        &cursor_path,
        &json!({"version":2,"samples":[
            {"timeMs":200,"cx":2,"cy":-1,"interactionType":"click","cursorType":"pointer"},
            {"timeMs":-10,"interactionType":"invalid","cursorType":"invalid"},
            null
        ]}),
    )
    .unwrap();
    let microphone = source.with_extension("microphone.wav");
    let webcam = root.join("input-webcam.mp4");
    std::fs::write(&microphone, b"audio sidecar").unwrap();
    std::fs::write(
        source.with_extension("system.mp4"),
        b"audio-only video sidecar",
    )
    .unwrap();
    std::fs::write(&webcam, b"webcam sidecar").unwrap();
    std::fs::write(root.join("input-other.mp4"), b"unrelated recording").unwrap();
    std::os::unix::fs::symlink(&source, source.with_extension("audio.m4a")).unwrap();
    let state = State::new().unwrap();
    state.lock().unwrap().media_base = "http://127.0.0.1:1".into();
    let imported = rpc::dispatch(&state, "arch-import", &[json!(source)]).unwrap();
    assert_eq!(imported["success"], true);
    let imported_path = std::path::Path::new(imported["path"].as_str().unwrap());
    assert_eq!(
        std::fs::read(imported_path.with_extension("microphone.wav")).unwrap(),
        b"audio sidecar"
    );
    assert!(!imported_path.with_extension("audio.m4a").exists());
    assert!(!imported_path
        .parent()
        .unwrap()
        .join("input-other.mp4")
        .exists());
    assert!(cursor_path.is_file() && microphone.is_file() && webcam.is_file());
    let session = rpc::dispatch(&state, "get-current-recording-session", &[]).unwrap();
    assert_eq!(
        std::fs::read(session["session"]["webcamPath"].as_str().unwrap()).unwrap(),
        b"webcam sidecar"
    );
    let telemetry = rpc::dispatch(&state, "get-cursor-telemetry", &[]).unwrap();
    assert_eq!(
        telemetry["samples"],
        json!([
            {"timeMs":0.0,"cx":0.5,"cy":0.5},
            {"timeMs":200.0,"cx":1.0,"cy":0.0,"interactionType":"click","cursorType":"pointer"}
        ])
    );
    let audio = rpc::dispatch(
        &state,
        "get-video-audio-fallback-paths",
        &[imported["path"].clone()],
    )
    .unwrap();
    assert_eq!(audio["paths"].as_array().unwrap().len(), 2);
    assert!(state
        .lock()
        .unwrap()
        .readable(std::path::Path::new(audio["paths"][0].as_str().unwrap()))
        .is_ok());
    // Importing a new file must not reopen an older project during editor hydration.
    assert_eq!(
        rpc::dispatch(&state, "load-current-project-file", &[]).unwrap()["success"],
        false
    );
    let data =
        json!({"version":2,"videoPath":imported["path"],"editor":{"padding":20,"clipRegions":[]}});
    let saved = rpc::dispatch(&state, "create-project-file", &[data]).unwrap();
    assert!(saved["projectId"].is_string());
    let mut edited = saved["project"].clone();
    edited["editor"]["padding"] = 42.into();
    let updated = rpc::dispatch(
        &state,
        "save-project-file",
        &[edited, json!("suggestion"), saved["path"].clone()],
    )
    .unwrap();
    assert_eq!(updated["path"], saved["path"]);
    let mut legacy_snapshot = updated["project"].clone();
    legacy_snapshot.as_object_mut().unwrap().remove("projectId");
    let resaved = rpc::dispatch(
        &state,
        "save-project-file",
        &[legacy_snapshot, json!(null), saved["path"].clone()],
    )
    .unwrap();
    assert_eq!(resaved["projectId"], saved["projectId"]);
    let new_state = State::new().unwrap();
    let loaded = rpc::dispatch(&new_state, "load-current-project-file", &[]).unwrap();
    assert_eq!(loaded["project"]["editor"]["padding"], 42);
    assert_eq!(
        rpc::dispatch(&new_state, "get-cursor-telemetry", &[]).unwrap(),
        telemetry
    );
    assert!(new_state.lock().unwrap().session["webcamPath"].is_string());
    let imported_cursor =
        std::path::PathBuf::from(format!("{}.cursor.json", imported_path.display()));
    let legacy_cursor = imported_path.with_extension("cursor.json");
    std::fs::rename(&imported_cursor, &legacy_cursor).unwrap();
    assert_eq!(
        rpc::dispatch(&state, "get-cursor-telemetry", &[]).unwrap(),
        telemetry
    );
    let rewritten = rpc::dispatch(
        &state,
        "set-cursor-telemetry",
        &[json!(null), json!([{ "timeMs":5, "cx":0.2, "cy":0.3 }])],
    )
    .unwrap();
    assert_eq!(
        rpc::dispatch(&state, "get-cursor-telemetry", &[]).unwrap(),
        rewritten
    );
    assert_eq!(files::read_json(&imported_cursor).unwrap()["version"], 2);
    rpc::dispatch(&state, "set-cursor-telemetry", &[json!(null), json!([])]).unwrap();
    assert!(!imported_cursor.exists() && !legacy_cursor.exists());
    assert_eq!(
        rpc::dispatch(&state, "get-cursor-telemetry", &[]).unwrap()["samples"],
        json!([])
    );
    assert!(std::path::Path::new(saved["path"].as_str().unwrap())
        .with_extension("recordarch.bak")
        .is_file());
    let library = rpc::dispatch(&state, "list-recordings", &[]).unwrap();
    assert_eq!(library["value"].as_array().unwrap().len(), 1);
    let paths = json!([imported["path"]]);
    rpc::dispatch(
        &state,
        "set-recordings-removed",
        &[paths.clone(), json!(true)],
    )
    .unwrap();
    assert!(
        rpc::dispatch(&state, "list-recordings", &[]).unwrap()["value"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    rpc::dispatch(&state, "set-recordings-removed", &[paths, json!(false)]).unwrap();
    assert_eq!(
        rpc::dispatch(&state, "list-recordings", &[]).unwrap()["value"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let outside = std::env::temp_dir().join(format!("unapproved-{}.mp4", uuid::Uuid::new_v4()));
    std::fs::write(&outside, b"private").unwrap();
    assert!(state.lock().unwrap().readable(&outside).is_err());
    assert!(rpc::dispatch(&state, "get-video-audio-fallback-paths", &[json!(outside)]).is_err());
    assert!(files::managed(&outside).is_err());
    let copy = rpc::dispatch(
        &state,
        "save-project-file-named",
        &[
            resaved["project"].clone(),
            json!("Copy"),
            json!(null),
            json!("copy"),
        ],
    )
    .unwrap();
    assert_ne!(copy["projectId"], saved["projectId"]);
    assert!(std::path::Path::new(saved["path"].as_str().unwrap()).is_file());
    let renamed = rpc::dispatch(
        &state,
        "save-project-file-named",
        &[
            copy["project"].clone(),
            json!("Renamed"),
            json!(null),
            json!("rename"),
        ],
    )
    .unwrap();
    assert_eq!(renamed["projectId"], copy["projectId"]);
    assert!(!std::path::Path::new(copy["path"].as_str().unwrap()).exists());
    let stream =
        rpc::dispatch(&state, "export-stream-open", &[json!({"extension":"gif"})]).unwrap();
    assert!(stream["tempPath"].is_string());
    rpc::dispatch(
        &state,
        "export-stream-close",
        &[stream["streamId"].clone(), json!({"abort":true})],
    )
    .unwrap();
    assert!(!std::path::Path::new(stream["tempPath"].as_str().unwrap()).exists());
    let destination = root.join("published.mp4");
    files::publish(&source, &destination).unwrap();
    assert!(files::publish(&outside, &destination).is_err());
    assert_eq!(std::fs::read(&destination).unwrap(), b"fixture media");
    {
        let mut s = state.lock().unwrap();
        s.renderer_ready = Some(std::time::Instant::now() - std::time::Duration::from_secs(31));
        s.jobs.insert(
            "abandoned".into(),
            json!({"id":"abandoned","status":"running"}),
        );
    }
    let abandoned = rpc::dispatch(&state, "arch-export-status", &[json!("abandoned")]).unwrap();
    assert_eq!(abandoned["job"]["status"], "failed");
    assert_eq!(abandoned["job"]["error"], "Desktop renderer disconnected");
    std::fs::remove_file(outside).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn ranges_cover_random_access_and_reject_invalid_offsets() {
    use record_arch::rpc::server::parse_range;
    assert_eq!(parse_range("bytes=0-99", 1000), Some((0, 99)));
    assert_eq!(parse_range("bytes=900-", 1000), Some((900, 999)));
    assert_eq!(parse_range("bytes=-20", 1000), Some((980, 999)));
    for invalid in [
        "bytes=1000-",
        "bytes=50-20",
        "bytes=-0",
        "bytes=0-1,3-4",
        "items=0-9",
    ] {
        assert_eq!(parse_range(invalid, 1000), None);
    }
    assert_eq!(parse_range("bytes=0-", 0), None);
}
