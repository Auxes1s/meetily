// Explicitly enabled isolated debug WebView check. Synthetic notes only.
(async () => {
  if (window.__summaryFixSmokeRunning) return;
  window.__summaryFixSmokeRunning = true;
  const invoke = (name, args) => window.__TAURI_INTERNALS__.invoke(name, args);
  const wait = async (check, label, steps = 240) => {
    for (let i = 0; i < steps; i++) {
      const value = await check(); if (value) return value;
      await new Promise(resolve => setTimeout(resolve, 250));
    }
    throw new Error(`Timed out: ${label}`);
  };
  const click = async label => (await wait(() => [...document.querySelectorAll('button')].find(node => node.textContent.trim() === label && !node.disabled), label)).click();
  const original = 'Unang mungkahi: panatilihin ang offline access. Napagkasunduan ang offline pilot. Mina will test the checklist; no deadline was agreed. 日本語の記録。';
  const create = async title => {
    const meeting = await invoke('api_create_meeting_draft', { title, timezone: 'Asia/Manila' });
    await invoke('api_save_meeting_notes', { input: {
      meeting_id: meeting.id, notes_json: JSON.stringify([{ id: crypto.randomUUID(), type: 'paragraph', content: [{ type: 'text', text: original }], children: [] }]),
      base_revision: 0, commit_id: crypto.randomUUID(), links: [],
    } });
    return meeting.id;
  };
  const navigate = (state, id) => {
    sessionStorage.setItem('summary-fix-smoke', JSON.stringify(state));
    sessionStorage.setItem('workspace_meeting_id', id); localStorage.setItem('meetily.lastNotesMeeting', id);
    location.href = `/meeting-details?id=${encodeURIComponent(id)}`;
  };
  const finished = async id => wait(async () => {
    const collection = await invoke('api_get_meeting_enhancements', { meetingId: id });
    const version = collection.versions[0];
    if (version?.status === 'failed') throw new Error(version.error);
    return version?.status === 'ready' && version;
  }, 'complete generated candidate', 2400);
  const preserved = async id => {
    const notes = await invoke('api_get_meeting_notes', { meetingId: id });
    if (JSON.parse(notes.notes_json)[0].content[0].text !== original) throw new Error('Generation changed original notes');
  };
  try {
    await wait(() => [...document.querySelectorAll('button')].some(node => Object.keys(node).some(key => key.startsWith('__reactProps$'))), 'hydrated WebView');
    await wait(() => invoke('api_get_meetings').then(() => true).catch(() => false), 'native database initialization');
    let state = JSON.parse(sessionStorage.getItem('summary-fix-smoke') || 'null');
    if (!state) {
      await invoke('api_save_model_config', { provider: 'builtin-ai', model: 'qwen3.5:4b', whisperModel: 'small' });
      const id = await create('Synthetic enhanced notes — Taglish 日本語');
      navigate({ phase: 'enhance', enhanced: id }, id); return;
    }
    if (state.phase === 'done') return;
    if (state.phase === 'enhance') {
      await click('Enhanced notes'); await click('Enhance notes');
      const version = await finished(state.enhanced); await preserved(state.enhanced);
      state.enhanced_version = version.id;
      state.summary = await create('Synthetic summary — notes only'); state.phase = 'summary';
      navigate(state, state.summary); return;
    }
    const toggle = [...document.querySelectorAll('button')].find(node => node.textContent.trim().startsWith('Show transcript'));
    if (toggle) toggle.click();
    await click('Summary'); await click('Generate Summary');
    const version = await finished(state.summary); await preserved(state.summary); await preserved(state.enhanced);
    await wait(() => document.body.innerText.includes('offline'), 'rendered generated summary');
    state.phase = 'done'; sessionStorage.setItem('summary-fix-smoke', JSON.stringify(state));
    await invoke('notepad_smoke_report', { report: { ok: true, native_webview: true, real_model: 'qwen3.5:4b',
      enhanced_notes_button: true, summary_button: true, notes_only: true, taglish_unicode: true, originals_preserved: true,
      enhanced_version: state.enhanced_version, summary_version: version.id } });
  } catch (error) {
    await invoke('notepad_smoke_report', { report: { ok: false, error: String(error), body: document.body.innerText.slice(0, 3000) } });
  }
})();
