# Migration status

This is a Tauri/Rust migration with the existing React editor retained. It is not a claim that every upstream platform integration has already been reproduced.

| Area | Implementation | Verification / difference |
| --- | --- | --- |
| Editor launch | Latest project or empty editor; separate Open/Record actions | Browser and packaged WebKit startup/playback verified |
| Projects | Original project schema, JSON persistence, autosave, backup, library | Rust roundtrip, identity, restore and access-boundary tests |
| Visual editing | Original crop, backgrounds, shadow, rounding, zoom/spring motion, cursor styles, annotations, captions, presets, clips, speeds and trims | Original editor test suite retained; render integration uses original FrameRenderer |
| MP4 export | Native FFmpeg decode/encode and audio mix; Pixi renders effects | Synthetic edited export produced H.264/AAC |
| GIF export | Same rendered frames, FFmpeg palette conversion | UI save verified; CLI job checks included in the integration script |
| Agent export | Submit/status/cancel jobs using the desktop effects renderer | Submission, progress, completion and cancellation verified |
| Linux recording | wf-recorder display/window-region capture, system audio, segmented pause/resume and recovery | Real Hyprland headless-display capture verified; physical display configurations need verification |
| Microphone/camera | FFmpeg PulseAudio/V4L2 sidecars, editor webcam settings | Device-specific verification required |
| Recorder visibility | Windows hide; tray and CLI remain available | Visible HUD capture exclusion is not supported |
| Window capture | Fixed Hyprland geometry region | Does not follow a moving window or isolate occlusion |
| Cursor telemetry | Existing telemetry can be loaded/edited/rendered | Native click/keyboard telemetry capture has not been ported |
| Captions | Existing caption editing/rendering; Rust whisper-cli integration and SRT/VTT sidecars | Needs a local whisper executable and model |
| Recording library | Discovery, thumbnails, hide/restore, appending media | Webcam/telemetry merge parity for multi-recording imports remains incomplete |
| Accounts/cloud sharing | Original UI source retained, cloud destination disabled | Requires an independent hosting/auth backend; upstream credentials are not copied |
| Updates/GPU backends | No upstream updater or platform-specific capture/export binaries | Linux software FFmpeg is the implemented encoder |

The compatibility surface is inventoried in `api-surface.json`. Unsupported calls return `UNSUPPORTED_CAPABILITY`; they do not report fabricated success. Event subscriptions and obsolete platform-only commands are not equivalent to missing editor features.

## Architecture

- `src/arch/`: editor entry, empty workspace, recorder, agent export orchestration.
- `src/desktop/`: transport, native dialogs/events, generated compatibility bridge.
- `src/components/video-editor/`: retained upstream editing domains and rendering UI.
- `src-tauri/src/core/`: project/media/device/process logic independent of Tauri.
- `src-tauri/src/rpc/`: shared native/CLI/local HTTP API handlers.
- `src-tauri/src/desktop.rs` and `tray.rs`: Tauri windows and tray lifecycle.
- `src-tauri/src/cli*.rs`: Clap CLI and export jobs.

New code is split by domain. Some inherited editor files remain large; they have not been mechanically split during the desktop migration. The compatibility name `window.electronAPI` is retained internally to reduce editor churn; no Electron runtime is packaged.

The local server binds only to loopback and uses an owner-only random token. The CLI socket is owner-only. Media reads are restricted to the managed library or explicitly opened files. Destructive library operations use the managed root and move files to trash. Child programs receive argument arrays, never shell-interpolated commands.
