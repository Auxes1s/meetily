#!/usr/bin/env python3
"""Run the two summary UI entrypoints in the isolated summary debug bundle."""
import argparse
import json
import os
from pathlib import Path
import plistlib
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--model', type=Path, required=True)
    args = parser.parse_args()
    identity = 'com.meetily.summary.test'
    assert plistlib.loads((args.bundle / 'Contents/Info.plist').read_bytes())['CFBundleIdentifier'] == identity
    profile = Path.home() / 'Library/Application Support' / identity
    marker = profile / '.summary-validation-owned'
    if profile.exists() and not marker.exists():
        raise RuntimeError('Refusing to modify an existing unmarked test profile')
    profile.mkdir(parents=True, exist_ok=True)
    marker.touch()
    args.output.mkdir(parents=True, exist_ok=True)
    model_dir = profile / 'models/summary'
    model_dir.mkdir(parents=True, exist_ok=True)
    link = model_dir / args.model.name
    if not link.exists():
        link.symlink_to(args.model.resolve())
    assert link.resolve() == args.model.resolve()
    status = profile / 'onboarding-status.json'
    if not status.exists():
        status.write_text(json.dumps({'status': {'completed': True, 'current_step': 4,
            'last_updated': '2026-10-16T00:00:00Z', 'version': '1.0', 'model_status': {
                'parakeet': 'not_downloaded', 'summary': 'downloaded', 'selected_summary_model': 'qwen3.5:4b'}}}))
    report = profile / 'summary-generation-smoke.json'
    if report.exists():
        report.rename(profile / f'summary-generation-smoke-{time.time_ns()}.json')
    env = dict(os.environ, MEETILY_SUMMARY_SMOKE='1')
    env.pop('MEETILY_LLAMA_HELPER', None)
    with (args.output / 'native.log').open('w') as log:
        process = subprocess.Popen([str((args.bundle / 'Contents/MacOS/meetily').resolve())], env=env,
                                   stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 600
            while time.monotonic() < deadline and process.poll() is None and not report.exists():
                time.sleep(.25)
            if not report.exists():
                raise RuntimeError('The native summary check did not produce a report')
            result = json.loads(report.read_text())
            (args.output / 'native-webview.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
            print(json.dumps(result, ensure_ascii=False))
            if not result.get('ok'):
                raise SystemExit(1)
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


if __name__ == '__main__':
    main()
