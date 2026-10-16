"""Give native Windows tests the same verified runtime DLLs as the installer."""
import hashlib
from pathlib import Path
import shutil
import sys

ROOT = Path(__file__).resolve().parents[2]
RUNTIMES = [
    ROOT / 'frontend/src-tauri/vulkan-runtime/vulkan-1.dll',
    ROOT / 'frontend/src-tauri/binaries/onnxruntime/onnxruntime.dll',
    ROOT / 'frontend/src-tauri/binaries/onnxruntime/onnxruntime_providers_shared.dll',
]


def stage(destinations):
    for source in RUNTIMES:
        if not source.is_file():
            raise RuntimeError('Verified staged runtime missing: ' + str(source))
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        for destination in destinations:
            destination.mkdir(parents=True, exist_ok=True)
            target = destination / source.name
            shutil.copyfile(source, target)
            if hashlib.sha256(target.read_bytes()).hexdigest() != digest:
                raise RuntimeError('Runtime copy mismatch: ' + str(target))
            print(str(target), digest)


if __name__ == '__main__':
    if len(sys.argv) < 2:
        raise SystemExit('Usage: stage-test-runtime.py <test binary directory> [...]')
    stage([Path(argument) for argument in sys.argv[1:]])
