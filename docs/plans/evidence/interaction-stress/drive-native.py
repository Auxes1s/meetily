"""Fresh isolated interaction stress run. Never opens or changes the production profile.

Build first, then run:
  python3 docs/plans/evidence/interaction-stress/drive-native.py \
    --app 'target/debug/bundle/macos/Meetily Responsive Test.app'

An existing responsive TEST profile is renamed into a retained temporary folder.
Native WKWebView snapshots are required: no desktop screenshot fallback is used.
The driver launches only the supplied, identifier-verified debug test bundle.
"""
from pathlib import Path
import argparse
import copy
import json
import os
import plistlib
import shutil
import sqlite3
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
IDENTIFIER = 'com.meetily.responsive.test'
PROFILE = Path.home() / 'Library/Application Support' / IDENTIFIER
REPORT = PROFILE / 'responsive-layout-smoke.json'


def write_json(name, data):
    (EVIDENCE / name).write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')


def run(app):
    app = app.resolve(strict=True)
    if app.suffix != '.app' or '/target/debug/' not in str(app):
        raise RuntimeError('Only a target/debug .app bundle may be launched')
    with (app / 'Contents/Info.plist').open('rb') as source:
        info = plistlib.load(source)
    if info.get('CFBundleIdentifier') != IDENTIFIER:
        raise RuntimeError('Refusing non-isolated application identifier')
    binary = app / 'Contents/MacOS' / info['CFBundleExecutable']
    processes = subprocess.check_output(['ps', '-axo', 'pid=,comm='], text=True)
    if any(str(binary) in line for line in processes.splitlines()):
        raise RuntimeError('The isolated test bundle is already running; close it before rerunning')
    backup = None
    if PROFILE.is_symlink():
        raise RuntimeError('Refusing symlinked test profile')
    if PROFILE.exists():
        backup_dir = Path(tempfile.mkdtemp(prefix='meetily-interaction-test-backup-', dir=PROFILE.parent))
        backup = backup_dir / IDENTIFIER
        PROFILE.rename(backup)
    journal = {'app': str(app), 'identifier': IDENTIFIER, 'profile': str(PROFILE),
               'retained_test_profile_backup': str(backup) if backup else None,
               'production_profile_touched': False, 'physical_recording_tested': False,
               'snapshots': [], 'started_at': time.strftime('%Y-%m-%dT%H:%M:%S%z')}
    write_json('native-run.json', journal)
    interaction = os.environ.get('MEETILY_INTERACTION_SMOKE') == '1'
    if interaction:
        PROFILE.mkdir(parents=True, exist_ok=True)
        (PROFILE / 'interaction-smoke.json').write_text(json.dumps({'enabled': True, 'source': 'MEETILY_INTERACTION_SMOKE=1 isolated driver'}) + '\n')
    journal['interaction_smoke_enabled'] = interaction
    write_json('native-run.json', journal)
    env = {**os.environ, 'MEETILY_RESPONSIVE_SMOKE': '1'}
    started = time.time()
    handled = set()
    with (EVIDENCE / 'native-app.log').open('w') as output:
        process = subprocess.Popen([str(binary)], env=env, stdout=output, stderr=subprocess.STDOUT)
        journal['pid'] = process.pid
        write_json('native-run.json', journal)
        try:
            for _ in range(10800):
                if process.poll() is not None:
                    raise RuntimeError(f'Isolated app exited before completion: {process.returncode}')
                if REPORT.exists() and REPORT.stat().st_mtime >= started:
                    try:
                        data = json.loads(REPORT.read_text())
                    except json.JSONDecodeError:
                        time.sleep(0.1)
                        continue
                    phase = data.get('phase')
                    if phase == 'interaction-case':
                        write_json('native-interaction-progress.json', data)
                        scenario = data.get('case', {}).get('scenario')
                        if scenario not in handled:
                            handled.add(scenario)
                            print('Interaction case:', scenario, flush=True)
                    if phase in ('native-input', 'native-key') and data['target'] not in handled:
                        handled.add(data['target'])
                        marker = data.get('marker', '').replace('\\', '\\\\').replace('"', '\\"')
                        key_action = 'key code 124' if phase == 'native-key' and data.get('key') == 'right' else f'keystroke "{marker}"'
                        script = f'''tell application "System Events"
                            tell (first application process whose unix id is {process.pid})
                                if name of window 1 is not "Meetily Responsive Test" then error "Wrong isolated test window"
                                set frontmost to true
                                {key_action}
                            end tell
                        end tell'''
                        readiness = f'''tell application "System Events"
                            tell (first application process whose unix id is {process.pid})
                                set frontmost to true
                                if (count of windows) is 0 then error "Isolated test window not ready"
                                if name of window 1 is not "Meetily Responsive Test" then error "Wrong isolated test window"
                            end tell
                        end tell'''
                        last_ready = None
                        # Poll AX readiness before typing; keyboard dispatch is
                        # executed exactly once, so retries cannot duplicate text.
                        for attempt in range(100):
                            last_ready = subprocess.run(['osascript', '-e', readiness], text=True, capture_output=True)
                            if last_ready.returncode == 0:
                                break
                            time.sleep(0.2)
                        else:
                            raise RuntimeError(f'Native window unavailable for isolated PID {process.pid}: {last_ready.stderr}')
                        subprocess.run(['osascript', '-e', script], check=True)
                        print('Native keyboard input:', data['target'], flush=True)
                    seed_key = ('seed-enhancements', data.get('meeting_id'))
                    if phase == 'seed-enhancements' and seed_key not in handled:
                        handled.add(seed_key)
                        prefix = data.get('version_prefix', 'responsive')
                        blocks = copy.deepcopy(data['blocks'])
                        if prefix == 'responsive':
                            blocks[0]['content'][0]['text'] = 'Disposable enhanced notes — Unicode 日本語 café'
                        else:
                            blocks[0]['content'][0]['text'] = 'Design review · Enhanced meeting notes · Synthetic fixture'
                        with sqlite3.connect(PROFILE / 'meeting_minutes.sqlite') as database:
                            for state, version in [('approved', f'{prefix}-approved'), ('candidate', f'{prefix}-candidate')]:
                                database.execute('''INSERT INTO meeting_enhancement_versions
                                    (id,meeting_id,state,status,document_json,source_snapshot_json,created_at,updated_at)
                                    VALUES (?,?,?,'ready',?,'{}',datetime('now'),datetime('now'))''',
                                    (version, data['meeting_id'], state, json.dumps({'schema_version': 1, 'summary_json': blocks, 'markdown': 'Synthetic enhanced notes', 'claims': []})))
                            database.execute('''INSERT INTO meeting_enhancement_settings (meeting_id,approved_version_id)
                                VALUES (?,?) ON CONFLICT(meeting_id) DO UPDATE SET approved_version_id=excluded.approved_version_id''',
                                (data['meeting_id'], f'{prefix}-approved'))
                        print('Seeded isolated enhanced documents:', prefix, flush=True)
                    name = data.get('screenshot')
                    if name and name not in handled:
                        if Path(name).name != name or not name.endswith('.png'):
                            raise RuntimeError('Invalid snapshot name')
                        native = PROFILE / 'responsive-screenshots' / name
                        if not native.is_file():
                            raise RuntimeError(f'Required native WKWebView snapshot missing: {name}')
                        shutil.copyfile(native, EVIDENCE / name)
                        handled.add(name)
                        journal['snapshots'].append({'filename': name, 'fixture': data.get('fixture', 'stress responsive fixture'),
                                                     'capture_started': data.get('capture_started', False)})
                        write_json('native-run.json', journal)
                        print(name, len(data.get('cases', [])), flush=True)
                    if phase == 'failed':
                        write_json('native-failure.json', data)
                        raise RuntimeError(data.get('error'))
                    if phase == 'finished':
                        write_json('native-last-report.json', data)
                        if interaction and len(data.get('interaction_cases', [])) != 8:
                            raise RuntimeError('Interaction opt-in did not execute all 8 required scenarios')
                        if interaction and data.get('runtime_errors'):
                            raise RuntimeError('Interaction run reported unhandled runtime errors')
                        write_json('native-webview-result.json', data)
                        journal['ok'] = data['ok']
                        journal['finished_at'] = time.strftime('%Y-%m-%dT%H:%M:%S%z')
                        write_json('native-run.json', journal)
                        print(json.dumps({'ok': data['ok'], 'responsive_cases': len(data['cases']),
                                          'design_cases': len(data.get('design_cases', [])), 'interaction_cases': len(data.get('interaction_cases', [])),
                                          'continuity': data['continuity']}), flush=True)
                        return
                time.sleep(0.1)
            raise RuntimeError('Native UI smoke timed out after 18 minutes')
        finally:
            if REPORT.exists():
                try:
                    write_json('native-last-report.json', json.loads(REPORT.read_text()))
                except (json.JSONDecodeError, OSError):
                    pass
            # This PID was created and identity-verified by this driver. Never
            # use pkill/killall or affect another running Meetily application.
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    args = parser.parse_args()
    run(args.app if args.app.is_absolute() else ROOT / args.app)
