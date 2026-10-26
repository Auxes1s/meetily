(async () => {
  const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
  const wait = async (check, label) => {
    for (let i = 0; i < 160; i++) { const result = await check(); if (result) return result; await new Promise(resolve => setTimeout(resolve, 125)); }
    throw new Error(`Timed out: ${label}`);
  };
  const click = label => {
    const button = [...document.querySelectorAll('button')].find(button => button.textContent.trim() === label);
    if (!button) throw new Error(`Missing button: ${label}`);
    button.click();
  };
  try {
    await invoke('notepad_smoke_report', { report: { ok: false, phase: 'starting' } });
    await new Promise(resolve => setTimeout(resolve, 2000));
    const phase = sessionStorage.getItem('notepad-smoke-phase');
    if (phase === 'finished') return;
    const onboarding = await invoke('get_onboarding_status');
    if (!onboarding?.completed) {
      await wait(() => [...document.querySelectorAll('button')].find(button => button.textContent.trim() === 'Use notes only'), 'notes-only setup');
      click('Use notes only'); return;
    }
    await wait(() => document.querySelector('[aria-label="Meeting title"]') || [...document.querySelectorAll('button')].find(button => button.textContent.trim() === 'New meeting'), 'workspace');
    if (phase === 'reopen') {
      const id = sessionStorage.getItem('notepad-smoke-meeting');
      await wait(() => document.querySelector('[aria-label="Meeting title"]'), 'reopened notes');
      const stored = await invoke('api_get_meeting_notes', { meetingId: id });
      if (!stored.notes_json.includes('Native notepad persistence — 日本語')) throw new Error('Reopened notes lost acknowledged content');
      await invoke('notepad_smoke_report', { report: { ok: true, native_webview: true, model_free_onboarding: true, save_and_reload: true, draft_switch: true, meeting_id: id, revision: stored.revision } });
      sessionStorage.setItem('notepad-smoke-phase', 'finished');
      return;
    }
    if (!document.querySelector('[aria-label="Meeting title"]')) {
      await wait(() => [...document.querySelectorAll('button')].some(button => button.textContent.trim() === 'New meeting' && Object.keys(button).some(key => key.startsWith('__reactProps$'))), 'hydrated new-meeting button');
      click('New meeting');
    }
    const title = await wait(() => document.querySelector('[aria-label="Meeting title"]'), 'draft title');
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(title, 'Native smoke notes');
    title.dispatchEvent(new Event('input', { bubbles: true }));
    const editor = await wait(() => document.querySelector('section[aria-label="My notes"] [contenteditable="true"]'), 'BlockNote editor');
    editor.focus();
    document.execCommand('insertText', false, 'Native notepad persistence — 日本語');
    const id = sessionStorage.getItem('workspace_meeting_id');
    sessionStorage.setItem('notepad-smoke-meeting', id);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 's', metaKey: true, bubbles: true }));
    await wait(async () => { const doc = await invoke('api_get_meeting_notes', { meetingId: id }); return doc?.notes_json?.includes('Native notepad persistence — 日本語') && doc.revision > 0; }, 'acknowledged native save');
    click('New meeting');
    await wait(() => sessionStorage.getItem('workspace_meeting_id') !== id, 'second draft');
    const first = await invoke('api_get_meeting_notes', { meetingId: id });
    if (!first.notes_json.includes('Native notepad persistence — 日本語')) throw new Error('Meeting switch changed original notes');
    sessionStorage.setItem('workspace_meeting_id', id);
    localStorage.setItem('meetily.lastNotesMeeting', id);
    sessionStorage.setItem('notepad-smoke-phase', 'reopen');
    location.href = '/';
  } catch (error) {
    await invoke('notepad_smoke_report', { report: { ok: false, error: String(error), native_webview: true, body: document.body.innerText.slice(0, 2000) } }).catch(() => {});
  }
})();
