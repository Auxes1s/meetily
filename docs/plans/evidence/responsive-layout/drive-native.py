"""Drive only the isolated responsive debug profile; retain native screenshots."""
from pathlib import Path
import copy
import json
import shutil
import sqlite3
import subprocess
import time

root = Path(__file__).resolve().parents[4]
evidence = Path(__file__).resolve().parent
profile = Path.home() / 'Library/Application Support/com.meetily.responsive.test'
report = profile / 'responsive-layout-smoke.json'
started = time.time()
handled = set()


def screenshot(name):
    native = profile / 'responsive-screenshots' / name
    if native.exists():
        shutil.copyfile(native, evidence / name)
        return
    info = json.loads(subprocess.check_output(['swift', str(evidence / 'native-window.swift')], text=True))
    coords = [str(round(info[key])) for key in ['X', 'Y', 'Width', 'Height']]
    subprocess.run(['screencapture', '-x', '-R' + ','.join(coords), str(evidence / name)], check=True)


for _ in range(4800):
    if report.exists() and report.stat().st_mtime >= started:
        data = json.loads(report.read_text())
        phase = data.get('phase')
        if phase == 'native-input' and data['target'] not in handled:
            handled.add(data['target'])
            processes = subprocess.check_output(['ps', '-axo', 'pid=,comm='], text=True)
            pid = next(line.strip().split(None, 1)[0] for line in processes.splitlines()
                       if 'Meetily Responsive Test.app/Contents/MacOS/meetily' in line)
            marker = data['marker'].replace('\\', '\\\\').replace('"', '\\"')
            script = f'''tell application "System Events"
                tell (first application process whose unix id is {pid})
                    if name of window 1 is not "Meetily Responsive Test" then error "Wrong native test window"
                    set frontmost to true
                    keystroke "{marker}"
                end tell
            end tell'''
            subprocess.run(['osascript', '-e', script], check=True)
            print('Native keyboard input:', data['target'], flush=True)
        if phase == 'failed':
            (evidence / 'native-failure.json').write_text(json.dumps(data, indent=2, ensure_ascii=False))
            screenshot('native-failure.png')
            raise RuntimeError(data.get('error'))
        if phase == 'seed-enhancements' and phase not in handled:
            handled.add(phase)
            db = sqlite3.connect(profile / 'meeting_minutes.sqlite')
            blocks = copy.deepcopy(data['blocks'])
            blocks[0]['content'][0]['text'] = 'Disposable enhanced notes — Unicode 日本語 café'
            for state, version in [('approved', 'responsive-approved'), ('candidate', 'responsive-candidate')]:
                db.execute('''INSERT INTO meeting_enhancement_versions
                    (id,meeting_id,state,status,document_json,source_snapshot_json,created_at,updated_at)
                    VALUES (?,?,?,'ready',?,'{}',datetime('now'),datetime('now'))''',
                    (version, data['meeting_id'], state, json.dumps({'schema_version': 1, 'summary_json': blocks, 'markdown': 'Disposable enhanced notes', 'claims': []})))
            db.execute('''INSERT INTO meeting_enhancement_settings (meeting_id,approved_version_id)
                VALUES (?,?) ON CONFLICT(meeting_id) DO UPDATE SET approved_version_id=excluded.approved_version_id''', (data['meeting_id'], 'responsive-approved'))
            db.commit()
            db.close()
            print('Seeded isolated enhanced documents', flush=True)
        name = data.get('screenshot')
        if name and name not in handled:
            handled.add(name)
            try:
                screenshot(name)
                print(name, len(data.get('cases', [])), flush=True)
            except (subprocess.CalledProcessError, RuntimeError) as failure:
                print('Screenshot unavailable:', name, failure, flush=True)
        if phase == 'finished':
            (evidence / 'native-webview-result.json').write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n')
            print(json.dumps({'ok': data['ok'], 'cases': len(data['cases']), 'continuity': data['continuity']}), flush=True)
            break
    time.sleep(0.1)
else:
    raise RuntimeError('Native responsive smoke timed out')
