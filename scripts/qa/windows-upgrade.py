"""Verify a real 0.4.1-to-preview NSIS upgrade using a disposable CI library."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import sqlite3
import subprocess
import time

spec = importlib.util.spec_from_file_location('native_driver', Path(__file__).with_name('windows-native.py'))
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)
OLD_SHA256 = '6321068c98da8bf5392df3825aaa5be0e28349e54436f53d485c08c54b9c9315'


def install(installer, destination):
    completed = subprocess.run([str(installer), '/S', '/D=' + str(destination)], timeout=180)
    if completed.returncode:
        raise RuntimeError('Installer failed: ' + str(completed.returncode))


def launch(app, logfile):
    return native.attach_installed(app, logfile.parent, logfile.stem + '-')


def close(app, handles):
    native.close_installed(handles)
    # Limit cleanup to the exact test installation on this disposable host.
    escaped = str(app).replace("'", "''")
    subprocess.run(['powershell', '-NoProfile', '-Command',
        "Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -eq '" + escaped +
        "' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }"], check=True)


def snapshot(database, ids):
    with sqlite3.connect(database) as db:
        placeholders = ','.join('?' for _ in ids)
        return {
            'meetings': db.execute(f'SELECT id,title,created_at,folder_path FROM meetings WHERE id IN ({placeholders}) ORDER BY id', ids).fetchall(),
            'transcripts': db.execute(f'SELECT id,meeting_id,transcript,timestamp,audio_start_time,audio_end_time,duration FROM transcripts WHERE meeting_id IN ({placeholders}) ORDER BY id', ids).fetchall(),
            'summary_source_strings': db.execute(f'SELECT meeting_id,result FROM summary_processes WHERE meeting_id IN ({placeholders}) ORDER BY meeting_id', ids).fetchall(),
            'original_notes': db.execute(f'SELECT meeting_id,notes_json,notes_markdown FROM meeting_notes WHERE meeting_id IN ({placeholders}) ORDER BY meeting_id', ids).fetchall(),
        }


def run(old_installer, installer, destination, output):
    if platform.system() != 'Windows' or os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('Upgrade fixtures require a fresh disposable Windows CI runner')
    if hashlib.sha256(old_installer.read_bytes()).hexdigest() != OLD_SHA256:
        raise RuntimeError('Previous installer differs from its official release digest')
    profile = Path(os.environ['APPDATA']) / 'com.meetily.ai'
    if profile.exists() or destination.exists():
        raise RuntimeError('Upgrade acceptance requires a fresh profile and installation path')
    output.mkdir(parents=True, exist_ok=False)
    report = {'ok': False, 'old_installer_sha256': OLD_SHA256,
              'new_installer_sha256': hashlib.sha256(installer.read_bytes()).hexdigest(),
              'host': platform.platform(), 'physical_capture_tested': False}
    app = destination / 'meetily.exe'
    handles = None
    try:
        install(old_installer, destination)
        handles = launch(app, output / 'old-driver.log'); driver = handles[0]
        if driver.invoke('plugin:app|version') != '0.4.1':
            raise RuntimeError('Previous installed app has the wrong version')
        database = profile / 'meeting_minutes.sqlite'
        driver.wait(database.is_file, 'legacy database initialization')
        ids = []
        for index in range(4):
            saved = driver.invoke('api_save_transcript', {
                'meetingTitle': f'Upgrade fixture {index} — 日本語 café',
                'transcripts': [{'id': f'legacy-passage-{index}', 'text': f'Original passage {index} — café 日本語', 'timestamp': '00:01',
                                 'audio_start_time': 1.0, 'audio_end_time': 2.0, 'duration': 1.0}],
                'folderPath': None, 'authToken': None,
            })
            ids.append(saved['meeting_id'])
        driver.invoke('save_onboarding_status_cmd', {'status': {
            'version': '1.0', 'completed': True, 'current_step': 4,
            'model_status': {'parakeet': 'not_downloaded', 'summary': 'not_downloaded'},
            'last_updated': '2026-10-16T00:00:00Z',
        }})
        summaries = [
            json.dumps({'Topics': {'title': 'Topics', 'blocks': [{'content': 'Legacy proposal — unapproved'}]}}, ensure_ascii=False),
            json.dumps(json.dumps({'markdown': 'Twice encoded — 日本語'}, ensure_ascii=False), ensure_ascii=False),
            json.dumps({'markdown': '# Unicode\n\nTaglish — café'}, ensure_ascii=False),
            json.dumps({'summary_json': [{'id': 'rich-block', 'type': 'paragraph', 'content': [{'type': 'text', 'text': 'Rich notes — 日本語', 'styles': {'bold': True}}], 'children': []}]}, ensure_ascii=False),
        ]
        with sqlite3.connect(database) as db:
            for index, meeting_id in enumerate(ids):
                db.execute("INSERT INTO summary_processes(meeting_id,status,created_at,updated_at,result) VALUES(?,'completed','2026-10-16','2026-10-16',?)", (meeting_id, summaries[index]))
                notes = json.dumps([{'id': f'original-{index}', 'type': 'paragraph', 'content': [{'type': 'text', 'text': f'Original notes {index} — 日本語 café', 'styles': {'bold': True}}], 'children': []}], ensure_ascii=False)
                db.execute("INSERT INTO meeting_notes(meeting_id,notes_json,notes_markdown,created_at,updated_at) VALUES(?,?,'Authoritative original','2026-10-16','2026-10-16')", (meeting_id, notes))
        before = snapshot(database, ids)
        close(app, handles); handles = None
        install(installer, destination)
        handles = launch(app, output / 'new-driver.log'); driver = handles[0]
        if driver.invoke('plugin:app|version') != '0.5.0-rc.1':
            raise RuntimeError('Upgraded installed app has the wrong version')
        driver.wait(lambda: driver.js('return !!document.querySelector(\'[aria-label="Meeting title"]\')'), 'upgraded workspace')
        for meeting_id in ids:
            driver.invoke('api_get_meeting_metadata', {'meetingId': meeting_id})
            driver.invoke('api_get_meeting_enhancements', {'meetingId': meeting_id})
        after = snapshot(database, ids)
        if before != after:
            raise RuntimeError('Upgrade changed legacy identities or canonical source bytes')
        driver.js("sessionStorage.setItem('workspace_meeting_id',arguments[0]); localStorage.setItem('meetily.lastNotesMeeting',arguments[0]); location.href='/'", ids[0])
        driver.wait(lambda: driver.js("return document.body.innerText.includes('Original notes 0 — 日本語 café')"), 'preserved original notes rendering')
        driver.js("location.href='/library'")
        driver.wait(lambda: driver.js("return document.body.innerText.includes('Upgrade fixture 3 — 日本語 café')"), 'upgraded meeting library')
        if before != snapshot(database, ids):
            raise RuntimeError('Opening the upgraded notes/library rewrote canonical source bytes')
        with sqlite3.connect(database) as db:
            if db.execute('PRAGMA integrity_check').fetchone()[0] != 'ok' or db.execute('PRAGMA foreign_key_check').fetchall():
                raise RuntimeError('Upgraded library failed integrity checks')
        backups = list(profile.glob('meeting_minutes.sqlite.backups/*'))
        if not backups:
            raise RuntimeError('Expected a native pre-migration database backup')
        report.update(ok=True, legacy_meetings=4, summary_formats=4, canonical_sources_unchanged=True,
                      original_notes_rendered=True, library_reopened=True, sqlite_integrity='ok', migration_backup_present=True)
    except Exception as error:
        report['error'] = str(error)
        raise
    finally:
        if handles is not None:
            close(app, handles)
        (output / 'upgrade-result.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--old-installer', type=Path, required=True)
    parser.add_argument('--installer', type=Path, required=True)
    parser.add_argument('--destination', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    run(args.old_installer.resolve(), args.installer.resolve(), args.destination.resolve(), args.output.resolve())
