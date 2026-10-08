"""Check the packaged WebKit app with synthetic media and an isolated library."""
import json
import os
import pathlib
import subprocess
import tempfile
import array
root = pathlib.Path(tempfile.mkdtemp(prefix='record-arch-native-'))
source = root / 'source.mp4'
subprocess.run(['ffmpeg', '-v', 'error', '-f', 'lavfi', '-i', 'testsrc2=size=640x360:rate=24', '-f', 'lavfi', '-i', 'sine=frequency=440', '-t', '2', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-c:a', 'aac', str(source)], check=True)
pathlib.Path(str(source) + '.cursor.json').write_text(json.dumps({'version': 2, 'samples': [
    {'timeMs': 0, 'cx': .2, 'cy': .3, 'cursorType': 'arrow'},
    {'timeMs': 1000, 'cx': .8, 'cy': .6, 'interactionType': 'click', 'cursorType': 'pointer'},
]}))
subprocess.run(['ffmpeg', '-v', 'error', '-f', 'lavfi', '-i', 'sine=frequency=880', '-t', '2', str(source.with_suffix('.microphone.wav'))], check=True)
subprocess.run(['ffmpeg', '-v', 'error', '-f', 'lavfi', '-i', 'color=c=pink:size=160x90:rate=24', '-t', '2', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', str(root / 'source-webcam.mp4')], check=True)
env = {**os.environ, 'RECORD_ARCH_HOME': str(root), 'RECORD_ARCH_SOCKET': str(root / 'backend.sock'), 'RECORD_ARCH_SMOKE_VIDEO': str(source)}
subprocess.run(['src-tauri/target/debug/record-arch-desktop'], env=env, check=True, timeout=60)
report = json.loads((root / '.cache/native-smoke.json').read_text())
assert report['success'] and report['playbackWidth'] == 640 and report['playbackHeight'] == 360, report
assert report['cursorSamples'] == 2 and report['audioCompanions'] == 1 and pathlib.Path(report['webcamPath']).is_file(), report
probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-show_format', '-of', 'json', report['tempFilePath']]))
assert probe['streams'][0]['codec_name'] == 'h264' and probe['streams'][0]['width'] == 640
assert any(s['codec_name'] == 'aac' for s in probe['streams'])
assert abs(float(probe['format']['duration']) - 1) < .1
pixels = subprocess.check_output(['ffmpeg', '-v', 'error', '-i', report['tempFilePath'], '-frames:v', '1', '-vf', 'scale=160:90', '-f', 'rawvideo', '-pix_fmt', 'rgb24', '-'])
colors = sum((r > 180 and g < 80 and b < 80) or (g > 180 and r < 80 and b < 80) for r,g,b in zip(pixels[::3],pixels[1::3],pixels[2::3]))
assert colors > 250, 'Export must contain footage, not only the background'
assert all(abs(a-b) < 15 for a,b in zip(pixels[:3],(27,42,51))), 'Background styling must be rendered'
audio = array.array('f', subprocess.check_output(['ffmpeg', '-v', 'error', '-i', report['tempFilePath'], '-f', 'f32le', '-ac', '1', '-']))
assert max(abs(sample) for sample in audio) < .002, 'Muting both source tracks must mute the embedded audio too'
print('PASS: native playback, cursor/audio/webcam sidecars, track muting, strict-CSP renderer, video pixels, styling, H.264/AAC and duration')
print('Test artifacts:', root)
