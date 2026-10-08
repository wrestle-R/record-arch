"""Check the packaged WebKit app with synthetic media and an isolated library."""
import json
import os
import pathlib
import subprocess
import tempfile
root = pathlib.Path(tempfile.mkdtemp(prefix='record-arch-native-'))
source = root / 'source.mp4'
subprocess.run(['ffmpeg', '-v', 'error', '-f', 'lavfi', '-i', 'testsrc2=size=640x360:rate=24', '-f', 'lavfi', '-i', 'sine=frequency=440', '-t', '2', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-c:a', 'aac', str(source)], check=True)
env = {**os.environ, 'RECORD_ARCH_HOME': str(root), 'RECORD_ARCH_SOCKET': str(root / 'backend.sock'), 'RECORD_ARCH_SMOKE_VIDEO': str(source)}
subprocess.run(['src-tauri/target/debug/record-arch-desktop'], env=env, check=True, timeout=60)
report = json.loads((root / '.cache/native-smoke.json').read_text())
assert report['success'] and report['playbackWidth'] == 640 and report['playbackHeight'] == 360, report
probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-show_format', '-of', 'json', report['tempFilePath']]))
assert probe['streams'][0]['codec_name'] == 'h264' and probe['streams'][0]['width'] == 640
assert any(s['codec_name'] == 'aac' for s in probe['streams'])
assert abs(float(probe['format']['duration']) - 1) < .1
pixels = subprocess.check_output(['ffmpeg', '-v', 'error', '-i', report['tempFilePath'], '-frames:v', '1', '-vf', 'scale=160:90', '-f', 'rawvideo', '-pix_fmt', 'rgb24', '-'])
colors = sum((r > 180 and g < 80 and b < 80) or (g > 180 and r < 80 and b < 80) for r,g,b in zip(pixels[::3],pixels[1::3],pixels[2::3]))
assert colors > 250, 'Export must contain footage, not only the background'
assert all(abs(a-b) < 15 for a,b in zip(pixels[:3],(27,42,51))), 'Background styling must be rendered'
print('PASS: packaged native playback, strict-CSP renderer, video pixels, styling, H.264/AAC and duration')
print('Test artifacts:', root)
