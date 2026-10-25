import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('fidelity', Path(__file__).parents[1] / 'review-q02-fidelity.py')
f = importlib.util.module_from_spec(spec)
spec.loader.exec_module(f)


class FidelityContracts(unittest.TestCase):
    def test_exact_hash_and_text_do_not_replace_independent_audio_review(self):
        reference = {'text': 'Yes yes café 日本語', 'pcm_sha256': 'fixture', 'independent_audio_review': {'status': 'pending'}}
        output = [{'text': reference['text'], 'is_partial': False, 'audio_source': 'microphone', 'sequence_id': 1, 'audio_start_time': 0}]
        result = f.compare(reference, output, 'fixture', 'microphone')
        self.assertEqual(result['diagnostic_word_error_rate'], 0)
        self.assertTrue(result['repeated_yes_retained'])
        self.assertFalse(result['accuracy_acceptance'])
        reference['independent_audio_review']['status'] = 'reviewed'
        self.assertFalse(f.compare(reference, output, 'wrong audio', 'microphone')['accuracy_acceptance'])

    def test_accents_and_repetition_are_not_normalized_away(self):
        self.assertEqual(f.words('cafe\u0301 café 日本語'), ['café', 'café', '日本語'])
        self.assertEqual(f.distance(f.words('yes yes'), f.words('yes')), 1)
        self.assertEqual(f.distance(f.words('José'), f.words('Jose')), 1)


if __name__ == '__main__':
    unittest.main()
