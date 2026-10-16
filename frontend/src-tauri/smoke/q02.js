// Opt-in com.meetily.q02.test only. UI evidence is kept separate from replay.
(async () => {
  if (window.__q02Running) return;
  window.__q02Running = true;
  const invoke = (name, args) => window.__TAURI_INTERNALS__.invoke(name, args);
  const control = (action, payload) => invoke('q02_validation_control', { action, payload: payload ?? null });
  const delay = ms => new Promise(r => setTimeout(r, ms));
  const wait = async check => { for (let i = 0; i < 600; i++) { const v = await check(); if (v) return v; await delay(100); } throw Error('Q02 UI timed out'); };
  const button = label => [...document.querySelectorAll('button')].find(b => b.textContent.trim() === label && !b.disabled);
  // Background WebKit may suspend animation frames. Preserve the missing
  // measurement rather than stalling replay or calling a timer a paint.
  const paint = () => new Promise(resolve => {
    let frame, settled = false;
    const timer = setTimeout(() => { settled = true; cancelAnimationFrame(frame); resolve(false); }, 2000);
    frame = requestAnimationFrame(() => {
      if (settled) return;
      frame = requestAnimationFrame(() => { if (!settled) { settled = true; clearTimeout(timer); resolve(true); } });
    });
  });
  const insert = (editor, text) => {
    const inline = [...editor.querySelectorAll('.bn-inline-content')].at(-1) || editor.querySelector('p') || editor;
    editor.focus(); const selection = getSelection(); const range = document.createRange();
    range.selectNodeContents(inline); range.collapse(false); selection.removeAllRanges(); selection.addRange(range);
    if (!document.execCommand('insertText', false, text)) throw Error('Native editor rejected Q02 insertion');
  };
  try {
    await delay(1000);
    const cfg = await control('info');
    let state = JSON.parse(sessionStorage.getItem('q02-state') || 'null');
    if (!state) {
      const meeting = await invoke('api_create_meeting_draft', { title: 'Q02 source replay — café 日本語', timezone: 'Asia/Manila' });
      state = { id: meeting.id };
      if (cfg.stress) {
        const blocks = Array.from({ length: 1000 }, (_, i) => ({ id: `q02-block-${i}`, type: 'paragraph', props: {}, content: [{ type: 'text', text: `Stress source ${i} — café 日本語`, styles: {} }], children: [] }));
        await invoke('api_save_meeting_notes', { input: { meeting_id: meeting.id, notes_json: JSON.stringify(blocks), notes_markdown: null, notes_plain_text: null, base_revision: 0, commit_id: crypto.randomUUID(), links: [] } });
        await control('seed_stress', { meeting_id: meeting.id });
      }
      sessionStorage.setItem('q02-state', JSON.stringify(state));
      sessionStorage.setItem('workspace_meeting_id', state.id); localStorage.setItem('meetily.lastNotesMeeting', state.id);
      location.href = '/'; return;
    }
    const editor = await wait(() => document.querySelector('section[aria-label="My notes"] [contenteditable="true"]'));
    insert(editor, ' Q02 continuity original — café 日本語'); await paint();
    await wait(async () => (await invoke('api_get_meeting_notes', { meetingId: state.id }))?.notes_json?.includes('Q02 continuity original'));
    const original = await invoke('api_get_meeting_notes', { meetingId: state.id });
    const ui = { synthetic_input_to_paint_ms: [], foreground_input_to_paint_ms: [], observed_save_ms: [], capture_to_event_ms: [], visible_text_samples: 0, capture_to_display_ms: [], event_to_paint_ms: [], foreground_samples: 0, background_samples: 0, paint_timeouts: 0,
      edit_attempts: 0, edit_ticks: 0, skipped_busy_edit_ticks: 0, edit_receipts: [], runtime_errors: [], health_samples: [], foreground_event_loop_delay_ms: [], background_event_loop_delay_ms: [] };
    window.addEventListener('error', e => ui.runtime_errors.push(String(e.message)));
    window.addEventListener('unhandledrejection', e => ui.runtime_errors.push(String(e.reason)));
    let heartbeatAt = performance.now(), heartbeatForeground = document.hasFocus() && document.visibilityState === 'visible';
    const heartbeatTimer = setInterval(() => {
      const now = performance.now(), foreground = document.hasFocus() && document.visibilityState === 'visible';
      ui[foreground && heartbeatForeground ? 'foreground_event_loop_delay_ms' : 'background_event_loop_delay_ms'].push(Math.max(0, now - heartbeatAt - 250));
      heartbeatAt = now; heartbeatForeground = foreground;
    }, 250);
    let offset = null, active = true, typing = false;
    const unlisten = await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', {
      event: 'q02-transcript', target: { kind: 'Any' }, handler: window.__TAURI_INTERNALS__.transformCallback(async e => {
        const received = performance.now(); const { update, wall_seconds } = e.payload;
        if (!update.text.trim()) return;
        ui.capture_to_event_ms.push(Math.max(0, (wall_seconds - update.audio_end_time) * 1000));
        if (!await paint()) { ui.paint_timeouts++; return; }
        ui.event_to_paint_ms.push(performance.now() - received);
        const visible = [...document.querySelectorAll('.transcript-scroll')].some(n => n.textContent.includes(update.text.trim()));
        if (visible && offset !== null) { ui.visible_text_samples++; ui.capture_to_display_ms.push(Math.max(0, performance.now() - offset - update.audio_end_time * 1000)); }
      })
    });
    const clockTimer = setInterval(async () => {
      const before = performance.now(); const clock = await control('clock'); const after = performance.now();
      if (clock.wall_seconds !== null && after - before < 20) offset = (before + after) / 2 - clock.wall_seconds * 1000;
    }, 1000);
    const typeTimer = setInterval(async () => {
      if (!active) return;
      ui.edit_ticks++;
      if (typing) { ui.skipped_busy_edit_ticks++; return; }
      typing = true;
      const receipt = { id: `q02-edit-${++ui.edit_attempts}`, at_ms: performance.now(), foreground: document.hasFocus() && document.visibilityState === 'visible', inserted: false, saved: false, painted: false };
      ui.edit_receipts.push(receipt);
      try {
        const foreground = receipt.foreground;
        const text = ` ${receipt.id} `; const started = performance.now();
        insert(editor, text);
        receipt.inserted = true;
        const saved = wait(async () => (await invoke('api_get_meeting_notes', { meetingId: state.id }))?.notes_json?.includes(text))
          .then(() => ({ ms: performance.now() - started }), error => ({ error: String(error) }));
        const painted = await paint(); const elapsed = performance.now() - started;
        receipt.painted = painted;
        receipt.paint_ms = painted ? elapsed : null;
        if (painted) {
          ui.synthetic_input_to_paint_ms.push(elapsed);
          if (foreground && document.hasFocus()) { ui.foreground_samples++; ui.foreground_input_to_paint_ms.push(elapsed); }
          else ui.background_samples++;
        } else ui.paint_timeouts++;
        const acknowledgment = await saved;
        if (acknowledgment.error) throw Error(acknowledgment.error);
        receipt.saved = true; receipt.save_ms = acknowledgment.ms;
        ui.observed_save_ms.push(acknowledgment.ms);
      } catch (error) { receipt.error = String(error); }
      finally {
        typing = false;
        try {
          const health = await control('health');
          ui.health_samples.push({ at_ms: performance.now(), ...health });
          await control('ui_progress', { edit_attempts: ui.edit_attempts, acknowledged_edits: ui.edit_receipts.filter(r => r.saved).length, failed_edits: ui.edit_receipts.filter(r => r.error).length, foreground_samples: ui.foreground_samples, background_samples: ui.background_samples, health });
        } catch (error) { ui.runtime_errors.push(String(error)); }
      }
    }, 5000);
    // Opening the real transcript surface permits actual visible-text samples.
    button('Show transcript')?.click();
    const results = [];
    for (let i = 0; i < (cfg.two_sessions ? 2 : 1); i++) {
      results.push(await invoke('q02_validation_replay', { meetingId: state.id }));
    }
    active = false; clearInterval(typeTimer); clearInterval(clockTimer); clearInterval(heartbeatTimer);
    await wait(() => !typing); await delay(500);
    await invoke('plugin:event|unlisten', { event: 'q02-transcript', eventId: unlisten });
    const final = await invoke('api_get_meeting_notes', { meetingId: state.id });
    if (!final.notes_json.includes('Q02 continuity original')) throw Error('Replay lost original notes');
    const lostEdits = ui.edit_receipts.filter(r => r.inserted && !final.notes_json.includes(` ${r.id} `)).map(r => r.id);
    const editAccounting = { attempts: ui.edit_attempts, inserted: ui.edit_receipts.filter(r => r.inserted).length, acknowledged: ui.edit_receipts.filter(r => r.saved).length, failed: ui.edit_receipts.filter(r => r.error).length, lost_edits: lostEdits, all_attempts_accounted: ui.edit_attempts === ui.edit_receipts.length && ui.edit_receipts.every(r => r.saved || r.error) };
    const sessions = await invoke('api_get_recording_sessions', { meetingId: state.id });
    const exports = await control('export', { meeting_id: state.id });
    const sidecar = cfg.sidecar_probe ? await control('sidecar') : null;
    const report = { ok: lostEdits.length === 0 && editAccounting.failed === 0 && editAccounting.all_attempts_accounted && ui.runtime_errors.length === 0, results, ui, edit_accounting: editAccounting, final_worker_health: await control('health'), sessions, original_revision: original.revision, final_revision: final.revision, native_webview: true,
      exports,
      sidecar,
      two_sessions: cfg.two_sessions === true, physical_capture: false, model_quality_acceptance: false,
      notes_preserved: true, stress_notepad_blocks: cfg.stress ? 1000 : null, synthetic_stored_transcripts: cfg.stress ? 10000 : null };
    await control('report', report); await control('exit');
  } catch (error) {
    await control('report', { ok: false, error: String(error), physical_capture: false }); await control('exit');
  }
})();
