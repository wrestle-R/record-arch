"""Opt-in Hyprland integration: capture only a temporary, empty virtual display."""
import json
import os
import pathlib
import signal
import subprocess
import tempfile
import time
import uuid

binary = str(pathlib.Path('src-tauri/target/debug/record-arch').resolve())
root = pathlib.Path(tempfile.mkdtemp(prefix='record-arch-capture-'))
monitor = 'RecordArchTest-' + uuid.uuid4().hex[:8]
env = {**os.environ, 'RECORD_ARCH_HOME': str(root), 'RECORD_ARCH_SOCKET': str(root / 'backend.sock')}
backend = None
created = False


def command(*args, check=True):
    result = subprocess.run([binary, '--json', *args], env=env, capture_output=True, text=True, timeout=45)
    if check and result.returncode:
        raise AssertionError(result.stderr or result.stdout)
    return json.loads(result.stdout or result.stderr)


def hypr(*args):
    return subprocess.check_output(['hyprctl', *args], text=True)


try:
    assert 'HYPRLAND_INSTANCE_SIGNATURE' in env, 'Run this test inside Hyprland'
    assert not any(m['name'] == monitor for m in json.loads(hypr('monitors', '-j')))
    assert hypr('output', 'create', 'headless', monitor).strip() == 'ok'
    created = True
    monitors = json.loads(hypr('monitors', '-j'))
    virtual = next(m for m in monitors if m['name'] == monitor)
    assert not any(w['monitor'] == virtual['id'] for w in json.loads(hypr('clients', '-j'))), 'Test display must be empty'
    backend = subprocess.Popen([binary, 'serve'], env=env, stdout=subprocess.DEVNULL)
    for _ in range(100):
        if (root / 'backend.sock').exists():
            break
        time.sleep(.05)
    sources = command('sources', 'list')
    assert any(s['id'] == f'screen:{monitor}' for s in sources)
    output = root / 'recordings' / 'paused.mp4'
    command('record', 'start', '--source', f'screen:{monitor}', '--hide-cursor', '--output', str(output))
    time.sleep(1.2)
    command('record', 'pause')
    paused = command('record', 'status')
    assert paused['paused'] and paused['recording']
    time.sleep(1.5)
    assert abs(command('record', 'status')['elapsed'] - paused['elapsed']) < .1
    command('record', 'resume')
    time.sleep(1.2)
    command('record', 'stop')
    probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_format', '-show_streams', '-of', 'json', str(output)]))
    duration = float(probe['format']['duration'])
    assert 2.2 < duration < 3.9, probe
    assert probe['streams'][0]['codec_name'] == 'h264'
    assert probe['streams'][0]['width'] == virtual['width']
    print(f'PASS: real Hyprland capture, pause/resume, finalization ({duration:.2f}s, paused time excluded)')
    # Simulate a recorder crash after a previously finalized segment.
    output = root / 'recordings' / 'recovered.mp4'
    command('record', 'start', '--source', f'screen:{monitor}', '--output', str(output))
    time.sleep(.7)
    command('record', 'pause')
    command('record', 'resume')
    active = command('record', 'status')
    os.kill(active['pid'], signal.SIGKILL)
    time.sleep(.2)
    failed = command('record', 'status')
    assert not failed['recording'] and failed['error']
    result = command('record', 'recover', failed['recoveryManifest'])
    assert result['success'] and output.is_file()
    assert not pathlib.Path(failed['recoveryManifest']).exists()
    print('PASS: unexpected recorder exit and recovery of finalized segments')
    # Device startup failure must stop screen capture and expose an error.
    result = command('record', 'start', '--source', f'screen:{monitor}', '--microphone', 'record-arch-nonexistent-source', check=False)
    assert result['success'] is False
    assert not command('record', 'status')['recording']
    print('PASS: device failure cleans up screen recording')
    output = root / 'recordings' / 'shutdown.mp4'
    command('record', 'start', '--source', f'screen:{monitor}', '--output', str(output))
    active = command('record', 'status')
    time.sleep(.6)
    backend.terminate()
    assert backend.wait(timeout=20) == 0
    assert output.is_file()
    try:
        os.kill(active['pid'], 0)
        raise AssertionError('Recorder child survived backend shutdown')
    except ProcessLookupError:
        pass
    print('PASS: backend shutdown finalizes capture and reaps children')
    print('Test artifacts:', root)
finally:
    if backend and backend.poll() is None:
        try:
            if command('record', 'status')['recording']:
                command('record', 'stop')
        finally:
            backend.terminate()
            backend.wait(timeout=10)
    if created:
        assert hypr('output', 'remove', monitor).strip() == 'ok'
