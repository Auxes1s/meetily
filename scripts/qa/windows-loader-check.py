"""Resolve native binary imports before execution and retain missing entrypoints."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import re
import subprocess


def check(binary, output):
    reader = Path('C:/Program Files/LLVM/bin/llvm-readobj.exe')
    imports = subprocess.check_output([str(reader), '--coff-imports', str(binary)], text=True)
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetProcAddress.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    kernel.GetProcAddress.restype = ctypes.c_void_p
    kernel.GetModuleFileNameW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_uint]
    kernel.GetModuleFileNameW.restype = ctypes.c_uint
    folders = [binary.parent, binary.parent.parent]
    cookies = [os.add_dll_directory(str(folder)) for folder in folders]
    report = {'binary': str(binary), 'ok': True, 'libraries': []}
    try:
        for block in imports.split('Import {')[1:]:
            name = re.search(r'Name: (.+)', block).group(1).strip()
            local = next((folder / name for folder in folders if (folder / name).is_file()), None)
            item = {'name': name, 'requested_path': str(local) if local else name}
            try:
                library = ctypes.WinDLL(str(local) if local else name)
                path = ctypes.create_unicode_buffer(32768)
                kernel.GetModuleFileNameW(library._handle, path, len(path))
                item['resolved_path'] = path.value
                missing = []
                for symbol in re.findall(r'Symbol: (.+?) \(\d+\)', block):
                    if not kernel.GetProcAddress(library._handle, symbol.encode('ascii')):
                        missing.append(symbol)
                item['missing_symbols'] = missing
                if missing:
                    report['ok'] = False
            except OSError as error:
                item['load_error'] = str(error)
                report['ok'] = False
            report['libraries'].append(item)
    finally:
        for cookie in cookies:
            cookie.close()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    for library in report['libraries']:
        if library.get('missing_symbols') or library.get('load_error'):
            print(json.dumps(library))
    if not report['ok']:
        raise SystemExit('Native loader import check failed; see ' + str(output))
    print('All native imports resolve: ' + str(binary))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    check(args.binary.resolve(), args.output.resolve())
