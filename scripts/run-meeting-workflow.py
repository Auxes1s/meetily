#!/usr/bin/env python3
"""Q01 native workflow in a disposable profile, with optional real meeting copy.

No packages are required. The fixture provider is loopback-only; the original
library is opened read-only and all recording paths in its copy are rebased.
"""
import argparse
import hashlib
import fcntl
import http.server
import json
import os
import plistlib
from pathlib import Path
import re
import shutil
import sqlite3
import subprocess
import tempfile
import threading
import time
import uuid

REPO = Path(__file__).resolve().parents[1]
PROFILE = Path.home() / 'Library/Application Support/com.meetily.workflow.test'


class FixtureProvider(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        try:
            request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            prompt = request['messages'][-1]['content']
            compact = 'passages:\n' in prompt
            sources = json.loads(prompt.rsplit('passages:\n' if compact else 'sources:\n', 1)[1])
            claims = [{'kind': 'highlight', 'text': source['text'][:200], 'status': 'confirmed',
                       'section': 'Highlights', 'owner': None, 'deadline': None,
                       **({'citations':[source['id']]} if compact else {'sources': [{'source_id': source['source_id'], 'excerpt': source['text'][:200]}]})}
                      for source in sources if source['text'].strip()]
            body = json.dumps({'choices': [{'message': {'content': json.dumps({'claims': claims})}}]}).encode()
            self.send_response(200)
        except Exception:
            body = b'{"error":"Invalid workflow fixture request"}'
            self.send_response(400)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            value.update(block)
    return value.hexdigest()


def meeting_digest(connection, meeting_id):
    records = {}
    for table in ('meetings', 'transcripts', 'meeting_notes', 'meeting_enhancement_versions',
                  'meeting_enhancement_settings', 'meeting_note_links', 'meeting_participants', 'meeting_tags'):
        key = 'id' if table == 'meetings' else 'meeting_id'
        rows = connection.execute(f'SELECT * FROM {table} WHERE {key}=?', (meeting_id,)).fetchall()
        records[table] = sorted([list(row) for row in rows], key=lambda row: json.dumps(row, ensure_ascii=False))
    return hashlib.sha256(json.dumps(records, ensure_ascii=False, sort_keys=True).encode()).hexdigest()


def prepare_copy(source, meeting_id, run):
    original = sqlite3.connect(source.as_uri() + '?mode=ro', uri=True)
    if not meeting_id:
        row = original.execute('SELECT meeting_id FROM transcripts GROUP BY meeting_id ORDER BY MAX(audio_end_time) DESC LIMIT 1').fetchone()
        if not row:
            raise RuntimeError('The source library contains no recorded meeting')
        meeting_id = row[0]
    row = original.execute('SELECT folder_path,status FROM meetings WHERE id=?', (meeting_id,)).fetchone()
    if not row or row[1] in ('recording', 'finalizing', 'deleting'):
        raise RuntimeError('Choose a completed source meeting')
    before = meeting_digest(original, meeting_id)
    copied = sqlite3.connect(PROFILE / 'meeting_minutes.sqlite')
    original.backup(copied)
    copied.execute('PRAGMA foreign_keys=ON')
    copied.execute('DELETE FROM meetings WHERE id<>?', (meeting_id,))
    tables = {row[0] for row in copied.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    # Durable deletion jobs are intentionally independent of canonical rows.
    # A copied job must never retry against a live recording or backup path.
    for table in ('meeting_deletion_jobs', 'meeting_deletion_tombstones', 'meeting_deletion_sessions'):
        if table in tables:
            copied.execute(f'DELETE FROM {table}')
    # No original recording path survives in a test library.
    folder = Path(row[0]) if row[0] else None
    artifacts = {}
    if folder:
        if not folder.is_dir() or folder.is_symlink():
            raise RuntimeError('Source recording must be an existing ordinary directory')
        for path in folder.rglob('*'):
            if path.is_symlink():
                raise RuntimeError('Source recording contains a symlink')
            if path.is_file():
                artifacts[str(path)] = digest(path)
        destination = run / 'recordings' / 'real-copy'
        shutil.copytree(folder, destination)
        copied.execute('UPDATE meetings SET folder_path=? WHERE id=?', (str(destination), meeting_id))
        if 'recording_sessions' in tables:
            sessions = copied.execute('SELECT id,folder_path FROM recording_sessions WHERE meeting_id=?', (meeting_id,)).fetchall()
            for ordinal, (session_id, session_folder) in enumerate(sessions):
                if not session_folder:
                    continue
                if Path(session_folder).resolve() == folder.resolve():
                    copied.execute('UPDATE recording_sessions SET folder_path=? WHERE id=?', (str(destination), session_id))
                else:
                    extra = Path(session_folder)
                    if not extra.is_dir() or extra.is_symlink():
                        raise RuntimeError('Additional recording folder is unavailable or a symlink')
                    for path in extra.rglob('*'):
                        if path.is_symlink():
                            raise RuntimeError('Additional recording folder contains a symlink')
                        if path.is_file():
                            artifacts[str(path)] = digest(path)
                    extra_copy = run / 'recordings' / f'session-copy-{ordinal}'
                    shutil.copytree(extra, extra_copy)
                    copied.execute('UPDATE recording_sessions SET folder_path=? WHERE id=?', (str(extra_copy), session_id))
    else:
        copied.execute('UPDATE meetings SET folder_path=NULL')
    count = copied.execute('SELECT COUNT(*) FROM transcripts WHERE meeting_id=?', (meeting_id,)).fetchone()[0]
    # The copy only uses the deterministic local provider, never saved credentials.
    copied.execute('DELETE FROM settings')
    copied.commit()
    copied.close()
    original.close()
    return meeting_id, count, before, artifacts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-db', type=Path, default=Path.home() / 'Library/Application Support/com.meetily.ai/meeting_minutes.sqlite')
    parser.add_argument('--meeting-id')
    parser.add_argument('--synthetic-only', action='store_true')
    parser.add_argument('--skip-build', action='store_true')
    parser.add_argument('--routing', choices=('legacy', 'overlapping'), default='overlapping')
    parser.add_argument('--timeout', type=int, default=300)
    parser.add_argument('--benchmark-audio', type=Path)
    parser.add_argument('--benchmark-seconds', type=float, default=90)
    parser.add_argument('--benchmark-start', type=float, default=60)
    args = parser.parse_args()
    os.umask(0o077)
    lock = open(Path(tempfile.gettempdir()) / 'meetily-q01-workflow.lock', 'w')
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        raise RuntimeError('Another isolated Q01 workflow is already running')
    binary = REPO / 'target/debug/bundle/macos/Meetily Workflow Test.app/Contents/MacOS/meetily'
    if subprocess.run(['pgrep', '-f', '^' + re.escape(str(binary))], stdout=subprocess.DEVNULL).returncode == 0:
        raise RuntimeError('Close the isolated workflow test app before preparing another run')
    run = Path(tempfile.mkdtemp(prefix='meetily-q01-')).resolve()
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), FixtureProvider)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    if PROFILE.exists():
        # Preserve previous test evidence rather than overwriting it.
        PROFILE.rename(PROFILE.with_name(PROFILE.name + '.previous-' + uuid.uuid4().hex))
    PROFILE.mkdir(parents=True)
    info = {'run_id': uuid.uuid4().hex, 'fixture_endpoint': f'http://127.0.0.1:{server.server_port}/v1',
            'continuous_whisper': args.routing == 'overlapping'}
    artifact_hashes = {}
    before = None
    if not args.synthetic_only:
        meeting_id, count, before, artifact_hashes = prepare_copy(args.source_db.resolve(), args.meeting_id, run)
        info.update(copied_meeting_id=meeting_id, copied_transcript_count=count)
    if args.benchmark_audio:
        info['benchmark_only'] = True
        (run / 'inputs').mkdir()
        subprocess.run([str(REPO / 'target/debug/ffmpeg'), '-hide_banner', '-loglevel', 'error',
                        '-ss', str(args.benchmark_start), '-i', str(args.benchmark_audio.resolve()),
                        '-t', str(args.benchmark_seconds), '-ac', '1', '-ar', '16000', '-f', 'f32le',
                        str(run / 'inputs/bench.f32')], check=True)
    (run / 'run.json').write_text(json.dumps(info, indent=2))
    (PROFILE / 'recording_preferences.json').write_text(json.dumps({'preferences': {
        'save_folder': str(run / 'recordings'), 'auto_save': True, 'file_format': 'mp4'}}))
    if args.routing == 'legacy':
        (PROFILE / 'transcription_preferences.json').write_text(json.dumps({'continuous_whisper': False}))
    (PROFILE / 'onboarding-status.json').write_text(json.dumps({'status': {
        'completed': True, 'current_step': 4, 'last_updated': '2026-10-14T00:00:00Z',
        'version': '1.0', 'model_status': {'parakeet': 'not_downloaded', 'summary': 'not_downloaded', 'selected_summary_model': 'workflow-fixture'}}}))
    env = dict(os.environ, MEETILY_WORKFLOW_SMOKE='1', MEETILY_TRANSCRIPT_SMOKE='1',
               MEETILY_WORKFLOW_RUN_DIR=str(run),
               MEETILY_SMOKE_MODEL_DIR=str(args.source_db.expanduser().resolve().parent / 'models'),
               MEETILY_SMOKE_MODEL='large-v3', NEXT_PUBLIC_MEETILY_LOCAL_BUILD='1',
               CMAKE_DISABLE_FIND_PACKAGE_OpenSSL='TRUE')
    if not args.skip_build:
        with (run / 'build.log').open('w') as log:
            subprocess.run(['pnpm', 'exec', 'tauri', 'build', '--debug', '--config',
                            'src-tauri/tauri.workflow-test.json', '--bundles', 'app'],
                           cwd=REPO / 'frontend', env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
    bundle_id = plistlib.loads((binary.parent.parent / 'Info.plist').read_bytes())['CFBundleIdentifier']
    if bundle_id != 'com.meetily.workflow.test':
        raise RuntimeError('Refusing to launch a bundle with a non-test application identifier')
    print(json.dumps({'run_dir': str(run), 'native_profile': str(PROFILE)}), flush=True)
    report = None
    try:
        for launch in range(2):
            with (run / f'native-{launch}.log').open('w') as log:
                process = subprocess.Popen([str(binary)], cwd=binary.parent, env=env, stdout=log, stderr=subprocess.STDOUT)
                deadline = time.monotonic() + args.timeout
                try:
                    while time.monotonic() < deadline:
                        path = run / 'report.json'
                        report = json.loads(path.read_text()) if path.exists() else None
                        if report and report.get('phase') in ('failed', 'finished'):
                            break
                        if process.poll() is not None:
                            break
                        time.sleep(0.25)
                    else:
                        raise RuntimeError('Native workflow timed out; inspect local evidence logs')
                finally:
                    if process.poll() is None:
                        process.terminate()
                        try:
                            process.wait(timeout=10)
                        except subprocess.TimeoutExpired:
                            process.kill()
                            process.wait()
            if report and report.get('phase') == 'restart' and launch == 0:
                continue
            break
        if not report or not report.get('ok'):
            raise RuntimeError((report or {}).get('error', 'Native workflow did not complete'))
        if report.get('benchmark_only'):
            print(json.dumps(report), flush=True)
            return
        exports = list((run / 'exports').glob('*'))
        checkpoint = json.loads((run / 'checkpoint.json').read_text())
        fixture = next(json.loads(p.read_text()) for p in exports if p.suffix == '.json' and 'Q01' in p.name)
        assert fixture['original_notes']['notes_json'] == checkpoint['notes_json']
        assert fixture['transcripts'] == checkpoint['transcripts']
        assert checkpoint['edit'] in json.dumps(fixture['versions'], ensure_ascii=False)
        markdown = next(p.read_text() for p in exports if p.suffix == '.md')
        assert checkpoint['original'] in markdown and checkpoint['edit'] in markdown
        if before:
            original = sqlite3.connect(args.source_db.resolve().as_uri() + '?mode=ro', uri=True)
            assert meeting_digest(original, info['copied_meeting_id']) == before, 'Original meeting changed'
            original.close()
            assert all(digest(Path(path)) == expected for path, expected in artifact_hashes.items()), 'Original recording changed'
            assert not (run / 'recordings/real-copy').exists(), 'Copied recording was not deleted'
            assert not list((run / 'recordings').glob('session-copy-*')), 'An additional copied session folder was not deleted'
            report['checks']['original_meeting_and_artifacts_preserved'] = True
        assert all(p.is_file() for p in exports), 'Deletion removed user-owned exports'
        report['checks']['parsed_exports_match_persisted_content'] = True
        (run / 'report.json').write_text(json.dumps(report, indent=2))
        print(json.dumps(report), flush=True)
    finally:
        server.shutdown()


if __name__ == '__main__':
    main()
