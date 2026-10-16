import { describe, expect, test } from 'bun:test';
import { localDateBound, manualMetadataLines, meetingSourceUrl, passageTime } from '../../src/lib/meeting-library';

describe('meeting library navigation and filter values', () => {
  test('source links preserve exact source identity and revision through URL encoding', () => {
    const url = new URL(meetingSourceUrl('meeting/漢?', {
      source_kind: 'approved', source_id: 'version & one', revision: 7,
      snippet: 'Private note text', audio_start_time: null, audio_end_time: null,
    }), 'https://meetily.invalid');
    expect(url.pathname).toBe('/meeting-details');
    expect(Object.fromEntries(url.searchParams)).toEqual({
      id: 'meeting/漢?', source: 'approved', sourceId: 'version & one', sourceRevision: '7',
    });
    expect(url.toString()).not.toContain('Private');
  });

  test('date boundaries use local calendar midnight and an exclusive next day', () => {
    const from = new Date(localDateBound('2026-10-13')!);
    const to = new Date(localDateBound('2026-10-13', true)!);
    expect([from.getFullYear(), from.getMonth(), from.getDate(), from.getHours()]).toEqual([2026, 9, 13, 0]);
    expect([to.getFullYear(), to.getMonth(), to.getDate(), to.getHours()]).toEqual([2026, 9, 14, 0]);
    expect(localDateBound('')).toBeNull();
    expect(() => localDateBound('2026-02-30')).toThrow('valid date');
    expect(() => localDateBound('tomorrow')).toThrow('valid date');
  });

  test('manual labels preserve Unicode and unavailable passage time stays explicit', () => {
    expect(manualMetadataLines(' María Ñ \r\n\r\n 技術 ')).toEqual(['María Ñ', '技術']);
    expect(passageTime(125.8)).toBe('2:05');
    expect(passageTime(null)).toBe('Time unavailable');
    expect(passageTime(-1)).toBe('Time unavailable');
    expect(passageTime(Number.NaN)).toBe('Time unavailable');
  });
});
