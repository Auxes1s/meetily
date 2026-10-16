"""Check an installed candidate in real WebView2 on a disposable Windows CI host.

Uses the W3C WebDriver HTTP API and Python's standard library. No application
IPC is mocked. This proves the listed native workflows on the recorded host;
Windows Server CI is not Windows 11 physical-capture acceptance.
"""
import argparse
import base64
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import platform
import re
import struct
import subprocess
import time
import urllib.error
import urllib.request
import uuid


class Driver:
    def __init__(self):
        self.session = ''
        self.app_pid = None

    def request(self, method, route, body=None):
        data = None if body is None else json.dumps(body).encode()
        req = urllib.request.Request('http://127.0.0.1:4444' + route, data=data,
                                     headers={'Content-Type': 'application/json'}, method=method)
        try:
            with urllib.request.urlopen(req, timeout=75) as response:
                value = json.load(response)['value']
        except urllib.error.HTTPError as error:
            raise RuntimeError(error.read().decode()) from error
        if isinstance(value, dict) and value.get('error'):
            raise RuntimeError(value)
        return value

    def command(self, method, route, body=None):
        if self.app_pid and method == 'POST' and route == '/window/rect':
            # Edge's attach mode does not own native window management. Resize
            # the actual installed application's HWND, never a CSS viewport.
            user = ctypes.WinDLL('user32', use_last_error=True)
            user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
            user.IsWindowVisible.argtypes = [wintypes.HWND]
            user.SetWindowPos.argtypes = [wintypes.HWND, wintypes.HWND, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_uint]
            windows = []
            callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
            @callback_type
            def visit(handle, _):
                owner = wintypes.DWORD()
                user.GetWindowThreadProcessId(handle, ctypes.byref(owner))
                if owner.value == self.app_pid and user.IsWindowVisible(handle):
                    windows.append(handle)
                return True
            user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
            user.EnumWindows(visit, 0)
            if len(windows) != 1:
                raise RuntimeError('Expected exactly one visible window for the installed test process')
            if not user.SetWindowPos(windows[0], None, 20, 20, body['width'], body['height'], 4):
                raise ctypes.WinError(ctypes.get_last_error())
            return {}
        return self.request(method, '/session/' + self.session + route, body)

    def js(self, source, *args):
        return self.command('POST', '/execute/sync', {'script': source, 'args': list(args)})

    def invoke(self, command, args=None):
        result = self.command('POST', '/execute/async', {
            'script': '''const done = arguments[arguments.length - 1];
              window.__TAURI_INTERNALS__.invoke(arguments[0], arguments[1])
                .then(value => done({ok:true, value}), error => done({ok:false, error:String(error)}));''',
            'args': [command, args or {}],
        })
        if not result['ok']:
            raise RuntimeError(result['error'])
        return result.get('value')

    def click(self, selector):
        element = self.command('POST', '/element', {'using': 'css selector', 'value': selector})
        element_id = element['element-6066-11e4-a52e-4f735466cecf']
        self.command('POST', '/element/' + element_id + '/click', {})
        return element_id

    def click_text(self, text):
        element = self.js('''const target = [...document.querySelectorAll('button')].find(b => b.textContent.trim() === arguments[0]);
          if (!target) throw Error('Missing button: ' + arguments[0]); target.scrollIntoView(); return target;''', text)
        self.command('POST', '/element/' + element['element-6066-11e4-a52e-4f735466cecf'] + '/click', {})

    def key(self, value, control=False):
        keys = ([{'type': 'keyDown', 'value': '\ue009'}] if control else [])
        keys += [{'type': 'keyDown', 'value': value}, {'type': 'keyUp', 'value': value}]
        if control:
            keys.append({'type': 'keyUp', 'value': '\ue009'})
        self.command('POST', '/actions', {'actions': [{'type': 'key', 'id': 'qa-keyboard', 'actions': keys}]})

    def wait(self, predicate, label):
        for _ in range(300):
            result = predicate()
            if result:
                return result
            time.sleep(.1)
        raise RuntimeError('Timed out: ' + label)


def attach_installed(app, output, prefix=''):
    """Microsoft's supported attach flow avoids the separate DevToolsActivePort profiles."""
    app_log = (output / (prefix + 'application.log')).open('w')
    driver_log = (output / (prefix + 'driver.log')).open('w')
    application = driver_process = None
    try:
        application = subprocess.Popen([str(app)], stdout=app_log, stderr=subprocess.STDOUT, env={
            **os.environ, 'TAURI_WEBVIEW_AUTOMATION': 'true',
            'WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS': '--remote-debugging-port=9222 --remote-debugging-address=127.0.0.1',
        })
        version = None
        for _ in range(300):
            if application.poll() is not None:
                raise RuntimeError(f'Installed application exited before WebView2 readiness: {application.returncode:#x}')
            try:
                with urllib.request.urlopen('http://127.0.0.1:9222/json/version', timeout=1) as response:
                    version = json.load(response)
                break
            except (OSError, ValueError):
                time.sleep(.2)
        if version is None:
            raise RuntimeError('Installed application did not expose its test-only WebView2 debugging endpoint')
        (output / (prefix + 'webview2.json')).write_text(json.dumps(version, indent=2) + '\n', encoding='utf-8')
        driver_process = subprocess.Popen(['msedgedriver.exe', '--port=4444', '--verbose'], stdout=driver_log, stderr=subprocess.STDOUT)
        driver = Driver()
        driver.app_pid = application.pid
        for _ in range(100):
            if driver_process.poll() is not None:
                raise RuntimeError('Microsoft Edge WebDriver stopped before readiness')
            try:
                driver.request('GET', '/status')
                break
            except (OSError, RuntimeError):
                time.sleep(.2)
        result = driver.request('POST', '/session', {'capabilities': {'alwaysMatch': {
            'browserName': 'webview2', 'ms:edgeChromium': True,
            'ms:edgeOptions': {'debuggerAddress': '127.0.0.1:9222'},
        }}})
        driver.session = result['sessionId']
        return driver, application, driver_process, app_log, driver_log
    except Exception:
        for process in (driver_process, application):
            if process is not None and process.poll() is None:
                process.terminate(); process.wait(timeout=15)
        app_log.close(); driver_log.close()
        raise


def close_installed(handles):
    driver, application, driver_process, app_log, driver_log = handles
    if driver.session:
        try:
            driver.command('DELETE', '')
        except (OSError, RuntimeError):
            pass
    for process in (driver_process, application):
        if process.poll() is None:
            process.terminate(); process.wait(timeout=15)
    app_log.close(); driver_log.close()


def run(app, output, expect_windows_11=False):
    if platform.system() != 'Windows' or os.environ.get('GITHUB_ACTIONS') != 'true':
        raise RuntimeError('This writes disposable fixtures and requires a fresh Windows GitHub Actions runner')
    if not app.is_file():
        raise RuntimeError('Installed application is missing: ' + str(app))
    host = json.loads(subprocess.check_output(['powershell', '-NoProfile', '-Command',
        'Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber,ProductType,OSArchitecture | ConvertTo-Json -Compress'], text=True))
    windows_11 = host['ProductType'] == 1 and int(host['BuildNumber']) >= 22000 and 'Windows 11' in host['Caption']
    if expect_windows_11 and not windows_11:
        raise RuntimeError('This acceptance run requires an actual Windows 11 client OS')
    binary = app.read_bytes()
    offset = struct.unpack_from('<I', binary, 0x3c)[0]
    if binary[:2] != b'MZ' or binary[offset:offset + 4] != b'PE\0\0' or struct.unpack_from('<H', binary, offset + 4)[0] != 0x8664:
        raise RuntimeError('Expected the existing x64 candidate executable')
    output.mkdir(parents=True, exist_ok=False)
    report = {'ok': False, 'host': platform.platform(), 'app': str(app), 'cases': [],
              'physical_capture_tested': False, 'windows_11_tested': windows_11,
              'os': host, 'host_architecture': platform.machine(), 'artifact_architecture': 'x64',
              'x64_under_arm_emulation': platform.machine().lower() in ('arm64', 'aarch64')}
    driver = Driver()
    handles = None
    try:
        handles = attach_installed(app, output)
        driver = handles[0]
        driver.command('POST', '/timeouts', {'script': 60000})
        driver.wait(lambda: driver.js("return !!document.querySelector('.onboarding-flow')"), 'fresh onboarding')
        for width, height in [(1100, 700), (768, 600), (480, 400), (1100, 400)]:
            driver.command('POST', '/window/rect', {'width': width, 'height': height})
            driver.wait(lambda: driver.js('return innerWidth > 0'), 'rendered viewport')
            driver.js("[...document.querySelectorAll('button')].find(b => b.textContent.trim() === 'Use notes only').scrollIntoView()")
            bounded = driver.js('''const b = [...document.querySelectorAll('button')].find(b => b.textContent.trim() === 'Use notes only');
              const r = b.getBoundingClientRect(); return document.documentElement.scrollWidth <= innerWidth + 1 &&
              r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1;''')
            if not bounded:
                raise RuntimeError(f'Onboarding action clipped at {width}x{height}')
            shot = driver.command('GET', '/screenshot')
            (output / f'onboarding-{width}x{height}.png').write_bytes(base64.b64decode(shot))
            report['cases'].append({'view': 'onboarding', 'requested_size': [width, height],
                                    'viewport': driver.js('return [innerWidth, innerHeight]')})
        driver.click_text('Get Started')
        driver.wait(lambda: driver.js("return [...document.querySelectorAll('h1')].some(n => n.textContent === 'Setup Overview')"), 'AI setup overview')
        for width, height in [(1100, 700), (480, 400), (1100, 400)]:
            driver.command('POST', '/window/rect', {'width': width, 'height': height})
            time.sleep(.3)
            driver.js("[...document.querySelectorAll('button')].find(b => b.textContent.trim() === \"Let's Go\").scrollIntoView()")
            if not driver.js('return document.documentElement.scrollWidth <= innerWidth + 1'):
                raise RuntimeError('AI setup overview overflows horizontally')
            (output / f'onboarding-overview-{width}x{height}.png').write_bytes(base64.b64decode(driver.command('GET', '/screenshot')))
            report['cases'].append({'view': 'onboarding-overview', 'requested_size': [width, height],
                                    'viewport': driver.js('return [innerWidth, innerHeight]')})
        driver.click('[aria-label="Setup step 1, completed"]')
        driver.wait(lambda: driver.js("return [...document.querySelectorAll('h1')].some(n => n.textContent === 'Welcome to Meetily')"), 'onboarding back navigation')
        report['onboarding_overview_and_back_navigation'] = True
        driver.click_text('Use notes only')
        driver.wait(lambda: driver.js('return !!document.querySelector(\'[aria-label="Meeting title"]\')'), 'notes workspace')
        if not driver.invoke('get_onboarding_status')['completed']:
            raise RuntimeError('Notes-only completion was not persisted')
        marker = 'Native Windows v0 QA — 日本語 café'
        driver.wait(lambda: driver.js('return !!document.querySelector(\'section[aria-label="My notes"] [contenteditable="true"]\')'), 'hydrated notes editor')
        editor = driver.click('section[aria-label="My notes"] [contenteditable="true"]')
        driver.js('window.__qaOriginalEditor = document.querySelector(\'section[aria-label="My notes"] [contenteditable="true"]\')')
        driver.command('POST', '/element/' + editor + '/value', {'text': marker})
        meeting_id = driver.js("return sessionStorage.getItem('workspace_meeting_id')")
        driver.wait(lambda: marker in ((driver.invoke('api_get_meeting_notes', {'meetingId': meeting_id}) or {}).get('notes_json') or ''), 'native note commit')
        for width, height in [(1440, 900), (900, 600), (600, 500), (480, 400), (1100, 400)]:
            driver.command('POST', '/window/rect', {'width': width, 'height': height})
            time.sleep(.4)
            if not driver.js('return document.documentElement.scrollWidth <= innerWidth + 1'):
                raise RuntimeError(f'Workspace overflow at {width}x{height}')
            if not driver.js('return document.body.innerText.includes(arguments[0])', marker):
                raise RuntimeError('Resizing lost visible note content')
            if not driver.js('return window.__qaOriginalEditor === document.querySelector(\'section[aria-label="My notes"] [contenteditable="true"]\')'):
                raise RuntimeError('Resizing remounted the original editor')
            shot = driver.command('GET', '/screenshot')
            (output / f'workspace-{width}x{height}.png').write_bytes(base64.b64decode(shot))
            report['cases'].append({'view': 'workspace', 'requested_size': [width, height],
                                    'viewport': driver.js('return [innerWidth, innerHeight]')})
        driver.click('section[aria-label="My notes"] [contenteditable="true"]')
        driver.key('z', control=True)
        driver.wait(lambda: not driver.js('return document.body.innerText.includes(arguments[0])', marker), 'undo after resize')
        driver.key('y', control=True)
        driver.wait(lambda: driver.js('return document.body.innerText.includes(arguments[0])', marker), 'redo after resize')
        driver.wait(lambda: marker in ((driver.invoke('api_get_meeting_notes', {'meetingId': meeting_id}) or {}).get('notes_json') or ''), 'redo native commit')
        report['undo_redo_after_resize'] = True
        driver.command('POST', '/refresh', {})
        driver.wait(lambda: driver.js('return document.body.innerText.includes(arguments[0])', marker), 'reloaded notes')
        report['notes_only_onboarding'] = True
        report['editor_native_commit_and_reload'] = True
        report['editor_identity_preserved_during_resize'] = True

        def snapshot(view, width, height):
            driver.command('POST', '/window/rect', {'width': width, 'height': height})
            time.sleep(.5)
            if not driver.js('return document.documentElement.scrollWidth <= innerWidth + 1'):
                raise RuntimeError(f'{view} overflows horizontally at {width}x{height}')
            name = f'{view}-{width}x{height}.png'
            (output / name).write_bytes(base64.b64decode(driver.command('GET', '/screenshot')))
            report['cases'].append({'view': view, 'requested_size': [width, height],
                                    'viewport': driver.js('return [innerWidth, innerHeight]')})

        driver.invoke('api_save_transcript', {
            'meetingTitle': 'Native Windows QA — 日本語 café', 'meetingId': meeting_id,
            'recordingSessionId': str(uuid.uuid4()), 'folderPath': None,
            'transcripts': [{'id': 'windows-qa-' + str(uuid.uuid4()), 'text': 'Disposable system transcript — 日本語 café ' * 30,
                             'timestamp': '00:01', 'audio_source': 'system', 'audio_start_time': 1.0,
                             'audio_end_time': 2.0, 'duration': 1.0, 'sequence_id': 1}],
        })
        driver.command('POST', '/refresh', {})
        driver.wait(lambda: driver.js('return !!document.querySelector(\'[aria-controls="meeting-transcript-panel"]\')'), 'transcript toggle')
        driver.click('[aria-controls="meeting-transcript-panel"]')
        driver.wait(lambda: driver.js("return [...document.querySelectorAll('.transcript-scroll')].some(n => n.getClientRects().length && n.textContent.includes('Disposable system transcript'))"), 'native saved transcript')
        for width, height in [(1440, 900), (480, 400), (1100, 400)]:
            snapshot('transcript', width, height)
            if not driver.js("return [...document.querySelectorAll('.transcript-scroll')].some(n => n.getClientRects().length && n.clientHeight >= 40 && n.scrollWidth <= n.clientWidth + 1)"):
                raise RuntimeError('Transcript scroll area is unusable at the requested size')
        driver.click('[aria-controls="meeting-transcript-panel"]')
        report['native_transcript_drawer_responsive'] = True
        driver.js("location.href='/library'")
        driver.wait(lambda: driver.js("return !!document.querySelector('.meeting-library')"), 'meeting library')
        for width, height in [(1440, 900), (480, 400), (1100, 400)]:
            snapshot('library', width, height)
        report['library_responsive'] = True
        driver.js("location.href='/settings'")
        driver.wait(lambda: driver.js("return document.querySelectorAll('[role=tab]').length >= 5"), 'settings categories')
        labels = driver.js("return [...document.querySelectorAll('[role=tab]')].map(n => n.textContent.trim())")
        for label in labels:
            element = driver.js("const n = [...document.querySelectorAll('[role=tab]')].find(n => n.textContent.trim() === arguments[0]); n.scrollIntoView(); return n", label)
            driver.command('POST', '/element/' + element['element-6066-11e4-a52e-4f735466cecf'] + '/click', {})
            for width, height in [(1100, 700), (480, 400), (1100, 400)]:
                snapshot('settings-' + re.sub('[^a-z]+', '-', label.lower()).strip('-'), width, height)
        report['settings_categories_responsive'] = labels
        # Verify sidecars from the installed directory, without loading a model.
        ffmpeg = subprocess.run([str(app.parent / 'ffmpeg.exe'), '-version'], capture_output=True, text=True, check=True, timeout=30)
        report['ffmpeg'] = ffmpeg.stdout.splitlines()[0]
        helper = subprocess.run([str(app.parent / 'llama-helper.exe')], input='{"type":"capabilities"}\n',
                                capture_output=True, text=True, timeout=30, check=True)
        responses = [json.loads(line) for line in helper.stdout.splitlines() if line.startswith('{')]
        if not any(response.get('protocol_version') == 2 and response.get('structured_generation') for response in responses):
            raise RuntimeError('Installed helper lacks the required protocol')
        report['helper_protocol_2'] = True
        for name in ['onnxruntime.dll', 'onnxruntime_providers_shared.dll', 'vulkan-1.dll', 'VulkanRT-License.txt']:
            if not (app.parent / name).is_file():
                raise RuntimeError('Installed runtime resource missing: ' + name)
        report['installed_onnx_files_present'] = True
        report['installed_vulkan_loader_and_license_present'] = True
        report['ok'] = True
    except Exception as error:
        report['error'] = str(error)
        if driver.session:
            try:
                (output / 'failure.png').write_bytes(base64.b64decode(driver.command('GET', '/screenshot')))
                (output / 'failure-body.txt').write_text(driver.js('return document.body.innerText'), encoding='utf-8')
            except (OSError, RuntimeError):
                pass
        raise
    finally:
        if handles is not None:
            close_installed(handles)
        (output / 'result.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--expect-windows-11', action='store_true')
    args = parser.parse_args()
    run(args.app.resolve(), args.output.resolve(), args.expect_windows_11)
