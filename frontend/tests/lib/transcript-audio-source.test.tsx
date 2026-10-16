import { describe, expect, test } from 'bun:test';
import { renderToStaticMarkup } from 'react-dom/server';
import { AudioSourceBadge } from '../../src/components/AudioSourceBadge';
import { audioSourceLabels, mergeTranscript, normalizeAudioSource, transcriptToSegment } from '../../src/lib/transcript-audio-source';
import type { AudioSource, Transcript } from '../../src/types';

const sources: AudioSource[] = ['microphone', 'system', 'mixed', 'unknown'];
const passage = (audio_source?: AudioSource): Transcript => ({
  id: 'transcript-session-0', text: 'Fixture speech', timestamp: '12:00:00',
  sequence_id: 0, audio_start_time: 12.5, audio_end_time: 13, audio_source,
});

describe('transcript capture provenance', () => {
  test('out-of-order revisions cannot roll back a live tail or final text', () => {
    const passage = { id: 'stable-tail', text: 'new tail', timestamp: '12:00:00', is_partial: true, update_revision: 3 };
    expect(mergeTranscript(passage, { ...passage, text: 'old tail', update_revision: 2 }).text).toBe('new tail');
    const final = mergeTranscript(passage, { ...passage, text: 'final wording', is_partial: false, update_revision: 4 });
    expect(mergeTranscript(final, { ...passage, text: 'late partial', update_revision: 5 }).text).toBe('final wording');
    expect(mergeTranscript(final, { ...final, text: 'stale final', update_revision: 2 }).text).toBe('final wording');
  });
  test('all sources survive JSON/recovery and display adaptation with timings intact', () => {
    for (const source of sources) {
      const saved = JSON.parse(JSON.stringify(passage(source)));
      const segment = transcriptToSegment(saved);
      expect(segment.audio_source).toBe(source);
      expect(segment.timestamp).toBe(12.5);
      expect(segment.endTime).toBe(13);
      expect(segment.id).toBe('transcript-session-0');
    }
  });

  test('absent, unsupported and generic Audio provenance never becomes Microphone', () => {
    for (const value of [undefined, null, 'Audio', 'Alice', 'speaker', 1, {}]) {
      expect(normalizeAudioSource(value)).toBe('unknown');
      expect(renderToStaticMarkup(<AudioSourceBadge source={value} />)).toContain('Audio source: Unknown');
    }
    expect(normalizeAudioSource('mic')).toBe('microphone');
    expect(transcriptToSegment(passage()).audio_source).toBe('unknown');
  });

  test('partial-to-final and duplicate updates retain source and stable identity', () => {
    const partial = { ...passage('system'), is_partial: true, text: 'Draft' };
    const final = mergeTranscript(partial, { ...passage(), id: 'temporary', is_partial: false, text: 'Final' });
    expect(final.id).toBe(partial.id);
    expect(final.audio_source).toBe('system');
    expect(final.text).toBe('Final');
    expect(mergeTranscript(final, partial)).toEqual(final);
    expect(mergeTranscript(final, final)).toEqual(final);
  });

  test('each compact badge has an icon, accessible channel label and hover description', () => {
    for (const source of sources) {
      const html = renderToStaticMarkup(<AudioSourceBadge source={source} />);
      expect(html).toContain(`Audio source: ${audioSourceLabels[source]}`);
      expect(html).toContain('role="img"');
      expect(html).toContain(`title="Capture channel: ${audioSourceLabels[source]}"`);
      expect(html).not.toContain(`>${audioSourceLabels[source]}</span>`);
      expect(html).toContain('aria-hidden="true"');
    }
  });
});
