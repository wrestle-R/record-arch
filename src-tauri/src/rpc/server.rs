use super::dispatch;
use crate::core::{files, state::Shared};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    os::unix::{
        fs::{OpenOptionsExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    thread,
};
use tiny_http::{Header, Response, Server, StatusCode};
pub fn socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| files::root().join(".cache"))
        .join("record-arch.sock")
}
fn result(state: &Shared, body: &Value) -> Value {
    let Some(channel) = body["channel"].as_str() else {
        return json!({"success":false,"error":"Missing channel"});
    };
    let args = body["args"].as_array().cloned().unwrap_or_default();
    match dispatch(state, channel, &args) {
        Ok(v) => v,
        Err(e) => {
            json!({"success":false,"error":e.to_string(),"message":e.to_string(),"code":"COMMAND_FAILED"})
        }
    }
}
pub fn start(state: &Shared) -> Result<()> {
    // Hold the owner-only lock for the entire backend lifetime.
    let lock_path = socket_path().with_extension("lock");
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(lock_path)?;
    fs2::FileExt::try_lock_exclusive(&lock).context("Record Arch backend is already running")?;
    let socket = socket_path();
    if socket.exists() {
        fs::remove_file(&socket)?;
    }
    let listener = UnixListener::bind(&socket)?;
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
    let shared = state.clone();
    thread::spawn(move || {
        let _lock = lock;
        for client in listener.incoming().flatten() {
            let s = shared.clone();
            thread::spawn(move || {
                let mut client = client;
                let _ = client.set_read_timeout(Some(std::time::Duration::from_secs(30)));
                let mut line = String::new();
                if BufReader::new(&client)
                    .take(16 * 1024 * 1024)
                    .read_line(&mut line)
                    .is_ok()
                {
                    if let Ok(body) = serde_json::from_str::<Value>(&line) {
                        let value = result(&s, &body);
                        let _ = writeln!(client, "{value}");
                    }
                }
            });
        }
    });
    let server = Server::http("127.0.0.1:0").map_err(|e| anyhow::anyhow!(e.to_string()))?;
    {
        let mut s = state.lock().unwrap();
        s.media_base = format!("http://{}", server.server_addr());
        files::write_json(
            &files::root().join(".cache/backend.json"),
            &json!({"url":s.media_base,"token":s.token}),
        )?;
        fs::set_permissions(
            files::root().join(".cache/backend.json"),
            fs::Permissions::from_mode(0o600),
        )?;
    }
    let shared = state.clone();
    thread::spawn(move || {
        for request in server.incoming_requests() {
            let state = shared.clone();
            thread::spawn(move || {
                serve(request, &state);
            });
        }
    });
    Ok(())
}
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).unwrap()
}
fn serve(mut request: tiny_http::Request, state: &Shared) {
    let origin = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Origin"))
        .map(|h| h.value.as_str().to_owned());
    let allowed_origin = origin.as_deref().filter(|o| {
        matches!(
            *o,
            "http://localhost:1420"
                | "http://127.0.0.1:1420"
                | "tauri://localhost"
                | "http://tauri.localhost"
        )
    });
    if origin.is_some() && allowed_origin.is_none() {
        let _ = request.respond(Response::empty(403));
        return;
    }
    if request.method() == &tiny_http::Method::Options {
        let mut response = Response::empty(204)
            .with_header(header(
                "Access-Control-Allow-Headers",
                "Content-Type, X-Record-Arch-Token",
            ))
            .with_header(header("Access-Control-Allow-Methods", "GET, POST, OPTIONS"));
        if let Some(o) = allowed_origin {
            response = response.with_header(header("Access-Control-Allow-Origin", o));
        }
        let _ = request.respond(response);
        return;
    }
    let url = match url::Url::parse(&format!("http://localhost{}", request.url())) {
        Ok(u) => u,
        Err(_) => {
            let _ = request.respond(Response::empty(400));
            return;
        }
    };
    let token = url
        .query_pairs()
        .find(|(k, _)| k == "token")
        .map(|(_, v)| v.to_string())
        .or_else(|| {
            request
                .headers()
                .iter()
                .find(|h| h.field.equiv("X-Record-Arch-Token"))
                .map(|h| h.value.as_str().to_owned())
        });
    if token.as_deref() != Some(state.lock().unwrap().token.as_str()) {
        let _ = request.respond(Response::empty(403));
        return;
    }
    if url.path() == "/rpc" && request.method() == &tiny_http::Method::Post {
        let mut body = String::new();
        let read = request
            .as_reader()
            .take(64 * 1024 * 1024)
            .read_to_string(&mut body);
        let value = read
            .ok()
            .and_then(|_| serde_json::from_str::<Value>(&body).ok())
            .map(|v| result(state, &v))
            .unwrap_or(json!({"success":false,"error":"Invalid RPC body"}));
        let mut response = Response::from_string(value.to_string())
            .with_header(header("Content-Type", "application/json"));
        if let Some(o) = allowed_origin {
            response = response.with_header(header("Access-Control-Allow-Origin", o));
        }
        let _ = request.respond(response);
        return;
    }
    let path = url
        .query_pairs()
        .find(|(k, _)| k == "path")
        .map(|(_, v)| PathBuf::from(v.as_ref()));
    let path = path.and_then(|p| state.lock().unwrap().readable(&p).ok());
    if url.path() != "/video" || path.is_none() {
        let _ = request.respond(Response::empty(403));
        return;
    }
    let path = path.unwrap();
    let Ok(mut file) = fs::File::open(&path) else {
        let _ = request.respond(Response::empty(404));
        return;
    };
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let range = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Range"))
        .map(|h| h.value.as_str());
    let (start, end) = if let Some(range) = range {
        match parse_range(range, size) {
            Some(r) => r,
            None => {
                let _ = request.respond(
                    Response::empty(416)
                        .with_header(header("Content-Range", &format!("bytes */{size}"))),
                );
                return;
            }
        }
    } else {
        (0, size.saturating_sub(1))
    };
    let length = if size == 0 { 0 } else { end - start + 1 };
    let _ = file.seek(SeekFrom::Start(start));
    let mime = match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "mov" => "video/quicktime",
        "ogg" | "opus" => "audio/ogg",
        "flac" => "audio/flac",
        "gif" => "image/gif",
        "mp3" => "audio/mpeg",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    let mut response = Response::new(
        StatusCode(if range.is_some() { 206 } else { 200 }),
        vec![
            header("Content-Type", mime),
            header("Accept-Ranges", "bytes"),
            header(
                "Access-Control-Allow-Origin",
                allowed_origin.unwrap_or("tauri://localhost"),
            ),
        ],
        file.take(length),
        Some(length as usize),
        None,
    );
    if range.is_some() {
        response.add_header(header(
            "Content-Range",
            &format!("bytes {start}-{end}/{size}"),
        ));
    }
    let _ = request.respond(response);
}
pub fn parse_range(range: &str, size: u64) -> Option<(u64, u64)> {
    let (left, right) = range.strip_prefix("bytes=")?.split_once('-')?;
    if size == 0 || right.contains(',') {
        return None;
    }
    if left.is_empty() {
        let suffix = right.parse::<u64>().ok()?;
        if suffix == 0 {
            return None;
        }
        return Some((size.saturating_sub(suffix), size - 1));
    }
    let start = left.parse::<u64>().ok()?;
    let end = if right.is_empty() {
        size - 1
    } else {
        right.parse::<u64>().ok()?.min(size - 1)
    };
    (start <= end && start < size).then_some((start, end))
}
pub fn call(channel: &str, args: Value) -> Result<Value> {
    let mut stream = UnixStream::connect(socket_path())
        .context("Start the desktop app or run record-arch serve first")?;
    stream.set_read_timeout(Some(std::time::Duration::from_secs(120)))?;
    writeln!(stream, "{}", json!({"channel":channel,"args":args}))?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    Ok(serde_json::from_str(&line)?)
}
