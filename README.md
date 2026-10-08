# Record Arch

An editor-first desktop recorder for Arch Linux and Hyprland. Rust/Tauri owns files, processes, recording, media transport, and FFmpeg encoding. The existing Recordly React/Pixi editor carries forward its timeline and visual effects; the UI remains TypeScript rather than being rewritten as Rust widgets.

**Development preview:** core editing and native export are implemented. See [migration status](docs/MIGRATION.md) for platform differences and features that still need hardware verification.

## Run locally

Install a current Rust toolchain, Node 22+ with npm, and the Linux build dependencies listed in [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/).

On Arch, the main packages are:

```sh
sudo pacman -S --needed base-devel rust nodejs npm webkit2gtk-4.1 libappindicator-gtk3 librsvg ffmpeg wf-recorder pipewire pipewire-pulse wireplumber
npm ci
npm run desktop
```

Use your existing Rust installation if managed by rustup. Screen capture expects Hyprland and `hyprctl`. The native webview uses the system GStreamer codecs for preview playback; install `gst-plugins-good`, `gst-plugins-bad`, `gst-plugins-ugly`, and `gst-libav` when required by your footage.

Build the desktop and CLI:

```sh
npm run build
cargo build --manifest-path src-tauri/Cargo.toml --features desktop --bins
```

The binaries are in `src-tauri/target/debug/`. Release builds use `--release`. `npm run tauri -- build` builds distributable desktop packages.

## Workspace

Launch opens the latest usable project, or an empty editor with **Open video** and **New recording**. **Ctrl+O** opens media/projects; **Ctrl+S** saves. The header separates **Open**, **Record**, **Clips**, and **Export**.

Everything stays under `~/Record-Arch` by default:

- `recordings/`: imported copies and captured media, including optional microphone/webcam sidecars.
- `projects/`: versioned `.recordarch` JSON projects and previous-save backups.
- `exports/`: rendered output.
- `.cache/`: temporary encoders, audio, and backend descriptor.
- `.trash/`: recoverable library deletions.

Set `RECORD_ARCH_HOME` to change the root. Original imported videos are kept unchanged. Existing `.recordly` projects can be opened. Paused captures use finalized segments in `.cache/capture-*`; `record status` reports a recovery manifest if the recorder exits unexpectedly. Recovery preserves completed segments, while an unfinished MP4 segment may be unusable.

On Hyprland, the editor and recording window hide during capture. Stop from the tray or CLI. A visible overlay cannot reliably be excluded from full-monitor capture in this backend, so it is hidden. Window capture currently records a fixed region; moving the window does not move that region.

## CLI for agents

With the desktop app running, use `src-tauri/target/debug/record-arch` (install it on your PATH to use the shorter name):

```sh
record-arch --json doctor
record-arch --json sources list
record-arch --json devices
record-arch recorder
record-arch record start --source screen:DP-1 --system-audio
record-arch record status
record-arch record pause
record-arch record resume
record-arch record stop
record-arch record recover /absolute/path/session.json
record-arch editor /absolute/path/video.mp4
record-arch project list
record-arch project create --video /absolute/path/video.mp4 --name Demo
record-arch project inspect /absolute/path/Demo.recordarch
record-arch project apply /absolute/path/Demo.recordarch --edits /absolute/path/editor-patch.json
record-arch export start /absolute/path/Demo.recordarch --output /absolute/path/demo.mp4 --width 1920 --height 1080 --fps 30 --wait
record-arch export list
record-arch export status JOB_ID
record-arch export cancel JOB_ID
```

`record start` also accepts `--microphone SOURCE_NAME`, `--webcam /dev/video0`, `--hide-cursor`, and `--output PATH`. Use the device/source IDs returned by discovery. `--format gif` exports animated GIFs. Commands return structured JSON and a nonzero exit code on failure.

Rendered exports require the desktop editor: Rust handles media and encoding while the webview runs the existing Pixi effects engine. `record-arch serve` runs the backend without a desktop window for capture/library commands; it cannot render visual effects by itself. `rpc CHANNEL '[ARGS]'` exposes lower-level commands.

Example Hyprland bindings (adjust the installed binary path):

```ini
bind = SUPER SHIFT, R, exec, record-arch recorder
bind = SUPER SHIFT, S, exec, record-arch record stop
```

## Verification

```sh
npm run typecheck
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --features desktop --bins -- -D warnings
```

Integration scripts live in `scripts/`; the native diagnostic uses an isolated library and synthetic test footage. [Test results](docs/TESTING.md) record what was actually verified, including hardware-dependent limits.

## Attribution and license

Derived from [Recordly](https://github.com/webadderallorg/Recordly). The original editor, localization, and assets retain their authorship and AGPLv3 terms. See [LICENSE.md](LICENSE.md), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), and [UPSTREAM.md](UPSTREAM.md). Record Arch has a separate name and icon; it is an independent project.
