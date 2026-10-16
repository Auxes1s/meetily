"""Embed Tauri's Common Controls v6 dependency in native Rust test executables."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]


def prepare(binary):
    tools = sorted((Path(os.environ['ProgramFiles(x86)']) / 'Windows Kits/10/bin').glob('*/x64/mt.exe'), reverse=True)
    if not tools:
        raise RuntimeError('Windows SDK manifest tool is missing')
    output = ROOT / 'target/v0-native-diagnostics'
    output.mkdir(parents=True, exist_ok=True)
    resource = str(binary.resolve()) + ';#1'
    before = subprocess.run([str(tools[0]), '-nologo', '-inputresource:' + resource,
                             '-out:' + str(output / (binary.stem + '-before.manifest'))], capture_output=True, text=True)
    (output / (binary.stem + '-manifest-before.json')).write_text(json.dumps({
        'exit_code': before.returncode, 'stdout': before.stdout, 'stderr': before.stderr,
        'interpretation': 'A missing manifest here means the Rust test does not inherit the production binary manifest.',
    }, indent=2) + '\n', encoding='utf-8')
    subprocess.run([str(tools[0]), '-nologo', '-manifest', str(ROOT / '.github/windows-test.manifest'),
                    '-outputresource:' + resource], check=True)
    after = output / (binary.stem + '-after.manifest')
    subprocess.run([str(tools[0]), '-nologo', '-inputresource:' + resource, '-out:' + str(after)], check=True)
    if 'Microsoft.Windows.Common-Controls' not in after.read_text(encoding='utf-8-sig'):
        raise RuntimeError('Native test Common Controls dependency was not embedded')
    print('Verified native test Common Controls v6 manifest: ' + str(binary))


if __name__ == '__main__':
    prepare(Path(sys.argv[1]))
