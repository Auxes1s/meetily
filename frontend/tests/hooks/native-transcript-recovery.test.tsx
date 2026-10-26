import { afterAll, afterEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
const originals = {
  core: { ...await import('@tauri-apps/api/core') }, cache: { ...await import('../../src/services/indexedDBService') },
  storage: { ...await import('../../src/services/storageService') }, preferences: { ...await import('../../src/lib/summary-language-preferences') },
};
let failJournal = false, audioStatus = 'success', saved: any[] = [], acknowledged = false;
const commands: string[] = [];
const passage = (sequence: number, text: string) => ({ id: `native-${sequence}`, text, sequence_id: sequence, audio_source: 'system', display_time: '00:00', confidence: 0.9, audio_start_time: sequence, audio_end_time: sequence + 1, duration: 1 });
mock.module('@tauri-apps/api/core', () => ({ ...originals.core, invoke: async (command: string) => {
  commands.push(command);
  if (command === 'recover_transcript_history') { if (failJournal) throw new Error('Journal ownership mismatch'); return [passage(0,'Final revision'),passage(1,'Missing from cache')]; }
  if (command === 'recover_audio_from_checkpoints') return { status: audioStatus, chunk_count: 1, estimated_duration_seconds: 2, message: 'fixture' };
  if (command === 'cleanup_checkpoints') return;
  throw new Error(`Unexpected command ${command}`);
} }));
mock.module('../../src/services/indexedDBService', () => ({ ...originals.cache, indexedDBService: {
  getMeetingMetadata: async () => ({ canonicalMeetingId: 'canonical', recordingSessionId: 'session', title: 'Fixture', folderPath: '/synthetic/recording' }),
  getTranscripts: async () => [{ id: 'cached', sequenceId: 0, text: 'Provisional', timestamp: '00:00', storedAt: 0, confidence: 0.5, is_partial: true }],
  markMeetingSaved: async () => { acknowledged = true; },
} }));
mock.module('../../src/services/storageService', () => ({ ...originals.storage, storageService: { saveMeeting: async (...input: any[]) => { saved = input; return { meeting_id: 'canonical' }; } } }));
mock.module('../../src/lib/summary-language-preferences', () => ({ ...originals.preferences, applyPinnedSummaryLanguageToMeeting: async () => {} }));
const { useTranscriptRecovery } = await import('../../src/hooks/useTranscriptRecovery');
let state: ReturnType<typeof useTranscriptRecovery>, renderer: ReactTestRenderer | undefined;
function View() { state = useTranscriptRecovery(); return null; }
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; failJournal = false; audioStatus = 'success'; saved = []; acknowledged = false; commands.length = 0; });
afterAll(() => {
  mock.module('@tauri-apps/api/core', () => originals.core); mock.module('../../src/services/indexedDBService', () => originals.cache);
  mock.module('../../src/services/storageService', () => originals.storage); mock.module('../../src/lib/summary-language-preferences', () => originals.preferences);
});
async function show() { await act(async () => { renderer = create(<View />); }); }
test('recovery commits native final passages before acknowledging and clearing checkpoints', async () => {
  await show(); await act(async () => { await state.recoverMeeting('cache-key'); });
  expect(saved[1].map((row: any) => [row.text,row.is_partial,row.audio_source])).toEqual([['Final revision',false,'system'],['Missing from cache',false,'system']]);
  expect(acknowledged).toBe(true); expect(commands).toEqual(['recover_transcript_history','recover_audio_from_checkpoints','cleanup_checkpoints']);
});
test('invalid journal ownership leaves cache acknowledgments and checkpoints untouched', async () => {
  failJournal = true; await show();
  await act(async () => { await expect(state.recoverMeeting('cache-key')).rejects.toThrow('ownership'); });
  expect(saved).toEqual([]); expect(acknowledged).toBe(false); expect(commands).toEqual(['recover_transcript_history']);
});
test('failed audio reconstruction retains checkpoints after transcript recovery', async () => {
  audioStatus = 'failed'; await show(); await act(async () => { await state.recoverMeeting('cache-key'); });
  expect(saved[1]).toHaveLength(2); expect(commands).not.toContain('cleanup_checkpoints');
});
