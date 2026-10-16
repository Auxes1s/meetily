(async () => {
  const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
  const report = value => invoke('notepad_smoke_report', { report: value });
  const wait = async (check, label, attempts = 160) => {
    for (let i = 0; i < attempts; i++) { const result = await check(); if (result) return result; await new Promise(resolve => setTimeout(resolve, 125)); }
    throw new Error(`Timed out: ${label}`);
  };
  const click = label => {
    const button = [...document.querySelectorAll('button')].find(button => button.textContent.trim().startsWith(label));
    if (!button) throw new Error(`Missing button: ${label}`);
    button.click();
  };
  const panel = () => document.querySelector('#meeting-transcript-panel');
  try {
    await new Promise(resolve => setTimeout(resolve, 2000));
    if (!(await invoke('get_onboarding_status'))?.completed) {
      await wait(() => [...document.querySelectorAll('button')].some(button => button.textContent.trim() === 'Use notes only'), 'notes setup');
      click('Use notes only'); return;
    }
    const phase = localStorage.getItem('transcript-smoke-phase');
    if (phase === 'finished') return;
    if (phase === 'reopen') {
      const id = localStorage.getItem('transcript-smoke-meeting');
      if (location.pathname !== '/meeting-details') { location.href = `/meeting-details?id=${id}&source=transcript`; return; }
      const docs = await invoke('api_get_meeting_transcripts', { meetingId: id, limit: 100, offset: 0 });
      if (!docs.transcripts.length || docs.transcripts.some(segment => segment.audio_source !== 'microphone')) throw new Error('Reopened transcript lost passages or provenance');
      await wait(() => document.querySelector('[aria-label="Meeting title"]'), 'reopened original notes');
      await wait(() => panel()?.querySelector('[aria-label="Audio source: Microphone"]'), 'saved transcript pane after relaunch');
      const notes = await invoke('api_get_meeting_notes', { meetingId: id });
      if (!notes?.notes_json.includes('Notes stay saved during speech')) throw new Error('Notes lost during transcript test');
      await report({ ok: true, native_webview: true, real_whisper_model: 'large-v3', fixture_channel: 'microphone', live_before_stop: true, collapse_reopen: true, saved_after_stop: true, relaunch_persistence: true, notes_preserved: true, segments: docs.transcripts.length });
      localStorage.setItem('transcript-smoke-phase', 'finished');
      return;
    }
    await wait(() => [...document.querySelectorAll('button')].some(button => button.textContent.trim() === 'New meeting'), 'workspace');
    const previous = sessionStorage.getItem('workspace_meeting_id');
    click('New meeting');
    await wait(() => sessionStorage.getItem('workspace_meeting_id') !== previous && document.querySelector('[aria-label="Meeting title"]'), 'new speech draft');
    const editor = await wait(() => document.querySelector('section[aria-label="My notes"] [contenteditable="true"]'), 'notes editor');
    editor.focus(); document.execCommand('insertText', false, 'Notes stay saved during speech');
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 's', metaKey: true, bubbles: true }));
    const id = sessionStorage.getItem('workspace_meeting_id');
    await wait(async () => (await invoke('api_get_meeting_notes', { meetingId: id }))?.notes_json.includes('Notes stay saved during speech'), 'original-note acknowledgment');
    if (panel().hidden) click('Show transcript');
    await report({ ok: false, phase: 'decoding-known-speech', native_webview: true });
    let failure;
    const running = invoke('transcript_smoke_run', { meetingId: id }).catch(error => { failure = error; return null; });
    await wait(() => { if (failure) throw new Error(String(failure)); return panel()?.querySelector('[aria-label="Audio source: Microphone"]'); }, 'live passage before stop', 2400);
    click('Hide transcript');
    await wait(() => [...document.querySelectorAll('button')].some(button => button.textContent.trim().startsWith('Show transcript')), 'collapsed transcript');
    click('Show transcript');
    await wait(() => !panel().hidden, 'reopened transcript');
    if (!panel().querySelector('[aria-label="Audio source: Microphone"]')) throw new Error('Collapsed pane lost its live passage');
    const result = await running;
    if (!result || !result.segments.length || result.segments.some(segment => segment.duration >= 15)) throw new Error('Expected a completed short spoken passage');
    await invoke('plugin:event|emit', { event: 'recording-stopped', payload: { meeting_id: id, recording_session_id: result.recording_session_id } });
    const saved = await invoke('api_get_meeting_transcripts', { meetingId: id, limit: 100, offset: 0 });
    if (saved.transcripts.length !== result.segments.length) throw new Error('Short final passages not saved after completion');
    localStorage.setItem('transcript-smoke-meeting', id);
    localStorage.setItem('transcript-smoke-phase', 'reopen');
    localStorage.setItem('meetily.lastNotesMeeting', id);
    sessionStorage.setItem('workspace_meeting_id', id);
    sessionStorage.removeItem('recording_meeting_id'); sessionStorage.removeItem('recording_session_id');
    await report({ ok: false, phase: 'ready-for-relaunch', live_before_stop: true, saved_after_stop: true });
    await invoke('confirm_workspace_quit');
  } catch (error) { await report({ ok: false, error: String(error), native_webview: true, buttons: [...document.querySelectorAll('button')].map(button => button.textContent.trim()).filter(Boolean).slice(0, 20) }).catch(() => {}); }
})();
