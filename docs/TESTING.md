# Verification record

Verified locally on Arch Linux and Hyprland on 8 October 2026. This is a development preview, with the remaining differences listed in [MIGRATION.md](MIGRATION.md).

## Checks completed

| Check | Result |
| --- | --- |
| TypeScript and production frontend build | Passed |
| Frontend regression tests | 980 tests across 116 files passed |
| Rust integration tests | Passed: import/save/reopen with legacy cursor/audio/webcam sidecars, telemetry normalization/save/clear, stable project identity, copy/rename, backups, library removal/restore, access boundaries, byte ranges, stream abort, atomic publication and abandoned export status |
| Rust formatting and Clippy | Passed with warnings treated as errors |
| Packaged Tauri/WebKit startup | Passed using production assets at `tauri://localhost`; no startup errors |
| Native preview codecs | H.264/AAC playback passed with the installed system GStreamer plugins |
| Native effects export | 640×360 H.264/AAC, 15 FPS, one second; decoded video pixels, background styling, cursor/audio/webcam discovery and source-track muting verified |
| Browser editor preview | Composed preview contains the synthetic video's red/green pixels |
| Editor/CLI flow | Playback, timeline hydration, save, MP4/GIF jobs, cancellation during rendering, subsequent export, latest-project reopening, source offsets, 2× speed, background changes and muted-audio samples passed |
| Export buttons | Both MP4 and GIF outputs were written and inspected |
| Hyprland capture | Passed on a temporary empty headless display through real `wf-recorder`; physical displays were not recorded |
| Recording lifecycle | Pause/resume excludes paused time; recorder-crash recovery, invalid-device cleanup and backend shutdown passed |
| JavaScript dependency audit | No reported vulnerabilities |

An initial packaged run caught WebKit's missing Screen Orientation API and Pixi's generated shader helpers conflicting with the app's CSP. Those were fixed and the native checks rerun. Agent exports also now suspend preview rendering to avoid competing for GPU work. Export streams return their temporary path and support abort cleanup.

## Reproduce automated checks

From the repository root:

```sh
npm ci
npm test
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --features desktop --bins -- -D warnings
```

For the native WebKit check, build with production assets first:

```sh
npm run tauri -- build --debug --no-bundle
python3 scripts/native-check.py
```

The native check creates synthetic footage and an isolated library under `/tmp`, opens the app, checks playback and export, and exits automatically. It does not import your existing recordings.

The compositor test is opt-in and must run in a Hyprland session:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --bin record-arch
python3 scripts/capture-check.py
```

It creates a uniquely named virtual display, checks that no client window is on it, records only that display, and removes it afterward. It exercises pause/resume, recovery, failed device selection and shutdown. Artifacts stay in the printed `/tmp/record-arch-capture-*` directory.

## Browser and agent integration

Generate a two-second fixture:

```sh
mkdir -p /tmp/record-arch-tests
ffmpeg -y -v error -f lavfi -i testsrc2=size=640x360:rate=24 -f lavfi -i sine=frequency=440 -t 2 -c:v libx264 -pix_fmt yuv420p -c:a aac /tmp/record-arch-tests/demo.mp4
```

Run the backend in one terminal:

```sh
RECORD_ARCH_HOME=/tmp/record-arch-integration RECORD_ARCH_SOCKET=/tmp/record-arch-integration.sock src-tauri/target/debug/record-arch serve
```

Run Vite in another terminal:

```sh
RECORD_ARCH_HOME=/tmp/record-arch-integration npm run dev
```

Then import the fixture and run the checks sequentially:

```sh
RECORD_ARCH_HOME=/tmp/record-arch-integration RECORD_ARCH_SOCKET=/tmp/record-arch-integration.sock src-tauri/target/debug/record-arch editor /tmp/record-arch-tests/demo.mp4
npx playwright install chromium
node scripts/integration.mjs
node scripts/ui-export.mjs
node scripts/preview-check.mjs
```

These scripts use software WebGL in an isolated browser. Export performance there does not represent the native GPU's performance. Do not run these scripts simultaneously: they intentionally share the same test backend and selected project.

## Still needs verification

- Real microphone and webcam devices, synchronization and device loss during capture.
- Capturing physical displays and moving/occluded windows across monitor configurations.
- Whisper transcription with an installed executable and a real local model.
- AppImage/DEB bundles and installation on a second machine; the tested native build embeds production assets but is not a tested distributable package.
- Long recordings, large projects and memory/performance under sustained use.

The CI workflow runs the frontend build/tests and Rust checks on Ubuntu. Native/compositor tests remain explicit local checks because they need a graphical session, codecs and compositor access.
