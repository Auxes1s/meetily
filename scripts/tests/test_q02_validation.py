import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('q02', Path(__file__).parents[1] / 'run-q02-validation.py')
q02 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(q02)


class ReplayMetrics(unittest.TestCase):
    def test_every_attempt_requires_a_unique_acknowledgment_and_exported_marker(self):
        ui = {'edit_attempts': 2, 'skipped_busy_edit_ticks': 0, 'runtime_errors': [],
              'edit_receipts': [{'id': f'q02-edit-{i}', 'inserted': True, 'saved': True} for i in (1, 2)]}
        notes = ' q02-edit-1  q02-edit-2 '
        self.assertEqual(q02.verify_edit_accounting(ui, notes)['exported'], 2)
        with self.assertRaisesRegex(RuntimeError, 'Export lost'):
            q02.verify_edit_accounting(ui, ' q02-edit-1  q02-edit-20 ')
        ui['edit_receipts'][1]['saved'] = False
        with self.assertRaisesRegex(RuntimeError, 'not acknowledged'):
            q02.verify_edit_accounting(ui, notes)
        ui['edit_receipts'][1] = dict(ui['edit_receipts'][0])
        with self.assertRaisesRegex(RuntimeError, 'duplicate receipts'):
            q02.verify_edit_accounting(ui, notes)

    def test_missing_edit_instrumentation_does_not_imply_coverage(self):
        self.assertEqual(q02.verify_edit_accounting({}, ''), {'instrumented': False})

    def test_backlog_keeps_microphone_and_system_independent(self):
        rows = [
            {'kind': 'input', 'source': 'microphone', 'audio_end': 5},
            {'kind': 'input', 'source': 'system', 'audio_end': 5},
            {'kind': 'decode', 'source': 'microphone', 'audio_end': 5, 'wall_seconds': 6, 'window_seconds': 5, 'inference_seconds': 1},
            {'kind': 'decode', 'source': 'system', 'audio_end': 5, 'wall_seconds': 7, 'window_seconds': 5, 'inference_seconds': 1},
        ]
        result = q02.summarize({'rows': rows}, {'audio_seconds': 10})
        self.assertEqual([r['seconds'] for r in result['backlog_trace']], [5, 0])
        self.assertEqual(result['inference_rtf'], .2)
        self.assertFalse(result['sustained_gate_passed'])
        self.assertFalse(result['fidelity_gate_passed'])

    def test_missing_latency_and_tiny_samples_do_not_become_a_pass(self):
        self.assertIsNone(q02.percentile([]))
        self.assertEqual(q02.percentile([1, 2, 100]), 100)
        self.assertIsNone(q02.summarize({'rows': []}, {'audio_seconds': 10})['p95_capture_to_native_event_seconds'])

    def test_continuity_distinguishes_empty_input_busy_decode_and_stranded_audio(self):
        rows = [
            {'kind': 'worker_wait', 'wait_seconds': 2, 'idle_with_queued_audio_seconds': .001,
             'fresh_buffered_audio_at_wait': False, 'queued_chunks_at_wait': 0},
            {'kind': 'worker_wait', 'wait_seconds': .0001, 'idle_with_queued_audio_seconds': .0001,
             'fresh_buffered_audio_at_wait': True, 'queued_chunks_at_wait': 2},
            {'kind': 'worker_wait', 'wait_seconds': 3, 'idle_with_queued_audio_seconds': .001,
             'fresh_buffered_audio_at_wait': True, 'queued_chunks_at_wait': 0},
            {'kind': 'decode', 'source': 'system', 'audio_end': 5, 'wall_seconds': 6,
             'window_seconds': 5, 'inference_seconds': 4, 'fresh_source_audio': True},
        ]
        result = q02.summarize({'rows': rows}, {'audio_seconds': 10, 'wall_seconds': 10})['processing_continuity']
        self.assertEqual(result['stranded_fresh_buffer_wait_count'], 1)
        self.assertEqual(result['stranded_fresh_buffer_wait_seconds'], 3)
        self.assertEqual(result['decode_busy_fraction_of_session_wall'], .4)
        self.assertAlmostEqual(result['idle_with_queued_audio_seconds'], .0021)

    def test_finalization_is_distinct_from_redecoding_unchanged_previews(self):
        rows = [{'kind': 'decode', 'source': 'system', 'audio_end': 5, 'wall_seconds': 6,
                 'window_seconds': 5, 'inference_seconds': 1, 'fresh_source_audio': False,
                 'finalization_decode': final} for final in [False, True]]
        result = q02.summarize({'rows': rows}, {'audio_seconds': 10})['processing_continuity']
        self.assertEqual(result['unchanged_preview_decodes'], 1)
        self.assertFalse(result['instrumented'])

    def test_pcm_backlog_excludes_silence_and_a_late_source_start(self):
        rows = [
            {'kind': 'input', 'source': 'microphone', 'audio_end': 5, 'samples': 32000},
            {'kind': 'input', 'source': 'system', 'audio_end': 101, 'samples': 32000},
            {'kind': 'input', 'source': 'system', 'audio_end': 101, 'samples': 16000},
            {'kind': 'decode', 'source': 'microphone', 'audio_end': 5, 'wall_seconds': 120,
             'window_seconds': 2, 'inference_seconds': 1},
            {'kind': 'decode', 'source': 'system', 'audio_end': 101, 'wall_seconds': 121,
             'window_seconds': 2, 'inference_seconds': 1},
        ]
        result = q02.summarize({'rows': rows}, {'audio_seconds': 200})
        self.assertEqual(result['max_decode_backlog_seconds'], 101)
        self.assertEqual(result['delivered_pcm_backlog']['max_seconds'], 2)
        self.assertTrue(result['delivered_pcm_backlog']['measured'])

    def test_windows_launch_requires_build_manifest_before_startup(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(q02.sys, 'platform', 'win32'):
            binary = Path(directory) / 'meetily.exe'
            binary.write_bytes(b'unit fixture, not a native executable')
            with self.assertRaisesRegex(RuntimeError, 'requires a Q02 build manifest'):
                q02.bundle_paths(binary)
            manifest = {'identifier': 'com.meetily.ai', 'native_validation_feature': True,
                        'binary': binary.name, 'binary_sha256': q02.sha256(binary)}
            (binary.parent / 'q02-validation-package.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(RuntimeError, 'manifest identity'):
                q02.bundle_paths(binary)

    def test_windows_manifest_rejects_changed_binary_and_missing_feature(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(q02.sys, 'platform', 'win32'):
            binary = Path(directory) / 'meetily.exe'
            binary.write_bytes(b'unit fixture, not a native executable')
            manifest = {'identifier': q02.IDENTIFIER, 'native_validation_feature': True,
                        'binary': binary.name, 'binary_sha256': q02.sha256(binary)}
            path = binary.parent / 'q02-validation-package.json'
            path.write_text(json.dumps(manifest))
            self.assertEqual(q02.bundle_paths(binary)[0], binary)
            binary.write_bytes(b'different binary')
            with self.assertRaisesRegex(RuntimeError, 'binary hash differs'):
                q02.bundle_paths(binary)
            manifest['binary_sha256'] = q02.sha256(binary)
            manifest['native_validation_feature'] = False
            path.write_text(json.dumps(manifest))
            with self.assertRaisesRegex(RuntimeError, 'feature'):
                q02.bundle_paths(binary)


if __name__ == '__main__':
    unittest.main()
