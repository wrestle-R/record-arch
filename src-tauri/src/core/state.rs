use super::{capture::Recorder, export::Encoder, files};
use anyhow::{bail, Result};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub struct State {
    pub video: Option<PathBuf>,
    pub renderer_ready: bool,
    pub unsaved: bool,
    pub jobs: HashMap<String, Value>,
    pub project: Option<PathBuf>,
    pub session: Value,
    pub approved: HashSet<PathBuf>,
    pub recorder: Recorder,
    pub encoders: HashMap<String, Encoder>,
    pub streams: HashMap<String, PathBuf>,
    pub media_base: String,
    pub token: String,
    pub events: Vec<Value>,
}
pub type Shared = Arc<Mutex<State>>;
impl State {
    pub fn new() -> Result<Shared> {
        files::init()?;
        Ok(Arc::new(Mutex::new(Self {
            video: None,
            renderer_ready: false,
            unsaved: false,
            jobs: HashMap::new(),
            project: None,
            session: Value::Null,
            approved: HashSet::new(),
            recorder: Recorder::default(),
            encoders: HashMap::new(),
            streams: HashMap::new(),
            media_base: String::new(),
            token: uuid::Uuid::new_v4().to_string(),
            events: Vec::new(),
        })))
    }
    pub fn approve(&mut self, path: &Path) -> Result<PathBuf> {
        let path = path.canonicalize()?;
        if !path.is_file() {
            bail!("Expected a file");
        }
        self.approved.insert(path.clone());
        Ok(path)
    }
    pub fn readable(&self, path: &Path) -> Result<PathBuf> {
        let path = path.canonicalize()?;
        if !self.approved.contains(&path) && !path.starts_with(files::root().canonicalize()?) {
            bail!("File has not been opened or approved");
        }
        Ok(path)
    }
    pub fn media_url(&self, path: &Path) -> String {
        let mut url = url::Url::parse(&format!("{}/video", self.media_base))
            .expect("Media server initialized");
        url.query_pairs_mut()
            .append_pair("path", &path.to_string_lossy())
            .append_pair("token", &self.token);
        url.to_string()
    }
    pub fn event(&mut self, channel: &str, args: Value) {
        self.events
            .push(serde_json::json!({"channel":channel,"args":args}));
        if self.events.len() > 128 {
            self.events.remove(0);
        }
    }
    pub fn select_video(&mut self, path: &Path) -> Result<()> {
        let path = self.approve(path)?;
        self.video = Some(path.clone());
        self.session = serde_json::json!({"videoPath":path,"webcamPath":null,"timeOffsetMs":0,"hideOverlayCursorByDefault":true});
        self.event(
            "recording-session-changed",
            serde_json::json!([self.session]),
        );
        Ok(())
    }
}
