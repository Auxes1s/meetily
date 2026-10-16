"""Stage the pinned official x64 Vulkan loader and its redistribution license."""
import hashlib
import io
from pathlib import Path
import struct
from urllib.request import Request, urlopen
import zipfile

VERSION = '1.4.309.0'
ARCHIVE_SHA256 = '7d969f4d7b44e387667d3148f61559497c22d50cbe3d50adc9e5409afbce2df1'
ARCHIVE_SIZE = 15_738_272
URL = f'https://sdk.lunarg.com/sdk/download/{VERSION}/windows/VulkanRT-{VERSION}-Components.zip?Human=true'
PREFIX = f'VulkanRT-{VERSION}-Components'


def stage(destination):
    # Checksum published by https://vulkan.lunarg.com/sdk/files.json.
    with urlopen(Request(URL, headers={'User-Agent': 'Mozilla/5.0'}), timeout=60) as response:
        data = response.read(ARCHIVE_SIZE + 1)
    if len(data) != ARCHIVE_SIZE or hashlib.sha256(data).hexdigest() != ARCHIVE_SHA256:
        raise RuntimeError('Official Vulkan runtime archive integrity check failed')
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        loader = archive.read(PREFIX + '/x64/vulkan-1.dll')
        license_text = archive.read(PREFIX + '/VulkanRT-License.txt')
    if loader[:2] != b'MZ':
        raise RuntimeError('Vulkan loader is not a Windows executable')
    offset = struct.unpack_from('<I', loader, 0x3c)[0]
    if loader[offset:offset + 4] != b'PE\0\0' or struct.unpack_from('<H', loader, offset + 4)[0] != 0x8664:
        raise RuntimeError('Vulkan loader is not x64')
    if destination.is_symlink():
        raise RuntimeError('Refusing a linked runtime staging directory')
    destination.mkdir(parents=True, exist_ok=True)
    for name, contents in [('vulkan-1.dll', loader), ('VulkanRT-License.txt', license_text)]:
        target = destination / name
        temporary = destination / (name + '.partial')
        temporary.write_bytes(contents)
        temporary.replace(target)
        print(name, hashlib.sha256(contents).hexdigest())


if __name__ == '__main__':
    stage(Path(__file__).resolve().parents[2] / 'frontend/src-tauri/vulkan-runtime')
