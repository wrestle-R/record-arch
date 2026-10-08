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
    let state = State::new().unwrap();
    state.lock().unwrap().media_base = "http://127.0.0.1:1".into();
    let imported = rpc::dispatch(&state, "arch-import", &[json!(source)]).unwrap();
    assert_eq!(imported["success"], true);
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
    let new_state = State::new().unwrap();
    let loaded = rpc::dispatch(&new_state, "load-current-project-file", &[]).unwrap();
    assert_eq!(loaded["project"]["editor"]["padding"], 42);
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
    assert!(files::managed(&outside).is_err());
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
