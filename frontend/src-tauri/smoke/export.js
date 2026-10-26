// Opt-in debug WebView smoke in com.meetily.export.test only. The operator
// cancels the first native Save dialog, then saves JSON and Markdown files.
(async () => {
  const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
  const report = value => invoke('notepad_smoke_report', { report: value });
  const wait = async (check, label) => {
    for (let i = 0; i < 480; i++) {
      const value = await check(); if (value) return value;
      await new Promise(resolve => setTimeout(resolve, 250));
    }
    throw new Error(`Timed out: ${label}`);
  };
  const button = label => [...document.querySelectorAll('button')].find(node => node.textContent.trim() === label);
  const click = label => {
    const node = button(label); if (!node) throw new Error(`Missing button: ${label}`); node.click();
  };
  const select = (label, value) => {
    const node = document.querySelector(`select[aria-label="${label}"]`);
    Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, 'value').set.call(node, value);
    node.dispatchEvent(new Event('change', { bubbles: true }));
  };
  try {
    await new Promise(resolve => setTimeout(resolve, 1500));
    let state = JSON.parse(sessionStorage.getItem('meeting-export-smoke') || 'null');
    if (state?.phase === 'finished') return;
    const onboarding = await invoke('get_onboarding_status');
    if (!onboarding?.completed) {
      await wait(() => button('Use notes only'), 'notes-only onboarding'); click('Use notes only'); return;
    }
    if (!state) {
      await report({ ok: false, phase: 'seeding', isolated: true });
      const meeting = await invoke('api_create_meeting_draft', { title: 'Native export smoke — 日本語', timezone: 'Asia/Manila' });
      const recordingSessionId = crypto.randomUUID();
      await invoke('api_save_transcript', { meetingTitle: meeting.title, meetingId: meeting.id, recordingSessionId, folderPath: null,
        transcripts: Array.from({ length: 1207 }, (_, i) => ({ id: `fixture-${i}`, text: `Offline source ${i} — Taglish 日本語`, timestamp: '12:00:00', audio_start_time: i, audio_end_time: i + 1, duration: 1, sequence_id: i, audio_source: ['microphone', 'system', 'mixed', 'unknown'][i % 4] })) });
      await invoke('api_update_meeting_metadata', { meetingId: meeting.id, participants: ['José'], tags: ['予算'] });
      state = { phase: 'workspace', id: meeting.id };
      sessionStorage.setItem('meeting-export-smoke', JSON.stringify(state));
      sessionStorage.setItem('workspace_meeting_id', meeting.id);
      localStorage.setItem('meetily.lastNotesMeeting', meeting.id);
      location.href = '/'; return;
    }
    if (state.phase === 'library') {
      await wait(() => [...document.querySelectorAll('article')].some(node => node.textContent.includes('Native export smoke — 日本語')), 'seeded library meeting');
      const row = [...document.querySelectorAll('article')].find(node => node.textContent.includes('Native export smoke — 日本語'));
      row.querySelector('[aria-label="Export meeting"]').click();
      await wait(() => document.querySelector('select[aria-label="Notes to include"]'), 'library export choices');
      if (document.querySelector('select[aria-label="Notes to include"]').value !== 'original') throw new Error('Legacy/absent approval was selected as approved');
      await report({ ok: true, phase: 'finished', native_webview: true, isolated: true, meeting_id: state.id, dialog_cancel: true, json_saved: true, markdown_saved: true, library_export_controls: true, transcript_count: 1207 });
      sessionStorage.setItem('meeting-export-smoke', JSON.stringify({ ...state, phase: 'finished' })); return;
    }
    const editor = await wait(() => document.querySelector('section[aria-label="My notes"] [contenteditable="true"]'), 'original editor');
    editor.focus(); document.execCommand('insertText', false, 'Latest native export notes — café 日本語');
    document.querySelector('[aria-label="Export meeting"]').click();
    await wait(() => document.querySelector('select[aria-label="Export format"]'), 'export choices');
    select('Export format', 'json');
    await new Promise(resolve => setTimeout(resolve, 100));
    click('Save export…');
    await report({ ok: false, phase: 'awaiting-cancel', meeting_id: state.id });
    await wait(() => document.body.innerText.includes('Export cancelled.'), 'native dialog cancellation');
    const saved = await invoke('api_get_meeting_notes', { meetingId: state.id });
    if (!saved.notes_json.includes('Latest native export notes — café 日本語')) throw new Error('Export did not acknowledge the latest editor edit');
    click('Save export…');
    await report({ ok: false, phase: 'awaiting-json-save', meeting_id: state.id, revision: saved.revision });
    await wait(() => document.body.innerText.includes('Export saved.'), 'native JSON save');
    select('Export format', 'markdown');
    await new Promise(resolve => setTimeout(resolve, 100));
    const checkbox = document.querySelector('[role="dialog"] input[type="checkbox"]'); checkbox.click();
    click('Save export…');
    await report({ ok: false, phase: 'awaiting-markdown-save', meeting_id: state.id });
    // The prior success notice is cleared synchronously by the new save.
    await wait(() => document.body.innerText.includes('Export saved.'), 'native Markdown save');
    sessionStorage.setItem('meeting-export-smoke', JSON.stringify({ ...state, phase: 'library' }));
    location.href = '/library';
  } catch (error) {
    await report({ ok: false, phase: 'failed', error: String(error), body: document.body.innerText.slice(0, 2500), native_webview: true }).catch(() => {});
  }
})();
