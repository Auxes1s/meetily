import hashlib
import importlib.util
import io
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location('stage_vulkan', Path(__file__).resolve().parents[1] / 'stage-vulkan-runtime.py')
runtime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)


def fixture(machine=0x8664):
    loader = bytearray(128)
    loader[:2] = b'MZ'
    struct.pack_into('<I', loader, 0x3c, 64)
    loader[64:68] = b'PE\0\0'
    struct.pack_into('<H', loader, 68, machine)
    output = io.BytesIO()
    with zipfile.ZipFile(output, 'w') as archive:
        archive.writestr(runtime.PREFIX + '/x64/vulkan-1.dll', loader)
        archive.writestr(runtime.PREFIX + '/x86/vulkan-1.dll', b'wrong architecture sentinel')
        archive.writestr(runtime.PREFIX + '/VulkanRT-License.txt', b'Synthetic license\r\nPreserve exact bytes\r\n')
    return output.getvalue(), bytes(loader)


class RuntimeStagingTests(unittest.TestCase):
    def test_rejects_unverified_archive_before_writing(self):
        with tempfile.TemporaryDirectory() as root, patch.object(runtime, 'urlopen', return_value=io.BytesIO(b'untrusted')):
            destination = Path(root) / 'runtime'
            with self.assertRaisesRegex(RuntimeError, 'integrity'):
                runtime.stage(destination)
            self.assertFalse(destination.exists())

    def test_stages_x64_only_and_retains_license_bytes(self):
        data, loader = fixture()
        with tempfile.TemporaryDirectory() as root, patch.object(runtime, 'urlopen', return_value=io.BytesIO(data)), \
             patch.object(runtime, 'ARCHIVE_SIZE', len(data)), patch.object(runtime, 'ARCHIVE_SHA256', hashlib.sha256(data).hexdigest()):
            destination = Path(root) / 'runtime'
            runtime.stage(destination)
            self.assertEqual((destination / 'vulkan-1.dll').read_bytes(), loader)
            self.assertEqual((destination / 'VulkanRT-License.txt').read_bytes(), b'Synthetic license\r\nPreserve exact bytes\r\n')
            self.assertEqual({p.name for p in destination.iterdir()}, {'vulkan-1.dll', 'VulkanRT-License.txt'})

    def test_rejects_x86_binary_even_under_x64_archive_path(self):
        data, _ = fixture(0x14c)
        with tempfile.TemporaryDirectory() as root, patch.object(runtime, 'urlopen', return_value=io.BytesIO(data)), \
             patch.object(runtime, 'ARCHIVE_SIZE', len(data)), patch.object(runtime, 'ARCHIVE_SHA256', hashlib.sha256(data).hexdigest()):
            with self.assertRaisesRegex(RuntimeError, 'not x64'):
                runtime.stage(Path(root) / 'runtime')


if __name__ == '__main__':
    unittest.main()
