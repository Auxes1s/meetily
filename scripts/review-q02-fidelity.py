#!/usr/bin/env python3
"""Diagnostic alignment against a pre-authored source; never invent audio review."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import unicodedata


def words(text):
    return re.findall(r"[^\W_]+", unicodedata.normalize('NFC', text).casefold())


def distance(a, b):
    row = list(range(len(b) + 1))
    for i, x in enumerate(a, 1):
        next_row = [i]
        for j, y in enumerate(b, 1):
            next_row.append(min(next_row[-1] + 1, row[j] + 1, row[j - 1] + (x != y)))
        row = next_row
    return row[-1]


def compare(reference, updates, pcm_hash, channel):
    finals = [u for u in updates if not u['is_partial'] and u['audio_source'] == channel and u['text'].strip()]
    finals.sort(key=lambda u: (u['audio_start_time'], u['sequence_id']))
    output = ' '.join(u['text'] for u in finals)
    expected = words(reference['text'])
    actual = words(output)
    matching_audio = pcm_hash == reference['pcm_sha256']
    reviewed = reference.get('independent_audio_review', {}).get('status') == 'reviewed'
    errors = distance(expected, actual)
    return {'reference_words': len(expected), 'output_words': len(actual), 'word_edit_distance': errors,
            'diagnostic_word_error_rate': errors / max(1, len(expected)),
            'reference_audio_matches': matching_audio, 'independent_audio_review_complete': reviewed,
            'repeated_yes_retained': 'yes yes' in ' '.join(actual),
            'repeated_repeat_retained': 'repeat repeat' in ' '.join(actual),
            'utf8_replacement_characters': output.count('\ufffd'),
            'accuracy_acceptance': matching_audio and reviewed and errors == 0,
            'comparison_policy': 'NFC and casefold; punctuation excluded; accents retained; source audio review required',
            'output': output}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--transcripts', type=Path, required=True)
    parser.add_argument('--source-pcm', type=Path, required=True)
    parser.add_argument('--channel', choices=('microphone', 'system'), default='microphone')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = compare(json.loads(args.reference.read_text()), json.loads(args.transcripts.read_text()),
                     hashlib.sha256(args.source_pcm.read_bytes()).hexdigest(), args.channel)
    args.output.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({k:v for k,v in result.items() if k != 'output'}))


if __name__ == '__main__':
    main()
