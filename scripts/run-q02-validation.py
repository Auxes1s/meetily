#!/usr/bin/env python3
"""Run a prebuilt, opt-in Q02 bundle using installed models and isolated records.

Each run archives the previous TEST profile. Never launch a production bundle.
No dependencies, model downloads, private library copies, or cloud services.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import plistlib
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

REPO = Path(__file__).resolve().parents[1]
IDENTIFIER = 'com.meetily.q02.test'


def sha256(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def verify_repeated_pcm(path, pattern, samples):
    if path.stat().st_size != samples * 4:
        raise RuntimeError('Retained source PCM sample count differs from supplied capture audio')
    offset = 0
    with path.open('rb') as stream:
        for data in iter(lambda: stream.read(1024 * 1024), b''):
            start = offset % len(pattern)
            expected = (pattern * math.ceil((start + len(data)) / len(pattern)))[start:start + len(data)]
            if data != expected:
                raise RuntimeError('Retained source PCM differs byte for byte from supplied capture audio')
            offset += len(data)
    return sha256(path)


def canonical_records(path):
    with sqlite3.connect(path.as_uri() + '?mode=ro', uri=True) as db:
        if db.execute("SELECT COUNT(*) FROM meetings WHERE title NOT LIKE 'Q02%'").fetchone()[0]:
            raise RuntimeError('Preservation run requires a wholly disposable Q02 library')
        if db.execute("SELECT COUNT(*) FROM recording_sessions WHERE status IN ('starting','recording','finalizing')").fetchone()[0]:
            raise RuntimeError('Close or recover the previous Q02 session before reusing its profile')
        records = {}
        for table in ('meetings', 'meeting_notes', 'transcripts', 'recording_sessions', 'meeting_note_links'):
            key = 'meeting_id' if table == 'meeting_notes' else 'id'
            records[table] = {row[0]: list(row[1:]) for row in db.execute(f'SELECT {key},* FROM {table}')}
        return records


def percentile(values, fraction=.95):
    values = sorted(v for v in values if v is not None and math.isfinite(v))
    return values[max(0, math.ceil(len(values) * fraction) - 1)] if values else None


def verify_edit_accounting(ui, notes_json):
    if 'edit_receipts' not in ui:
        return {'instrumented': False}
    receipts = ui['edit_receipts']
    ids = [r['id'] for r in receipts]
    if len(receipts) != ui['edit_attempts'] or len(set(ids)) != len(ids):
        raise RuntimeError('Original-note edit attempts have missing or duplicate receipts')
    if ui.get('runtime_errors') or any(not r.get('inserted') or not r.get('saved') or r.get('error') for r in receipts):
        raise RuntimeError('Original-note edit attempt failed or was not acknowledged')
    if any(f" {identifier} " not in notes_json for identifier in ids):
        raise RuntimeError('Export lost an attempted original-note edit')
    return {'instrumented': True, 'attempts': len(receipts), 'acknowledged': len(receipts),
            'exported': len(receipts), 'failed': 0, 'lost': 0,
            'skipped_busy_ticks': ui['skipped_busy_edit_ticks']}


def summarize(trace, native):
    latest, decoded, queued, latency, windows = {}, {}, [], [], []
    intervals, pcm_backlog = {}, []
    coverage_measured = True
    inference = 0
    waits, stale_previews, decoder_instances = [], 0, 0
    for row in trace['rows']:
        source = row.get('source')
        if row['kind'] == 'input':
            latest[source] = row['audio_end']
            if 'samples' in row:
                intervals.setdefault(source, []).append((row['audio_end'] - row['samples'] / 16000, row['audio_end']))
            else:
                coverage_measured = False
        elif row['kind'] == 'decode':
            decoded[source] = row['audio_end']
            inference += row['inference_seconds']
            windows.append(row['window_seconds'])
            if row.get('fresh_source_audio') is False and not row.get('finalization_decode', False):
                stale_previews += 1
            queued.append({'wall_seconds': row['wall_seconds'], 'seconds': sum(max(0, end - decoded.get(s, 0)) for s, end in latest.items())})
            # Count delivered PCM coverage, excluding silence and late-starting
            # source clocks. Union overlapping VAD padding rather than counting
            # the same captured interval twice. Completed decoding is distinct
            # from canonical text finalization.
            remaining = 0
            for channel, ranges in intervals.items():
                boundary = decoded.get(channel, -math.inf)
                retained = [(max(start, boundary), end) for start, end in ranges if end > boundary]
                intervals[channel] = retained
                cursor = -math.inf
                for start, end in sorted(retained):
                    remaining += max(0, end - max(start, cursor))
                    cursor = max(cursor, end)
            pcm_backlog.append({'wall_seconds': row['wall_seconds'], 'seconds': remaining})
        elif row['kind'] == 'event':
            latency.append(max(0, row['wall_seconds'] - row['audio_end']))
        elif row['kind'] == 'worker_wait':
            waits.append(row)
        elif row['kind'] == 'worker_ready':
            decoder_instances += row['decoder_instances']
    middle = [r['seconds'] for r in queued if native['audio_seconds'] * .25 < r['wall_seconds'] < native['audio_seconds'] * .5]
    late = [r['seconds'] for r in queued if native['audio_seconds'] * .75 < r['wall_seconds'] < native['audio_seconds']]
    pcm_middle = [r['seconds'] for r in pcm_backlog if native['audio_seconds'] * .25 < r['wall_seconds'] < native['audio_seconds'] * .5]
    pcm_late = [r['seconds'] for r in pcm_backlog if native['audio_seconds'] * .75 < r['wall_seconds'] < native['audio_seconds']]
    stranded = [r for r in waits if r.get('fresh_buffered_audio_at_wait')
                and r.get('queued_chunks_at_wait') == 0 and r['wait_seconds'] > .001]
    continuity = {'instrumented': bool(waits), 'wait_observations': len(waits),
                  'decoder_instances': decoder_instances if decoder_instances else None,
                  'idle_with_queued_audio_seconds': sum(r.get('idle_with_queued_audio_seconds', 0) for r in waits),
                  'max_idle_with_queued_audio_seconds': max((r.get('idle_with_queued_audio_seconds', 0) for r in waits), default=None),
                  'stranded_fresh_buffer_wait_count': len(stranded),
                  'stranded_fresh_buffer_wait_seconds': sum(r['wait_seconds'] for r in stranded),
                  'unchanged_preview_decodes': stale_previews,
                  'decode_busy_fraction_of_session_wall': inference / native['wall_seconds'] if native.get('wall_seconds') else None,
                  'scope': 'single overlapping worker; queue wake time excludes busy inference and input-empty waits; endpoint finalization can reuse provisional audio'}
    return {'inference_seconds': inference, 'inference_rtf': inference / native['audio_seconds'],
            'decode_count': len(windows), 'p95_decode_window_seconds': percentile(windows),
            'max_decode_backlog_seconds': max((r['seconds'] for r in queued), default=0),
            'p95_decode_backlog_seconds': percentile([r['seconds'] for r in queued]),
            'middle_p95_backlog_seconds': percentile(middle), 'late_p95_backlog_seconds': percentile(late),
            'p95_capture_to_native_event_seconds': percentile(latency), 'backlog_trace': queued,
            'backlog_definition': 'capture-clock proxy: latest source-VAD end minus completed decode end per source; includes silence/start gaps; unresolved final hypotheses are separate',
            'delivered_pcm_backlog': {'measured': coverage_measured and bool(pcm_backlog),
                'max_seconds': max((r['seconds'] for r in pcm_backlog), default=None) if coverage_measured else None,
                'p95_seconds': percentile([r['seconds'] for r in pcm_backlog]) if coverage_measured else None,
                'middle_p95_seconds': percentile(pcm_middle) if coverage_measured else None,
                'late_p95_seconds': percentile(pcm_late) if coverage_measured else None,
                'definition': 'unique delivered source PCM coverage beyond completed decode ends; silence and overlapping padding excluded; provisional text is separate'},
            'processing_continuity': continuity,
            'sustained_gate_passed': False, 'fidelity_gate_passed': False}


def profile_dir():
    if sys.platform == 'darwin':
        return Path.home() / 'Library/Application Support' / IDENTIFIER
    if sys.platform == 'win32':
        return Path(os.environ['APPDATA']) / IDENTIFIER
    raise RuntimeError('Q02 supports macOS and Windows native targets')


def bundle_paths(bundle):
    if sys.platform == 'darwin':
        contents = bundle / 'Contents'
        info = plistlib.loads((contents / 'Info.plist').read_bytes())
        if info.get('CFBundleIdentifier') != IDENTIFIER:
            raise RuntimeError('Refusing a non-Q02 bundle identifier')
        return contents / 'MacOS/meetily', contents / 'MacOS/ffmpeg', contents / 'MacOS/llama-helper'
    if bundle.suffix.lower() != '.exe':
        raise RuntimeError('Windows --bundle must name the isolated Q02 executable')
    # The native command guard runs only after application startup. Reject an
    # accidental production executable BEFORE it can open its own library.
    manifest_path = bundle.parent / 'q02-validation-package.json'
    try:
        manifest = json.loads(manifest_path.read_text())
    except (OSError, ValueError) as error:
        raise RuntimeError('Windows requires a Q02 build manifest beside the executable') from error
    if not isinstance(manifest, dict) or (
        manifest.get('identifier') != IDENTIFIER
        or manifest.get('native_validation_feature') is not True
        or manifest.get('binary') != bundle.name
        or not bundle.is_file()
        or manifest.get('binary_sha256') != sha256(bundle)
    ):
        raise RuntimeError('Windows Q02 build manifest identity, feature or binary hash differs')
    return bundle, bundle.parent / 'ffmpeg.exe', bundle.parent / 'llama-helper.exe'


def link_models(source, destination):
    # Link individual immutable files; never link a writable production directory.
    for p in source.rglob('*'):
        if p.is_file() and p.suffix in ('.bin', '.onnx', '.txt', '.gguf'):
            dst = destination / p.relative_to(source)
            dst.parent.mkdir(parents=True, exist_ok=True)
            try:
                dst.symlink_to(p.resolve())
            except OSError:
                shutil.copy2(p, dst)


def mac_processes():
    result = {}
    for line in subprocess.run(['ps', '-Ao', 'pid=,ppid=,%cpu=,rss=,comm='], capture_output=True, text=True, check=True).stdout.splitlines():
        parts = line.split(None, 4)
        if len(parts) == 5:
            result[int(parts[0])] = {'ppid': int(parts[1]), 'cpu_percent': float(parts[2]), 'rss_kib': int(parts[3]), 'comm': parts[4]}
    return result


def sample_resources(pid, before, attributed):
    processes = mac_processes()
    owned = {pid}
    while True:
        children = {p for p, row in processes.items() if row['ppid'] in owned}
        if children <= owned:
            break
        owned |= children
    # WebKit XPC services have launchd as parent. Attribute only a NEW process
    # whose open files actually identify this isolated profile; no host-wide sum.
    for p, row in processes.items():
        if p not in before and p not in attributed and 'WebKit' in row['comm']:
            paths = subprocess.run(['lsof', '-p', str(p), '-Fn'], capture_output=True, text=True).stdout
            if IDENTIFIER in paths:
                attributed.add(p)
    rows = [{'pid': p, **{k:v for k,v in processes[p].items() if k != 'comm'}} for p in owned | attributed if p in processes]
    gpu = []
    try:
        registry = subprocess.run(['/usr/sbin/ioreg', '-a', '-r', '-c', 'AGXAccelerator', '-d', '1'], capture_output=True, check=True)
        gpu = [r.get('PerformanceStatistics', {}) for r in plistlib.loads(registry.stdout)]
    except (OSError, subprocess.CalledProcessError, plistlib.InvalidFileException):
        pass
    return {'wall_time': time.time(), 'processes': rows, 'native_pid': pid, 'host_gpu_statistics': gpu,
            'native_rss_kib': processes.get(pid, {}).get('rss_kib'),
            'attributed_tree_rss_kib': sum(r['rss_kib'] for r in rows),
            'attributed_tree_cpu_percent': sum(r['cpu_percent'] for r in rows),
            'webview_memory_attributed': bool(attributed)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--model-dir', type=Path, required=True)
    parser.add_argument('--pcm', type=Path, help='Exact bounded 16 kHz mono f32le; never re-encode this input')
    parser.add_argument('--system-pcm', type=Path)
    parser.add_argument('--source', choices=('microphone', 'system', 'overlap', 'alternating'), default='microphone')
    parser.add_argument('--provider', choices=('whisper', 'parakeet'), default='whisper')
    parser.add_argument('--routing', choices=('overlapping', 'legacy'), default='overlapping')
    parser.add_argument('--language', choices=('auto','en','auto-translate'), default='auto')
    parser.add_argument('--seconds', type=float, default=30)
    parser.add_argument('--unpaced', action='store_true', help='Actual unpaced worker replay, NOT a virtual-clock or paced acceptance run')
    parser.add_argument('--pause', action='store_true', help='Replay a VAD pause boundary and 250ms gap; native device controls remain separate')
    parser.add_argument('--stress', action='store_true')
    parser.add_argument('--two-sessions', action='store_true')
    parser.add_argument('--package-only', action='store_true')
    parser.add_argument('--sidecar-probe', action='store_true', help='Load the installed built-in model through the packaged native helper using a public literal prompt')
    parser.add_argument('--preserve-test-profile', action='store_true', help='Verify existing disposable Q02 canonical records survive startup of this bundle')
    parser.add_argument('--timeout', type=float)
    args = parser.parse_args()
    if not math.isfinite(args.seconds) or not 0 < args.seconds <= 14400:
        parser.error('--seconds must be in (0,14400]')
    os.umask(0o077)
    binary, ffmpeg, helper = bundle_paths(args.bundle.expanduser().resolve())
    source = args.model_dir.expanduser().resolve()
    if not binary.is_file() or not source.is_dir():
        raise RuntimeError('Existing Q02 bundle and model directory are required')
    # Cross-platform exclusive admission. Stale locks require checking the printed PID.
    lock_path = Path(tempfile.gettempdir()) / 'meetily-q02-validation.lock'
    fd = os.open(lock_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.write(fd, str(os.getpid()).encode()); os.close(fd)
    try:
        run = Path(tempfile.mkdtemp(prefix='meetily-q02-')).resolve()
        print(json.dumps({'run_dir': str(run), 'pid': os.getpid()}), flush=True)
        package = {'binary_sha256': sha256(binary), 'platform': sys.platform, 'identifier': IDENTIFIER,
                   'physical_capture': False, 'instrumented_validation_package': True,
                   'ffmpeg': subprocess.run([str(ffmpeg), '-version'], capture_output=True, text=True, check=True).stdout.splitlines()[0],
                   'helper': subprocess.run([str(helper), '--help'], capture_output=True, text=True, timeout=30).returncode,
                   'ffmpeg_sha256': sha256(ffmpeg), 'helper_sha256': sha256(helper)}
        (run / 'package.json').write_text(json.dumps(package, indent=2))
        if args.package_only:
            print(json.dumps(package)); return
        profile = profile_dir()
        previous_records = canonical_records(profile / 'meeting_minutes.sqlite') if args.preserve_test_profile else None
        if profile.exists() and not args.preserve_test_profile:
            profile.rename(profile.with_name(profile.name + '.previous-' + run.name))
        if sys.platform == 'darwin' and not args.preserve_test_profile:
            # Native records and WebKit recovery/local-storage data must share
            # the same fresh test identity. Archive only this harness's stores.
            for parent in ('WebKit', 'Caches'):
                browser_store = Path.home() / 'Library' / parent / IDENTIFIER
                if browser_store.exists():
                    browser_store.rename(browser_store.with_name(browser_store.name + '.previous-' + run.name))
        profile.mkdir(parents=True, exist_ok=True)
        if not args.preserve_test_profile:
            link_models(source, profile / 'models')
        (profile / 'onboarding-status.json').write_text(json.dumps({'status': {'completed': True, 'current_step': 4, 'version': '1.0', 'last_updated': '2026-10-14T00:00:00Z', 'model_status': {'parakeet': 'downloaded', 'summary': 'not_downloaded'}}}))
        (profile / 'recording_preferences.json').write_text(json.dumps({'preferences': {'save_folder': str(run / 'recordings'), 'auto_save': True, 'file_format': 'mp4'}}))
        if args.routing == 'legacy':
            (profile / 'transcription_preferences.json').write_text(json.dumps({'continuous_whisper': False}))
        (run / 'inputs').mkdir()
        if args.pcm:
            audio = args.pcm.resolve().read_bytes()
        else:
            fixture = REPO / 'frontend/src-tauri/tests/fixtures/audio-source-speech.wav'
            audio = subprocess.run([str(ffmpeg), '-v', 'error', '-i', str(fixture), '-ac', '1', '-ar', '16000', '-f', 'f32le', 'pipe:1'], capture_output=True, check=True).stdout
            audio += bytes(2 * 16000 * 4)
        if not audio or len(audio) % 4:
            raise RuntimeError('PCM must contain complete f32 samples')
        second = args.system_pcm.resolve().read_bytes() if args.system_pcm else audio
        if len(second) != len(audio):
            raise RuntimeError('Source PCM durations must match')
        silence = bytes(len(audio))
        mic = silence if args.source == 'system' else audio
        system = silence if args.source == 'microphone' else second
        if args.source == 'alternating':
            mic, system = audio + silence, silence + second
        (run / 'inputs/microphone.f32').write_bytes(mic)
        (run / 'inputs/system.f32').write_bytes(system)
        model = 'large-v3' if args.provider == 'whisper' else 'parakeet-tdt-0.6b-v3-int8'
        config = {'provider': 'localWhisper' if args.provider == 'whisper' else 'parakeet', 'model': model,
                  'routing': args.routing, 'seconds': args.seconds, 'paced': not args.unpaced, 'pause': args.pause,
                  'stress': args.stress, 'two_sessions': args.two_sessions, 'source': args.source,
                  'sidecar_probe': args.sidecar_probe,
                  'microphone_sha256': hashlib.sha256(mic).hexdigest(), 'system_sha256': hashlib.sha256(system).hexdigest(),
                  'language': args.language, 'model_files': {str(p.relative_to(source)): sha256(p) for p in (source / ('ggml-large-v3.bin' if args.provider == 'whisper' else 'parakeet')).rglob('*') if p.is_file()} if args.provider != 'whisper' else {'ggml-large-v3.bin': sha256(source / 'ggml-large-v3.bin')}}
        (run / 'run.json').write_text(json.dumps(config, indent=2))
        env = dict(os.environ, MEETILY_Q02_VALIDATE='1', MEETILY_Q02_RUN_DIR=str(run), RUST_LOG='warn')
        for key in ('MEETILY_LLAMA_HELPER', 'ORT_DYLIB_PATH', 'MEETILY_WORKFLOW_SMOKE', 'MEETILY_TRANSCRIPT_SMOKE', 'MEETILY_RECORDING_SMOKE'):
            env.pop(key, None)
        measurements = []
        before = set(mac_processes()) if sys.platform == 'darwin' else set()
        attributed = set()
        with (run / 'native.log').open('w') as log:
            process = subprocess.Popen([str(binary)], cwd=run, env=env, stdout=log, stderr=subprocess.STDOUT)
            awake = subprocess.Popen(['/usr/bin/caffeinate', '-ims', '-w', str(process.pid)]) if sys.platform == 'darwin' else None
            deadline = time.monotonic() + (args.timeout or args.seconds * (2 if args.two_sessions else 1) + 600)
            next_progress = time.monotonic() + 60
            try:
                while process.poll() is None and time.monotonic() < deadline:
                    if sys.platform == 'darwin':
                        sample = sample_resources(process.pid, before, attributed)
                        measurements.append(sample)
                        with (run / 'resources.jsonl').open('a') as stream:
                            stream.write(json.dumps(sample) + '\n')
                    report = run / 'report.json'
                    if report.exists():
                        break
                    if time.monotonic() >= next_progress:
                        progress = run / 'progress.json'
                        if progress.exists():
                            print(json.dumps({'run_dir': str(run), 'progress': json.loads(progress.read_text())}), flush=True)
                        next_progress = time.monotonic() + 60
                    time.sleep(1)
                else:
                    if process.poll() is None:
                        raise RuntimeError('Q02 replay timed out; retained source evidence is incomplete')
                report = json.loads((run / 'report.json').read_text())
                if not report.get('ok'):
                    raise RuntimeError(report.get('error', 'Native replay failed'))
            finally:
                if process.poll() is None:
                    process.terminate()
                    try: process.wait(timeout=10)
                    except subprocess.TimeoutExpired: process.kill(); process.wait()
                if awake is not None:
                    awake.terminate(); awake.wait()
                (run / 'resources.json').write_text(json.dumps({'samples': measurements, 'gpu_power_metrics': 'IORegistry utilization is host-wide; process GPU/power unavailable without root/full Xcode on this host',
                    'process_tree_memory': 'descendants plus WebKit processes verified by isolated-profile open files; unattributed XPC memory excluded'}, indent=2))
        report['performance'] = [summarize(json.loads((run / f"trace-{r['recording_session_id']}.json").read_text()), r) for r in report['results']]
        report['resource_summary'] = {key: {'n': len(measurements), 'p95': percentile([s.get(key) for s in measurements]),
            'max': max((s[key] for s in measurements if s.get(key) is not None), default=None)}
            for key in ('native_rss_kib', 'attributed_tree_rss_kib', 'attributed_tree_cpu_percent')}
        for result in report['results']:
            folder = run / 'recordings' / result['recording_session_id']
            result['retained_microphone_pcm_sha256'] = verify_repeated_pcm(folder / 'microphone-source.f32', mic, round(args.seconds * 16000))
            result['retained_system_pcm_sha256'] = verify_repeated_pcm(folder / 'system-source.f32', system, round(args.seconds * 16000))
            result['exact_source_audio_retention_verified'] = True
        report['ui_summary'] = {key: {'n': len(value), 'p95': percentile(value), 'max': max(value, default=None)} for key, value in report['ui'].items() if isinstance(value, list) and all(isinstance(v, (int, float)) for v in value)}
        exported = json.loads((run / 'exports/meeting.json').read_text())
        report['export_transcript_count'] = len(exported['transcripts'])
        report['export_session_count'] = len(exported['recording_sessions'])
        assert report['export_session_count'] == len(report['results'])
        assert report['export_transcript_count'] == sum(r['final_segments'] for r in report['results']) + (10000 if args.stress else 0)
        assert 'Q02 continuity original' in exported['original_notes']['notes_json']
        report['verified_edit_accounting'] = verify_edit_accounting(report['ui'], exported['original_notes']['notes_json'])
        expected_sources = {'microphone'} if args.source == 'microphone' else {'system'} if args.source == 'system' else {'microphone', 'system'}
        real = [t for t in exported['transcripts'] if t['recording_session_id'] is not None]
        if not real or {t['audio_source'] for t in real} != expected_sources:
            raise RuntimeError('Exported source labels do not match the injected active channels')
        for result in report['results']:
            updates = json.loads((run / f"transcripts-{result['recording_session_id']}.json").read_text())
            if any(u['audio_source'] not in expected_sources for u in updates):
                raise RuntimeError('A live update acquired an uninjected source label')
            final = {u['sequence_id']: u for u in updates if not u['is_partial'] and u['text'].strip()}
            saved = [t for t in real if t['recording_session_id'] == result['recording_session_id']]
            if len(final) != len(saved):
                raise RuntimeError('Final Stop manifest and export counts differ')
            for t in saved:
                u = final[t['sequence_id']]
                if (t['text'], t['audio_source'], t['audio_start_time'], t['audio_end_time']) != (u['text'], u['audio_source'], u['audio_start_time'], u['audio_end_time']):
                    raise RuntimeError('Export changed a final passage or its source/capture timing')
                if not 0 <= t['audio_start_time'] <= t['audio_end_time'] <= args.seconds + .1:
                    raise RuntimeError('Passage lies outside its source capture clock')
        sessions = exported['recording_sessions']
        if len({s['id'] for s in sessions}) != len(sessions) or {s['id'] for s in sessions} != {s['id'] for s in report['sessions']} or any(s['status'] != 'completed' for s in sessions) or any(s['meeting_id'] != exported['meeting']['id'] for s in report['sessions']):
            raise RuntimeError('Recording identity or completion differs in export')
        if len(sessions) == 2 and sessions[1]['timeline_offset'] < sessions[0]['duration_seconds']:
            raise RuntimeError('Second recording lost the continuous meeting timeline')
        report['live_export_source_timing_manifest_verified'] = True
        with sqlite3.connect(profile / 'meeting_minutes.sqlite') as db:
            report['sqlite_integrity'] = db.execute('PRAGMA integrity_check').fetchone()[0]
            report['foreign_key_violations'] = db.execute('PRAGMA foreign_key_check').fetchall()
        if previous_records is not None:
            after = canonical_records(profile / 'meeting_minutes.sqlite')
            report['disposable_package_startup_preservation'] = {table: all(after[table].get(key) == row for key, row in rows.items()) for table, rows in previous_records.items()}
            assert all(report['disposable_package_startup_preservation'].values())
        (run / 'report.json').write_text(json.dumps(report, indent=2))
        print(json.dumps({'run_dir': str(run), 'ok': report['ok'], 'performance': [{k:v for k,v in r.items() if k != 'backlog_trace'} for r in report['performance']], 'ui_summary': report['ui_summary']}), flush=True)
    finally:
        lock_path.unlink()


if __name__ == '__main__':
    main()
