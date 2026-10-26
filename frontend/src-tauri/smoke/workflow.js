// Native IPC and canonical persistence in com.meetily.workflow.test only.
(async () => {
  if (window.__meetilyWorkflowRunning) return;
  window.__meetilyWorkflowRunning = true;
  const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
  const control = (action, payload) => invoke('workflow_smoke_control', { action, payload: payload ?? null });
  const assert = (condition, message) => { if (!condition) throw new Error(message); };
  const wait = async (check, label) => {
    for (let i = 0; i < 720; i++) { const value = await check(); if (value) return value; await new Promise(r => setTimeout(r, 250)); }
    throw new Error(`Timed out: ${label}`);
  };
  const findButton = label => [...document.querySelectorAll('button')].find(n => n.textContent.trim() === label && !n.disabled);
  const click = async label => (await wait(() => findButton(label), label)).click();
  const inspect = id => control('inspect', { meeting_id: id });
  const notes = id => invoke('api_get_meeting_notes', { meetingId: id });
  const versions = id => invoke('api_get_meeting_enhancements', { meetingId: id });
  const save = state => control('checkpoint', state);
  const navigate = (state, path) => save(state).then(() => { location.href = path; });
  const insert = (editor, text) => { editor.focus(); const selection = window.getSelection(); const range = document.createRange(); const inline = [...editor.querySelectorAll('.bn-inline-content')].at(-1) || editor.querySelector('p') || editor; range.selectNodeContents(inline); range.collapse(false); selection.removeAllRanges(); selection.addRange(range); assert(document.execCommand('insertText', false, text), 'Native editor rejected text insertion'); };
  try {
    await new Promise(r => setTimeout(r, 1000));
    const info = await control('info');
    if (info.benchmark_only) {
      const report = await invoke('workflow_smoke_benchmark');
      await control('report', report); await control('exit'); return;
    }
    let state = await control('load_checkpoint');
    if (state?.phase === 'finished') return;
    const onboarding = await invoke('get_onboarding_status');
    if (!onboarding?.completed) { await click('Use notes only'); window.__meetilyWorkflowRunning = false; setTimeout(() => location.reload(), 500); return; }
    if (!state) {
      const meeting = await invoke('api_create_meeting_draft', { title: 'Q01 lantern workflow — café 日本語', timezone: 'Asia/Manila' });
      const routing = await invoke('get_continuous_transcription');
      assert(routing === info.continuous_whisper, 'Native default or saved route differs from requested route');
      state = { run_id: info.run_id, phase: 'settings', id: meeting.id, original: 'Q01 lantern source notes — café 日本語', edit: 'Q01 approved review addition', checks: {}, routing_preference: routing };
      sessionStorage.setItem('workspace_meeting_id', meeting.id); localStorage.setItem('meetily.lastNotesMeeting', meeting.id);
      await navigate(state, '/settings'); return;
    }
    if (state.phase === 'settings') {
      const betaTab = await wait(() => findButton('Beta'), 'Beta tab');
      betaTab.focus();
      betaTab.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
      const toggle = () => document.querySelector('button[aria-labelledby="continuous-transcription-label"]');
      await wait(() => toggle() && !toggle().disabled, 'continuous transcription switch');
      assert((toggle().getAttribute('aria-checked') === 'true') === state.routing_preference, 'Beta switch differs from native preference');
      for (const choice of [!state.routing_preference, state.routing_preference]) {
        toggle().click();
        await wait(async () => (await invoke('get_continuous_transcription')) === choice && !toggle().disabled && (toggle().getAttribute('aria-checked') === 'true') === choice, 'native routing preference save');
      }
      state.checks.beta_routing_switch = true;
      state.phase = 'record'; await navigate(state, '/'); return;
    }
    if (state.phase === 'record') {
      const editor = await wait(() => document.querySelector('section[aria-label="My notes"] [contenteditable="true"]'), 'original editor');
      insert(editor, state.original);
      const stored = await wait(async () => { const n = await notes(state.id); return n?.notes_json?.includes(state.original) && n; }, 'original note commit');
      const replay = await invoke('transcript_smoke_run', { meetingId: state.id });
      assert(replay.overlapping === state.routing_preference, 'Native replay ignored the Beta routing preference');
      assert(replay.segments.length > 0, 'Replay produced no finalized speech');
      const record = await inspect(state.id);
      assert(record.transcripts.length === replay.segments.length, 'Native final transcript count differs from persisted count');
      assert(record.transcripts.every(t => t.audio_source === 'microphone'), 'Replay lost capture provenance');
      await control('artifacts', { meeting_id: state.id, recording_session_id: replay.recording_session_id });
      state.notes_json = stored.notes_json; state.transcripts = record.transcripts;
      state.checks.native_pause_resume_replay = replay.pause_resume === true;
      state.checks.native_route_matches_beta = true;
      state.checks.notes_and_native_replay = true; state.phase = 'enhance'; await navigate(state, '/'); return;
    }
    if (state.phase === 'enhance') {
      const stored = await notes(state.id);
      const generated = await invoke('api_start_meeting_enhancement', { input: { meeting_id: state.id, expected_notes_revision: stored.revision, model: 'custom-openai', model_name: 'workflow-fixture', template_id: 'highlights', instructions: '', remote_destination: null, summary_language: 'en' } });
      state.version_id = generated.version_id;
      await wait(async () => { const v = await invoke('api_get_enhancement_status', { meetingId: state.id, versionId: generated.version_id }); if (v.status === 'failed') throw new Error(v.error); return v.status === 'ready'; }, 'generated candidate');
      assert((await notes(state.id)).notes_json === state.notes_json, 'Enhancement changed original notes');
      state.checks.generated_candidate = true; state.phase = 'approve'; await navigate(state, '/'); return;
    }
    if (state.phase === 'approve') {
      await click('Enhanced notes');
      let panel = document.querySelector('section[aria-label="Enhanced notes"]');
      if (!panel) { const toggle = [...document.querySelectorAll('button')].find(n => /enhanced notes/i.test(n.textContent)); if (toggle) toggle.click(); }
      const editor = await wait(() => { const e = document.querySelector('section[aria-label="Enhanced notes"] [contenteditable="true"]'); return e && !e.closest('[hidden]') && e; }, 'visible candidate editor');
      insert(editor, state.edit); await click('Save edits');
      await wait(async () => (await versions(state.id)).versions.some(v => v.id === state.version_id && v.document_json?.includes(state.edit)), 'candidate edit commit');
      await click('Approve this version');
      const approved = await wait(async () => { const v = await versions(state.id); return v.approved_version_id === state.version_id && v; }, 'approval commit');
      state.approved_json = approved.versions.find(v => v.id === state.version_id).document_json;
      assert((await notes(state.id)).notes_json === state.notes_json, 'Approval changed original notes');
      state.checks.edited_and_approved = true; state.phase = 'restart'; await save(state);
      await control('report', { ok: false, phase: 'restart', run_id: info.run_id });
      await control('exit'); return;
    }
    if (state.phase === 'restart') {
      assert((await invoke('get_continuous_transcription')) === state.routing_preference, 'Native routing preference lost across process restart');
      state.checks.routing_preference_survived_restart = true;
      const record = await inspect(state.id);
      assert(record.original_notes.notes_json === state.notes_json, 'Original notes changed across process restart');
      assert(JSON.stringify(record.transcripts) === JSON.stringify(state.transcripts), 'Transcript changed across process restart');
      const collection = await versions(state.id);
      assert(collection.approved_version_id === state.version_id && collection.versions.some(v => v.id === state.version_id && v.document_json === state.approved_json), 'Approved edit lost across restart');
      await invoke('api_sync_meeting_search', { limit: 100 });
      for (const query of ['lantern', 'approved review addition']) {
        const result = await invoke('api_search_meetings', { request: { query, limit: 100 } });
        assert(result.meetings.some(m => m.id === state.id), `Search lost ${query}`);
      }
      for (const format of ['json', 'markdown']) {
        const result = await invoke('api_export_meeting', { request: { meeting_id: state.id, format, document: 'both', include_transcript: true, expected_approved_version_id: state.version_id } });
        assert(result.status === 'saved', `Export failed: ${format}`);
      }
      if (info.copied_meeting_id) {
        const copy = await inspect(info.copied_meeting_id);
        assert(copy.transcripts.length === info.copied_transcript_count, 'Real copied transcript count changed');
        assert(copy.transcripts.every(t => t.audio_source === 'mixed'), 'Mixed recording gained invented provenance');
        await invoke('api_export_meeting', { request: { meeting_id: info.copied_meeting_id, format: 'json', document: 'original', include_transcript: true, expected_approved_version_id: null } });
        state.checks.real_copy_export = true;
      }
      state.checks.process_restart_search_export = true; state.phase = 'delete'; await navigate(state, '/library'); return;
    }
    if (state.phase === 'delete') {
      await wait(() => document.querySelector('section[aria-label="Meeting results"]'), 'library');
      for (const id of [state.id, info.copied_meeting_id].filter(Boolean)) {
        const row = await wait(() => [...document.querySelectorAll('article')].find(n => n.querySelector(`a[href*="${id}"]`)), 'meeting library row');
        const deleteButton = [...row.querySelectorAll('button')].find(n => (n.getAttribute('aria-label') || '').startsWith('Delete '));
        assert(deleteButton, 'Missing library delete action'); deleteButton.click(); await click('Delete meeting');
        await wait(async () => { const result = await invoke('api_list_meetings', { request: { limit: 100 } }); return !result.meetings.some(m => m.id === id); }, 'native deletion');
        const pending = await invoke('api_list_pending_meeting_deletions');
        assert(!pending.some(job => job.meeting_id === id), 'Managed deletion remained pending');
      }
      state.checks.managed_deletion = true; state.phase = 'finished'; await save(state);
      await control('report', { ok: true, phase: 'finished', run_id: info.run_id, checks: state.checks, native_webview: true, native_ipc: true, physical_capture: false, model_quality_acceptance: false });
      await control('exit');
    }
  } catch (error) {
    await control('report', { ok: false, phase: 'failed', error: String(error), body: document.body.innerText.slice(0, 1800) }).catch(() => {});
  }
})();
