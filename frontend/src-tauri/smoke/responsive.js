// Explicit opt-in, isolated debug WKWebView only. All meetings are disposable.
(async () => {
  if (window.__responsiveSmokeRunning) return;
  window.__responsiveSmokeRunning = true;
  const nativeInvoke = window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);
  const report = value => nativeInvoke('notepad_smoke_report', { report: value });
  const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
  const assert = (value, label) => { if (!value) throw new Error(label); };
  const wait = async (check, label) => {
    for (let i = 0; i < 200; i++) { const result = await check(); if (result) return result; await sleep(100); }
    throw new Error(`Timed out: ${label}`);
  };
  const visible = node => !!node && node.getClientRects().length > 0 && getComputedStyle(node).visibility !== 'hidden';
  const button = label => [...document.querySelectorAll('button')].find(node => node.textContent.trim() === label && visible(node));
  const click = label => {
    const node = button(label); assert(node, `Missing button: ${label}`);
    // Radix tabs activate on mouse-down; a programmatic click alone does not.
    node.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, button: 0 }));
    node.click();
  };
  const sizes = [[1440, 900], [1100, 700], [900, 600], [768, 600], [600, 500], [480, 400], [1100, 400]];
  let titlebarHeight;
  const resize = async (width, height, settle = 350) => {
    await nativeInvoke('plugin:window|set_size', { label: 'main', value: { Logical: { width, height } } });
    await sleep(80);
    titlebarHeight ??= height - innerHeight;
    await wait(() => innerWidth === width && innerHeight === height - titlebarHeight, `native window ${width}×${height} (viewport ${innerWidth}×${innerHeight})`);
    await sleep(settle);
  };
  const setSidebar = async expanded => {
    const toggle = document.querySelector('.sidebar-toggle');
    if ((toggle.getAttribute('aria-expanded') === 'true') !== expanded) toggle.click();
    await sleep(350);
  };
  const setDrawer = async open => {
    const toggle = document.querySelector('[aria-controls="meeting-transcript-panel"]');
    if ((toggle.getAttribute('aria-expanded') === 'true') !== open) toggle.click();
    await sleep(80);
  };
  const bounds = node => {
    const r = node.getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height, scrollWidth: node.scrollWidth, clientWidth: node.clientWidth, scrollHeight: node.scrollHeight, clientHeight: node.clientHeight };
  };
  const layoutCheck = () => {
    assert(document.documentElement.scrollWidth <= innerWidth + 1, 'Application overflows horizontally');
    const panes = document.querySelector('.workspace-panes');
    if (panes) {
      assert(panes.clientHeight >= 60, `Workspace too short: ${panes.clientHeight}`);
      const w = document.querySelector('.meeting-workspace').clientWidth;
      const expanded = document.querySelector('.sidebar-toggle').getAttribute('aria-expanded') === 'true';
      assert(w === innerWidth - (expanded && innerWidth > 700 ? 256 : 64), 'Sidebar did not reflow the available workspace width');
      const open = panes.dataset.transcriptOpen === 'true';
      assert(visible(document.querySelector('.workspace-notes')) === (!open || w >= 960), 'Wrong notes container reflow');
      assert(visible(document.querySelector('.workspace-transcript')) === open, 'Wrong transcript visibility');
      for (const node of document.querySelectorAll('.workspace-header button, .notes-toolbar button, .workspace-recording button')) {
        if (!visible(node)) continue;
        const r = node.getBoundingClientRect();
        assert(r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1, `Control outside window: ${node.getAttribute('aria-label') || node.textContent}`);
      }
      const details = document.querySelector('.meeting-details-layout');
      if (open && details) {
        assert(visible(document.querySelector('.details-tabs')) === (details.clientWidth < 900), 'Nested tabs follow the window instead of the drawer');
        assert(visible(document.querySelector('.details-separator')) === (details.clientWidth >= 900), 'Wrong nested separator visibility');
        const scroll = [...document.querySelectorAll('.transcript-scroll')].find(visible);
        if (scroll) assert(scroll.clientHeight >= 40 && scroll.scrollWidth <= scroll.clientWidth + 1, 'Transcript scroll area is clipped or widened');
      }
      for (const selector of ['.notes-scroll', '.enhancement-panel']) {
        const scroll = document.querySelector(selector);
        if (visible(scroll)) assert(scroll.clientHeight >= 60 && scroll.scrollWidth <= scroll.clientWidth + 1, `Editor scroll area is clipped or widened: ${selector}`);
      }
    }
    for (const table of document.querySelectorAll('.tableWrapper')) {
      if (visible(table)) assert(table.scrollWidth > table.clientWidth, 'Wide table was squeezed instead of scrolling locally');
    }
    const dialog = document.querySelector('[role="dialog"][data-state="open"]');
    if (dialog) {
      const r = dialog.getBoundingClientRect();
      assert(r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1, 'Dialog is outside viewport');
      assert(dialog.scrollWidth <= dialog.clientWidth + 1, 'Dialog overflows horizontally');
      dialog.scrollTop = dialog.scrollHeight;
      const save = button('Save export…');
      if (save) assert(save.getBoundingClientRect().bottom <= r.bottom + 1, 'Export action unreachable after scrolling');
      dialog.scrollTop = 0;
    }
    const library = document.querySelector('.meeting-library');
    if (library) {
      assert(library.scrollWidth <= library.clientWidth + 1, 'Library overflows horizontally');
      library.scrollTop = library.scrollHeight;
      assert(visible(document.querySelector('[aria-label="Meeting result pages"]')), 'Library pagination missing');
      library.scrollTop = 0;
    }
  };
  const snapshot = async (state, name, extra = {}) => {
    // WKWebView snapshots do not require stealing desktop focus.
    await sleep(200);
    await report({ phase: 'snapshot', screenshot: name, native_webview: true, isolated: true, meeting_id: state.id, cases: state.cases, ...extra });
    await sleep(1200);
  };
  let releaseSaves = () => {};
  let state;
  let interaction = false;
  const runtimeErrors = [];
  const saveRuntimeErrors = () => sessionStorage.setItem('interaction-runtime-errors', JSON.stringify(runtimeErrors));
  const persistInteraction = () => { state.runtimeErrors = runtimeErrors; sessionStorage.setItem('responsive-smoke', JSON.stringify(state)); };
  const interactionCase = async value => { state.interactionCases ??= []; state.interactionCases.push(value); persistInteraction(); await report({ phase: 'interaction-case', cases: state.cases, interaction_cases: state.interactionCases, case: value }); };
  const checkRuntime = () => assert(runtimeErrors.length === 0, `Unhandled interaction runtime errors: ${JSON.stringify(runtimeErrors)}`);
  const escape = () => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
  const intercept = handlers => {
    const originalFetch = window.fetch.bind(window);
    const ipc = window.webkit.messageHandlers.ipc;
    const originalPost = ipc.postMessage.bind(ipc);
    const response = (result, ok = true) => new Response(JSON.stringify(result), { status: 200, headers: { 'Content-Type': 'application/json', 'Tauri-Response': ok ? 'ok' : 'error' } });
    const handler = command => handlers[command];
    window.fetch = async (input, options) => {
      const url = String(input instanceof Request ? input.url : input);
      if (url.startsWith('ipc://')) {
        const command = decodeURIComponent(new URL(url).pathname.slice(1));
        if (handler(command)) {
          try { const result = await handler(command)(JSON.parse(options?.body || '{}')); if (result !== undefined) return response(result); }
          catch (error) { return response(String(error), false); }
        }
      }
      return originalFetch(input, options);
    };
    ipc.postMessage = async payload => {
      const message = typeof payload === 'string' ? JSON.parse(payload) : payload;
      if (handler(message.cmd)) {
        try {
          const result = await handler(message.cmd)(message.payload || {});
          if (result !== undefined) { window.__TAURI_INTERNALS__.runCallback(message.callback, result); return; }
        } catch (error) { window.__TAURI_INTERNALS__.runCallback(message.error, String(error)); return; }
      }
      originalPost(payload);
    };
    return () => { window.fetch = originalFetch; ipc.postMessage = originalPost; };
  };
  const gate = () => {
    let release, reject;
    const promise = new Promise((resolve, fail) => { release = resolve; reject = fail; });
    return { promise, release, reject };
  };
  try {
    await sleep(1500);
    // Store is initialized by the app; the fs plugin is not. Only read this
    // explicit test-profile opt-in. Disable autosave and never mutate the store.
    const interactionStore = await nativeInvoke('plugin:store|load', { path: 'interaction-smoke.json', options: { autoSave: false } });
    const [interactionEnabled, interactionExists] = await nativeInvoke('plugin:store|get', { rid: interactionStore, key: 'enabled' });
    interaction = interactionExists && interactionEnabled === true;
    if (interaction) {
      runtimeErrors.push(...JSON.parse(sessionStorage.getItem('interaction-runtime-errors') || '[]'));
      const capture = (kind, value) => { runtimeErrors.push({ kind, message: String(value) }); saveRuntimeErrors(); };
      window.addEventListener('error', event => capture('error', event.error || event.message));
      window.addEventListener('unhandledrejection', event => capture('unhandledrejection', event.reason));
      // Global fixture safety: lifecycle and export cannot reach native. A
      // narrower export mock installed below resolves the expected request.
      intercept(Object.fromEntries(['start_recording', 'start_recording_with_devices', 'start_recording_with_devices_and_meeting', 'stop_recording', 'pause_recording', 'resume_recording', 'cancel_recording_start', 'api_export_meeting', 'api_export_meeting_notes'].map(command => [command, () => { capture('forbidden-native-command', command); throw new Error(`Forbidden native side effect in interaction fixture: ${command}`); }])));
    }
    const onboarding = await nativeInvoke('get_onboarding_status');
    if (!onboarding?.completed) {
      await wait(() => button('Use notes only'), 'notes-only onboarding');
      for (const [width, height] of [[1100, 700], [480, 400]]) {
        await resize(width, height);
        assert(document.documentElement.scrollWidth <= innerWidth + 1, 'Onboarding overflows horizontally');
        button('Use notes only').scrollIntoView({ block: 'nearest' });
        const r = button('Use notes only').getBoundingClientRect();
        assert(r.top >= 0 && r.bottom <= innerHeight + 1, 'Notes-only onboarding action unreachable');
        await snapshot({ id: null, cases: [] }, `onboarding-welcome-${width}x${height}.png`, { fixture: 'fresh isolated profile', capture_started: false });
      }
      click('Use notes only'); return;
    }
    await nativeInvoke('plugin:window|set_position', { label: 'main', value: { Logical: { x: 20, y: 40 } } });
    state = JSON.parse(sessionStorage.getItem('responsive-smoke') || 'null');
    if (state?.phase === 'finished') return;
    if (!state) {
      const title = `Responsive fixture — café 日本語 👩🏽‍💻 ${'long-title'.repeat(25)}`;
      const meeting = await nativeInvoke('api_create_meeting_draft', { title, timezone: 'Asia/Manila' });
      const paragraph = (id, text) => ({ id, type: 'paragraph', content: [{ type: 'text', text, styles: {} }], children: [] });
      const blocks = [paragraph('intro', 'Disposable original notes — Unicode 日本語 café 👩🏽‍💻'), paragraph('url', `https://example.invalid/${'unbrokenslug'.repeat(120)}`),
        { id: 'wide-table', type: 'table', content: { type: 'tableContent', columnWidths: Array(12).fill(160), rows: [0, 1, 2].map(row => ({ cells: Array.from({ length: 12 }, (_, col) => [{ type: 'text', text: `Row ${row}, column ${col} — 日本語`, styles: {} }]) })) }, children: [] },
        ...Array.from({ length: 60 }, (_, i) => paragraph(`note-${i}`, `Original note ${i} — deliberately long enough to scroll. ${'Long paragraph with Unicode café 日本語. '.repeat(6)}`))];
      await nativeInvoke('api_save_meeting_notes', { input: { meeting_id: meeting.id, notes_json: JSON.stringify(blocks), notes_markdown: 'Synthetic original notes', notes_plain_text: 'Synthetic original notes', base_revision: 0, commit_id: crypto.randomUUID(), title, links: [] } });
      await nativeInvoke('api_save_transcript', { meetingTitle: title, meetingId: meeting.id, recordingSessionId: crypto.randomUUID(), folderPath: null,
        transcripts: Array.from({ length: 70 }, (_, i) => ({ id: `responsive-transcript-${i}`, text: `Disposable passage ${i} — 日本語 ${'transcript words '.repeat(35)} https://example.invalid/${'segment'.repeat(100)}`, timestamp: '12:00:00', audio_start_time: i, audio_end_time: i + 1, duration: 1, sequence_id: i, audio_source: ['microphone', 'system', 'mixed', 'unknown'][i % 4] })) });
      await report({ phase: 'seed-enhancements', meeting_id: meeting.id, blocks });
      await wait(async () => (await nativeInvoke('api_get_meeting_enhancements', { meetingId: meeting.id })).versions.length >= 2, 'disposable enhanced fixtures');
      const realisticTitle = 'Product planning · A calmer meeting workspace';
      const realistic = await nativeInvoke('api_create_meeting_draft', { title: realisticTitle, timezone: 'Asia/Manila' });
      const heading = (id, text, level = 2) => ({ id, type: 'heading', props: { level }, content: [{ type: 'text', text, styles: {} }], children: [] });
      const bullet = (id, text) => ({ ...paragraph(id, text), type: 'bulletListItem' });
      const realisticBlocks = [
        paragraph('realistic-intro', 'Thursday, 15 October · Design & product · Synthetic demo meeting'),
        heading('realistic-purpose', 'A quieter place for good conversations'),
        paragraph('realistic-context', 'We reviewed how Meetily can help people stay present in a meeting, keep their own notes, and find the important details afterward.'),
        heading('realistic-decisions', 'What we agreed'),
        bullet('realistic-decision-1', 'Keep original notes at the center of the workspace.'),
        bullet('realistic-decision-2', 'Use warm paper, soft sage accents, and glass for navigation.'),
        bullet('realistic-decision-3', 'Keep recording controls clear and easy to reach.'),
        heading('realistic-next', 'Next steps'),
        { id: 'realistic-task-1', type: 'checkListItem', props: { checked: true }, content: [{ type: 'text', text: 'Alex — prepare the first design review', styles: {} }], children: [] },
        { id: 'realistic-task-2', type: 'checkListItem', props: { checked: false }, content: [{ type: 'text', text: 'Sam — check the small-window experience', styles: {} }], children: [] },
        paragraph('realistic-question', 'Open question: how can enhanced notes feel helpful while keeping the original wording close at hand?'),
      ];
      await nativeInvoke('api_save_meeting_notes', { input: { meeting_id: realistic.id, notes_json: JSON.stringify(realisticBlocks), notes_markdown: 'Synthetic product planning notes', notes_plain_text: 'Synthetic product planning notes. Original notes, warm paper, sage accents, and clear recording controls.', base_revision: 0, commit_id: crypto.randomUUID(), title: realisticTitle, links: [] } });
      await nativeInvoke('api_update_meeting_metadata', { meetingId: realistic.id, participants: ['Alex Chen', 'Sam Rivera', 'Jordan Lee'], tags: ['Design review', 'Product'] });
      const passages = [
        ['microphone', 'Let’s make the notes the most comfortable part of the workspace. People should be able to write before or during a meeting.'],
        ['system', 'Agreed. The warm paper surface feels calm, and sage gives us a clear accent without making every control compete.'],
        ['microphone', 'Glass belongs in the navigation and recording controls. Long notes and transcripts still need a solid reading surface.'],
        ['system', 'I’ll review the narrow-window layout. The recording status needs to stay understandable when the window is small.'],
        ['mixed', 'We’ll bring the first version to a design review on Friday and keep the original notes easy to find.'],
        ['unknown', 'These passages and participant names are synthetic fixtures for visual review. No audio was captured.'],
      ];
      await nativeInvoke('api_save_transcript', { meetingTitle: realisticTitle, meetingId: realistic.id, recordingSessionId: crypto.randomUUID(), folderPath: null,
        transcripts: passages.map(([audio_source, text], i) => ({ id: `design-transcript-${i}`, text, timestamp: `09:${String(i).padStart(2, '0')}:00`, audio_start_time: i * 35, audio_end_time: i * 35 + 28, duration: 28, sequence_id: i, audio_source })) });
      await report({ phase: 'seed-enhancements', meeting_id: realistic.id, blocks: realisticBlocks, version_prefix: 'design', synthetic: true });
      await wait(async () => (await nativeInvoke('api_get_meeting_enhancements', { meetingId: realistic.id })).versions.length >= 2, 'realistic enhanced fixture');
      state = { id: meeting.id, realisticId: realistic.id, realisticTitle, phase: 'workspace', cases: [], designCases: [] };
      sessionStorage.setItem('responsive-smoke', JSON.stringify(state));
      sessionStorage.setItem('workspace_meeting_id', state.id);
      localStorage.setItem('meetily.lastNotesMeeting', state.id);
      location.href = `/meeting-details?id=${encodeURIComponent(state.id)}`; return;
    }
    if (state.phase === 'library') {
      await wait(() => document.querySelector('article'), 'library results');
      for (const [width, height] of sizes) for (const expanded of [false, true]) {
        await resize(width, height); await setSidebar(expanded); layoutCheck();
        state.cases.push({ route: 'library', width, height, expanded });
        document.querySelector('article [aria-label="Export meeting"]').click();
        await wait(() => document.querySelector('select[aria-label="Export format"]'), 'library export dialog');
        await sleep(250); layoutCheck();
        state.cases.push({ route: 'library-export', width, height, expanded });
        if (!expanded && (width === 480 || (width === 1100 && height === 400))) await snapshot(state, `library-export-${width}x${height}.png`);
        click('Close'); await sleep(250);
        if (!expanded && (width === 1440 || width === 480)) await snapshot(state, `library-${width}x${height}.png`);
      }
      state.phase = 'design-workspace'; sessionStorage.setItem('responsive-smoke', JSON.stringify(state));
      location.href = `/meeting-details?id=${encodeURIComponent(state.realisticId)}`; return;
    }
    if (state.phase === 'design-workspace') {
      await wait(() => document.querySelector('#original-notes [contenteditable="true"]'), 'realistic original notes');
      await wait(() => document.querySelector('.recording-dock'), 'recording dock');
      await setSidebar(true); await resize(1440, 900); await setDrawer(false);
      const editorStyle = getComputedStyle(document.querySelector('#original-notes .bn-editor'));
      const paperStyle = getComputedStyle(document.querySelector('#original-notes .notes-panel'));
      assert(editorStyle.backgroundColor === paperStyle.backgroundColor, `BlockNote paper mismatch: ${editorStyle.backgroundColor} vs ${paperStyle.backgroundColor}`);
      assert(/Source[_ ]Sans/i.test(editorStyle.fontFamily), `BlockNote font mismatch: ${editorStyle.fontFamily}`);
      state.designCases.push({ route: 'editor-materials', paper_background_matches: true, font: editorStyle.fontFamily, background: editorStyle.backgroundColor });
      await snapshot(state, 'design-notes-1440x900.png', { fixture: 'realistic synthetic meeting', capture_started: false });
      await setDrawer(true); click('Transcript'); await sleep(200);
      const channelIcons = [...document.querySelectorAll('.workspace-transcript [role="img"][aria-label^="Audio source:"]')].filter(visible);
      assert(channelIcons.length > 0, 'Audio capture-channel icons missing');
      for (const icon of channelIcons) {
        assert(icon.textContent.trim() === '' && icon.querySelector('svg'), 'Capture channel badge is not icon-only');
        assert(icon.getAttribute('title') === icon.getAttribute('aria-label').replace('Audio source:', 'Capture channel:'), 'Capture channel hover title/accessibility label missing');
        assert(icon.getBoundingClientRect().width <= 25, 'Capture channel icon exceeds compact rail');
      }
      state.designCases.find(value => value.route === 'editor-materials').icon_only_audio_channels = true;
      await snapshot(state, 'design-transcript-1440x900.png', { fixture: 'synthetic transcript with source badges', capture_started: false });
      await setDrawer(false); click('Enhanced notes'); await sleep(300);
      await snapshot(state, 'design-enhanced-1440x900.png', { fixture: 'approved and candidate synthetic enhancement', human_approved: false });
      click('My notes'); await resize(480, 400); await setSidebar(false);
      const menuButton = document.querySelector('[aria-label="More note actions"]');
      menuButton.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse', ctrlKey: false }));
      await wait(() => document.querySelector('[role="menu"]'), 'note action menu');
      const menu = document.querySelector('[role="menu"]');
      assert(menu.textContent.includes('Markdown') && menu.textContent.includes('JSON'), 'Note export actions missing');
      const mr = menu.getBoundingClientRect();
      assert(mr.left >= 0 && mr.right <= innerWidth + 1 && mr.top >= 0 && mr.bottom <= innerHeight + 1, 'Note action menu outside narrow viewport');
      await snapshot(state, 'design-note-actions-480x400.png');
      menu.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
      await wait(() => !document.querySelector('[role="menu"]'), 'close note action menu');
      state.designCases.push({ route: 'note-actions', width: 480, height: 400, actions_reachable: true });

      // Debug WKWebView fixture only: use the existing context action to render
      // statuses with no native capture. React does not expose this context as a
      // production global, and no hook or event is added to application code.
      const recordingContext = () => {
        const node = document.querySelector('.recording-dock');
        const key = Object.keys(node).find(value => value.startsWith('__reactFiber$'));
        assert(key, 'React fixture context unavailable');
        for (let fiber = node[key]; fiber; fiber = fiber.return) {
          for (let dependency = fiber.dependencies?.firstContext; dependency; dependency = dependency.next) {
            if (typeof dependency.memoizedValue?.setStatus === 'function' && typeof dependency.memoizedValue?.confirmRecordingStarted === 'function') return dependency.memoizedValue;
          }
        }
        throw new Error('Recording context fixture lookup failed');
      };
      const fixtureSession = crypto.randomUUID();
      let fixtureNative = { is_recording: false, is_paused: false, is_active: false, recording_duration: null, active_duration: null, phase: 'idle' };
      const lifecycleCalls = [];
      const originalFetch = window.fetch.bind(window);
      const handler = window.webkit.messageHandlers.ipc;
      const originalPost = handler.postMessage.bind(handler);
      const isLifecycle = command => /^(start_recording|stop_recording|pause_recording|resume_recording|cancel_recording_start)/.test(command);
      window.fetch = async (input, options) => {
        const url = String(input instanceof Request ? input.url : input);
        if (url.startsWith('ipc://')) {
          const command = decodeURIComponent(new URL(url).pathname.slice(1));
          if (isLifecycle(command)) { lifecycleCalls.push(command); throw new Error(`Capture forbidden in visual fixture: ${command}`); }
          if (command === 'get_recording_state') return new Response(JSON.stringify(fixtureNative), { status: 200, headers: { 'Content-Type': 'application/json', 'Tauri-Response': 'ok' } });
        }
        return originalFetch(input, options);
      };
      handler.postMessage = payload => {
        const message = typeof payload === 'string' ? JSON.parse(payload) : payload;
        if (isLifecycle(message.cmd)) { lifecycleCalls.push(message.cmd); window.__TAURI_INTERNALS__.runCallback(message.error, 'Capture forbidden in visual fixture'); return; }
        if (message.cmd === 'get_recording_state') { window.__TAURI_INTERNALS__.runCallback(message.callback, fixtureNative); return; }
        return originalPost(payload);
      };
      const emit = (event, payload = {}) => nativeInvoke('plugin:event|emit', { event, payload });
      const statusFixture = async (name, expected) => {
        await wait(() => document.querySelector('.recording-dock [role="status"]')?.textContent.includes(expected), `${name} recording fixture`);
        for (const [width, height] of [[1440, 900], [480, 400]]) {
          await resize(width, height);
          if (name === 'error') {
            // Error explanations legitimately make the bounded recording footer
            // scroll. Exercise every action through its real local scroller.
            assert(document.documentElement.scrollWidth <= innerWidth + 1, 'Recording error fixture overflows horizontally');
            const footer = document.querySelector('.workspace-recording');
            assert(footer.scrollWidth <= footer.clientWidth + 1, 'Recording error footer widens');
            for (const action of footer.querySelectorAll('button')) {
              action.scrollIntoView({ block: 'nearest', inline: 'nearest' });
              const r = action.getBoundingClientRect();
              assert(r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1, 'Recording error action unreachable');
            }
          } else { layoutCheck(); }
          await snapshot(state, `design-recording-${name}-${width}x${height}.png`, { fixture: 'recording visual state only', fixture_method: 'existing React recording context + native event + mocked status transport', capture_started: false, physical_recording_tested: false });
          state.designCases.push({ route: 'recording-fixture', status: name, width, height, capture_started: false });
        }
      };
      try {
        await statusFixture('idle', 'Ready to record');
        fixtureNative = { ...fixtureNative, is_recording: true, is_active: true, phase: 'recording', meeting_id: state.realisticId, recording_session_id: fixtureSession, recording_duration: 83, active_duration: 83 };
        await emit('recording-started', { meeting_id: state.realisticId, recording_session_id: fixtureSession });
        await statusFixture('recording', 'Recording');
        fixtureNative = { ...fixtureNative, is_paused: true, is_active: false };
        await emit('recording-paused'); await statusFixture('paused', 'Paused');
        fixtureNative = { ...fixtureNative, is_recording: false, is_paused: false, is_active: false, phase: 'idle' };
        await emit('recording-stopped', { meeting_id: state.realisticId, recording_session_id: fixtureSession, message: 'Synthetic visual fixture stopped; no capture started.' });
        await statusFixture('stopping', 'Stopping');
        recordingContext().setStatus('processing'); await statusFixture('processing', 'Finishing');
        recordingContext().setStatus('saving'); await statusFixture('saving', 'Saving');
        recordingContext().setStatus('error', 'Synthetic visual fixture: microphone unavailable. No recording was started.');
        await statusFixture('error', 'Check recording status');
        assert(document.body.innerText.includes('Recording could not start'), 'Recording error explanation missing');
        recordingContext().setStatus('idle');
        sessionStorage.removeItem('recording_meeting_id'); sessionStorage.removeItem('recording_session_id');
        window.dispatchEvent(new Event('recording-state-refresh')); await sleep(700);
        assert(lifecycleCalls.length === 0, 'Visual fixtures invoked native recording lifecycle');
      } finally { window.fetch = originalFetch; handler.postMessage = originalPost; }
      state.recordingVisualFixtures = { states: ['idle', 'recording', 'paused', 'stopping', 'processing', 'saving', 'error'], native_lifecycle_calls: lifecycleCalls, capture_started: false, physical_recording_tested: false, fixture_method: 'debug-only existing React context lookup, native events, mocked get_recording_state transport' };

      // Activate the shipped media rules in this disposable document. This
      // verifies the real CSS fallback without changing macOS preferences.
      const mediaRules = [];
      const scan = rules => { for (const rule of rules) { if (rule.type === CSSRule.MEDIA_RULE && /prefers-reduced-(motion|transparency)/.test(rule.conditionText)) mediaRules.push(rule); if (rule.cssRules) scan(rule.cssRules); } };
      for (const sheet of document.styleSheets) { try { scan(sheet.cssRules); } catch {} }
      assert(mediaRules.length >= 2, 'Reduced motion/transparency rules missing');
      const originals = mediaRules.map(rule => rule.media.mediaText);
      try {
        mediaRules.forEach(rule => { rule.media.mediaText = 'all'; });
        await sleep(100); await resize(1100, 700); await setSidebar(true);
        const glass = document.querySelector('.recording-dock');
        const style = getComputedStyle(glass);
        assert((style.backdropFilter || style.webkitBackdropFilter) === 'none', 'Opaque fallback still blurs');
        assert(!style.backgroundColor.startsWith('rgba('), 'Opaque fallback remains translucent');
        assert(parseFloat(style.transitionDuration) <= 0.00001, 'Reduced-motion transition is active');
        await snapshot(state, 'design-reduced-motion-opaque-1100x700.png', { fixture: 'shipped CSS media rules forced active in isolated document', native_media_preferences_changed: false });
        state.designCases.push({ route: 'accessibility-materials', opaque: true, blur_disabled: true, reduced_motion: true, method: 'shipped media rules activated via CSSOM', os_preference_tested: false });
      } finally { mediaRules.forEach((rule, i) => { rule.media.mediaText = originals[i]; }); }
      state.phase = 'design-library'; sessionStorage.setItem('responsive-smoke', JSON.stringify(state));
      location.href = '/library'; return;
    }
    if (state.phase === 'design-library') {
      await wait(() => document.querySelector('article'), 'realistic library');
      const search = document.querySelector('#library-search');
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(search, 'calmer');
      search.dispatchEvent(new Event('input', { bubbles: true }));
      await wait(() => document.querySelectorAll('article').length === 1 && document.querySelector('article').textContent.includes(state.realisticTitle), 'filtered realistic meeting');
      for (const [width, height] of [[1440, 900], [480, 400]]) {
        await resize(width, height); await setSidebar(width > 700); layoutCheck();
        await snapshot(state, `design-library-${width}x${height}.png`, { fixture: 'realistic synthetic meeting' });
        click('Metadata'); await wait(() => document.querySelector('article form'), 'meeting metadata form');
        const form = document.querySelector('article form');
        assert(form.querySelectorAll('textarea').length === 2, 'Manual metadata fields missing');
        for (const node of form.querySelectorAll('textarea, button')) {
          node.scrollIntoView({ block: 'nearest', inline: 'nearest' });
          const r = node.getBoundingClientRect();
          assert(r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1, 'Metadata control unreachable');
        }
        await snapshot(state, `design-library-metadata-${width}x${height}.png`);
        click('Cancel'); await wait(() => !document.querySelector('article form'), 'metadata cancel');
        const deletion = document.querySelector('article button[aria-label^="Delete "]');
        deletion.scrollIntoView({ block: 'nearest' }); deletion.click();
        await wait(() => document.querySelector('[role="alertdialog"]'), 'delete confirmation');
        const dialog = document.querySelector('[role="alertdialog"]');
        assert(dialog.scrollWidth <= dialog.clientWidth + 1, 'Delete dialog horizontal overflow');
        button('Cancel').scrollIntoView({ block: 'nearest' });
        const r = button('Cancel').getBoundingClientRect();
        assert(r.top >= 0 && r.bottom <= innerHeight + 1, 'Delete cancellation unreachable');
        await snapshot(state, `design-library-delete-cancel-${width}x${height}.png`);
        click('Cancel'); await wait(() => !document.querySelector('[role="alertdialog"]'), 'delete cancel');
        await nativeInvoke('api_get_meeting_metadata', { meetingId: state.realisticId });
        state.designCases.push({ route: 'library-metadata-delete', width, height, metadata_cancelled: true, deletion_cancelled: true, meeting_preserved: true });
      }
      state.phase = 'design-settings'; sessionStorage.setItem('responsive-smoke', JSON.stringify(state));
      location.href = '/settings'; return;
    }
    if (state.phase === 'design-settings') {
      await wait(() => document.querySelector('[aria-label="Settings categories"]'), 'settings tabs');
      for (const [width, height] of [[1440, 900], [480, 400], [1100, 400]]) {
        await resize(width, height); await setSidebar(false);
        for (const label of ['General', 'Recordings', 'Transcription', 'Summary', 'Beta']) {
          const tab = button(label); tab.scrollIntoView({ block: 'nearest', inline: 'nearest' }); click(label); await sleep(350);
          assert(tab.getAttribute('aria-selected') === 'true', `Settings tab failed: ${label}`);
          const r = tab.getBoundingClientRect();
          assert(r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1, `Settings tab unreachable: ${label}`);
          assert(document.documentElement.scrollWidth <= innerWidth + 1, `Settings overflow: ${label}`);
          const panel = document.querySelector('[role="tabpanel"][data-state="active"]');
          assert(panel && panel.scrollWidth <= panel.clientWidth + 1, `Settings content overflow: ${label}`);
          if (width === 1440 || (width === 480 && ['General', 'Transcription', 'Beta'].includes(label))) await snapshot(state, `design-settings-${label.toLowerCase()}-${width}x${height}.png`);
          state.designCases.push({ route: 'settings', tab: label, width, height, category_reachable: true, no_horizontal_overflow: true });
        }
      }
      if (interaction) {
        checkRuntime(); state.phase = 'interaction-workspace'; persistInteraction();
        location.href = `/meeting-details?id=${encodeURIComponent(state.realisticId)}`; return;
      }
      state.phase = 'finished'; sessionStorage.setItem('responsive-smoke', JSON.stringify(state));
      await report({ ok: true, phase: 'finished', native_webview: true, isolated: true, animations_enabled: true, native_wkwebview_snapshots: true, cases: state.cases, design_cases: state.designCases, continuity: state.continuity, recording_visual_fixtures: state.recordingVisualFixtures, meeting_id: state.id, realistic_meeting_id: state.realisticId, synthetic_content: true, human_visual_acceptance: 'pending', windows_tested: false });
      return;
    }
    if (state.phase === 'interaction-workspace') {
      const original = await wait(() => document.querySelector('#original-notes [contenteditable="true"]'), 'stress original editor');
      const enhanced = await wait(() => document.querySelector('#enhanced-notes [contenteditable="true"]'), 'stress enhanced editor');
      await resize(1100, 700); await setSidebar(false); await setDrawer(false);
      // Real click events and native sizes interleave while chrome transitions
      // are active. Content and editor instances must survive every operation.
      for (let i = 0; i < 100; i++) {
        document.querySelector('.sidebar-toggle').click();
        document.querySelector('[aria-controls="meeting-transcript-panel"]').click();
        if (i % 5 === 0) await resize(i % 10 ? 480 : 1100, i % 10 ? 400 : 700, 20);
        if (i % 10 === 0) await sleep(20);
      }
      await sleep(400); await setSidebar(false); await setDrawer(false); await resize(480, 400); layoutCheck();
      assert(document.querySelector('#original-notes [contenteditable="true"]') === original && document.querySelector('#enhanced-notes [contenteditable="true"]') === enhanced, 'Toggle storm remounted editors');
      await interactionCase({ scenario: 'toggle-resize-storm', sidebar_toggles: 100, drawer_toggles: 100, native_resizes: 20, editor_identity: true });
      for (let i = 0; i < 30; i++) { for (let burst = 0; burst < 8; burst++) { button(i % 2 ? 'My notes' : 'Enhanced notes').click(); } await sleep(20); }
      await sleep(400); click('My notes'); await wait(() => visible(original), 'recover after rapid document tabs');
      assert(document.querySelector('#original-notes [contenteditable="true"]') === original && document.querySelector('#enhanced-notes [contenteditable="true"]') === enhanced, 'Rapid document tabs remounted editors');
      await interactionCase({ scenario: 'rapid-document-tabs', attempted_clicks: 240, original_restored: true, editor_identity: true });

      const delayedSave = gate(); let saveCalls = 0;
      const restoreSave = intercept({ api_save_meeting_notes: async () => { saveCalls++; await delayedSave.promise; } });
      try {
        await nativeInvoke('plugin:window|set_focus', { label: 'main' });
        original.focus(); const range = document.createRange(); range.selectNodeContents(original.querySelector('.bn-inline-content') || original); range.collapse(false);
        const selection = getSelection(); selection.removeAllRanges(); selection.addRange(range);
        const marker = ' Interaction delayed-save fixture';
        await report({ phase: 'native-input', target: 'stress-original', marker });
        await wait(() => original.textContent.includes(marker), 'stress original keyboard edit');
        for (let i = 0; i < 50; i++) button('Enhanced notes').click();
        await wait(() => saveCalls === 1, 'one held note-save request'); await sleep(150);
        assert(saveCalls === 1 && visible(original), 'Repeated tab switch duplicated save or hid unsaved original');
        delayedSave.reject(new Error('Expected fixture save failure'));
        await wait(() => document.body.innerText.includes('Expected fixture save failure'), 'save failure retained by shipped handler');
        assert(visible(original) && original.textContent.includes(marker), 'Failed tab switch discarded original edit');
      } finally { delayedSave.release(); restoreSave(); }
      click('Save'); await wait(async () => (await nativeInvoke('api_get_meeting_notes', { meetingId: state.realisticId })).notes_json.includes('Interaction delayed-save fixture'), 'save retry native persistence');
      click('Enhanced notes'); await wait(() => visible(enhanced), 'switch after successful retry'); click('My notes'); await wait(() => visible(original), 'return original after retry');
      await interactionCase({ scenario: 'delayed-save-tab-switch', attempted_clicks: 50, native_save_requests_while_held: saveCalls, injected_failure_caught: true, edit_retained: true, retry_persisted: true });

      const options = await nativeInvoke('api_get_meeting_export_options', { meetingId: state.realisticId });
      const firstOptions = gate(); let optionCalls = 0; let exportCalls = 0; const exportGate = gate();
      const restoreExport = intercept({
        api_get_meeting_export_options: async () => { optionCalls++; if (optionCalls === 1) { await firstOptions.promise; return { ...options, title: 'STALE OPTIONS MUST NOT WIN' }; } return options; },
        api_export_meeting: async () => { exportCalls++; await exportGate.promise; return { status: 'cancelled', path: null }; },
      });
      try {
        document.querySelector('[aria-label="Export meeting"]').click(); await wait(() => optionCalls === 1, 'held first export options');
        click('Close'); await sleep(180); document.querySelector('[aria-label="Export meeting"]').click();
        await wait(() => document.querySelector('select[aria-label="Export format"]'), 'new export options');
        firstOptions.release(); await sleep(200);
        assert(!document.body.innerText.includes('STALE OPTIONS MUST NOT WIN'), 'Late export options replaced current dialog');
        const save = button('Save export…'); for (let i = 0; i < 75; i++) save.click();
        await wait(() => exportCalls === 1, 'one mocked export request'); await sleep(100);
        assert(exportCalls === 1, 'Rapid export save duplicated requests');
        escape(); await sleep(100); assert(visible(document.querySelector('[role="dialog"]')), 'Busy export closed on Escape');
        exportGate.release(); await wait(() => document.body.innerText.includes('Export cancelled. No file was saved.'), 'mocked export result');
        escape(); await wait(() => !document.querySelector('[role="dialog"][data-state="open"]'), 'idle export Escape');
        document.querySelector('[aria-label="Export meeting"]').click(); await wait(() => document.querySelector('select[aria-label="Export format"]'), 'export reopen');
        const overlay = document.querySelector('[data-state="open"][class*="fixed inset-0"]');
        assert(overlay, 'Dialog overlay unavailable');
        overlay.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse', clientX: 1, clientY: 1 }));
        overlay.dispatchEvent(new MouseEvent('click', { bubbles: true, button: 0, clientX: 1, clientY: 1 }));
        await wait(() => !document.querySelector('[role="dialog"][data-state="open"]'), 'outside dismiss');
      } finally { firstOptions.release(); exportGate.release(); restoreExport(); }
      await interactionCase({ scenario: 'export-option-race-save-spam', attempted_saves: 75, mocked_export_requests: exportCalls, stale_options_ignored: true, busy_escape_blocked: true, idle_escape_and_outside_dismiss: true, native_file_dialog: false, export_file_written: false });
      await snapshot(state, 'interaction-workspace-recovered-480x400.png', { fixture: 'adversarial interaction recovery' });

      const draftGate = gate(); let draftCalls = 0;
      const restoreDraft = intercept({ api_create_meeting_draft: async () => { draftCalls++; await draftGate.promise; } });
      try {
        const newMeeting = button('New meeting'); for (let i = 0; i < 100; i++) newMeeting.click();
        await wait(() => draftCalls === 1, 'single held new draft request'); await sleep(150); assert(draftCalls === 1, 'New-meeting spam created duplicate drafts');
        draftGate.release(); await wait(() => sessionStorage.getItem('workspace_meeting_id') !== state.realisticId, 'new draft recovery');
      } finally { draftGate.release(); restoreDraft(); }
      await interactionCase({ scenario: 'new-draft-spam', attempted_clicks: 100, native_draft_requests: draftCalls, disposable_profile_only: true });
      checkRuntime(); state.phase = 'interaction-library'; persistInteraction(); location.href = '/library'; return;
    }
    if (state.phase === 'interaction-library') {
      await wait(() => document.querySelector('article'), 'stress library'); await resize(480, 400); await setSidebar(false);
      const search = document.querySelector('#library-search'); Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(search, 'calmer'); search.dispatchEvent(new Event('input', { bubbles: true }));
      await wait(() => document.querySelectorAll('article').length === 1 && document.querySelector('article').textContent.includes(state.realisticTitle), 'stress fixture search');
      click('Metadata'); await wait(() => document.querySelector('article form'), 'metadata editor');
      const form = document.querySelector('article form'); const fields = [...form.querySelectorAll('textarea')];
      Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set.call(fields[0], 'Alex Chen\nSam Rivera\nStress fixture'); fields[0].dispatchEvent(new Event('input', { bubbles: true }));
      const metadataGate = gate(); let metadataCalls = 0;
      const restoreMetadata = intercept({ api_update_meeting_metadata: async () => { metadataCalls++; await metadataGate.promise; } });
      try {
        for (let i = 0; i < 100; i++) form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
        await wait(() => metadataCalls > 0, 'held metadata update'); await sleep(150); assert(metadataCalls === 1, `Repeated metadata submit duplicated requests: ${metadataCalls}`);
        assert(button('Cancel').disabled && fields.every(field => field.disabled), 'Metadata controls accept changes during held save');
        button('Cancel').click(); assert(document.querySelector('article form') === form, 'Busy metadata save cancelled');
        metadataGate.release(); await wait(() => !document.querySelector('article form'), 'metadata save settled');
      } finally { metadataGate.release(); restoreMetadata(); }
      click('Metadata'); await wait(() => document.querySelector('article form'), 'metadata reopen after save');
      const savedText = document.querySelector('article form textarea').value; assert(savedText.includes('Stress fixture'), 'Metadata successful save not retained');
      const unsavedField = document.querySelector('article form textarea'); Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set.call(unsavedField, `${savedText}\nCANCELLED EDIT MUST NOT SAVE`); unsavedField.dispatchEvent(new Event('input', { bubbles: true }));
      const cancel = button('Cancel'); cancel.scrollIntoView({ block: 'nearest' }); cancel.click(); await wait(() => !document.querySelector('article form'), 'metadata cancellation after save');
      click('Metadata'); await wait(() => document.querySelector('article form'), 'metadata reopen after cancel');
      assert(document.querySelector('article form textarea').value === savedText, 'Metadata cancellation persisted unsaved edit');
      click('Cancel'); await wait(() => !document.querySelector('article form'), 'final metadata cancel');
      await interactionCase({ scenario: 'metadata-submit-storm', submit_events: 100, native_requests: metadataCalls, controls_locked: true, busy_cancel_blocked: true, save_persisted: true, cancellation_recovered: true });
      document.querySelector('article button[aria-label^="Delete "]').click(); await wait(() => document.querySelector('[role="alertdialog"]'), 'narrow delete dialog');
      const dialog = document.querySelector('[role="alertdialog"]'); for (const action of dialog.querySelectorAll('button')) { action.scrollIntoView({ block: 'nearest' }); const r = action.getBoundingClientRect(); assert(r.left >= 0 && r.right <= innerWidth + 1 && r.top >= 0 && r.bottom <= innerHeight + 1, 'Narrow modal action unreachable'); }
      escape(); await wait(() => !document.querySelector('[role="alertdialog"]'), 'delete Escape');
      await nativeInvoke('api_get_meeting_metadata', { meetingId: state.realisticId });
      await interactionCase({ scenario: 'narrow-delete-dismiss', controls_reachable: true, escape_dismissed: true, meeting_preserved: true });
      checkRuntime(); state.phase = 'interaction-settings'; persistInteraction(); location.href = '/settings'; return;
    }
    if (state.phase === 'interaction-settings') {
      await wait(() => document.querySelector('[aria-label="Settings categories"]'), 'stress settings');
      const labels = ['General', 'Recordings', 'Transcription', 'Summary', 'Beta'];
      for (let i = 0; i < 100; i++) {
        const label = labels[i % labels.length]; click(label);
        if (i % 10 === 0) await resize(i % 20 ? 480 : 1100, i % 20 ? 400 : 700, 20);
        if (i % 5 === 0) await sleep(20);
      }
      await resize(480, 400); await sleep(200);
      const tabs = [...document.querySelectorAll('[aria-label="Settings categories"] [role="tab"]')];
      tabs[0].scrollIntoView({ block: 'nearest', inline: 'nearest' }); tabs[0].focus();
      await report({ phase: 'native-key', target: 'settings-arrow-right', key: 'right' });
      await wait(() => document.activeElement === tabs[1], 'native settings arrow focus');
      assert(tabs[1].getAttribute('aria-selected') === 'true', 'Settings keyboard activation failed');
      const r = tabs[1].getBoundingClientRect(); assert(r.left >= 0 && r.right <= innerWidth + 1, 'Keyboard-focused settings tab not scrolled into view');
      assert(document.documentElement.scrollWidth <= innerWidth + 1, 'Settings storm overflow');
      await interactionCase({ scenario: 'settings-click-keyboard-resize', repeated_tab_clicks: 100, native_arrow_keyboard: true, focus_reachable: true, horizontal_overflow: false });
      await snapshot(state, 'interaction-settings-recovered-480x400.png', { fixture: 'settings storm then native keyboard focus' });
      checkRuntime(); state.phase = 'finished'; persistInteraction();
      await report({ ok: true, phase: 'finished', native_webview: true, isolated: true, cases: state.cases, design_cases: state.designCases, interaction_cases: state.interactionCases, runtime_errors: runtimeErrors, expected_fixture_failures: ['Expected fixture save failure handled by note-save and document-tab UI'], continuity: state.continuity, recording_visual_fixtures: state.recordingVisualFixtures, native_recording_lifecycle_calls: 0, export_file_dialogs: 0, export_files_written: 0, synthetic_content: true, windows_tested: false }); return;
    }
    const original = await wait(() => document.querySelector('#original-notes [contenteditable="true"]'), 'original editor');
    await wait(() => document.querySelector('#enhanced-notes [contenteditable="true"]'), 'enhanced editor');
    await nativeInvoke('plugin:window|set_focus', { label: 'main' });
    await sleep(350);
    const enhanced = document.querySelector('#enhanced-notes [contenteditable="true"]');
    let held;
    let holdSaves = false;
    const hold = () => { holdSaves = true; held = new Promise(resolve => { releaseSaves = () => { holdSaves = false; resolve(); }; }); };
    const calls = [];
    // Tauri deliberately makes invoke immutable. Intercept its native IPC
    // fetch transport instead, and prove that the barrier is active below.
    const nativeFetch = window.fetch.bind(window);
    window.fetch = async (input, options) => {
      const url = String(input instanceof Request ? input.url : input);
      if (url.startsWith('ipc://')) {
        const command = decodeURIComponent(new URL(url).pathname.slice(1));
        calls.push(command);
        if (holdSaves && (command === 'api_save_meeting_notes' || command === 'api_save_enhancement_version')) await held;
      }
      return nativeFetch(input, options);
    };
    // WKWebView may have fallen back from custom-protocol fetch during
    // onboarding. Its native message handler is the fallback transport.
    const ipcHandler = window.webkit.messageHandlers.ipc;
    const nativePost = ipcHandler.postMessage.bind(ipcHandler);
    ipcHandler.postMessage = async payload => {
      const message = typeof payload === 'string' ? JSON.parse(payload) : payload;
      calls.push(message.cmd);
      if (holdSaves && (message.cmd === 'api_save_meeting_notes' || message.cmd === 'api_save_enhancement_version')) await held;
      nativePost(payload);
    };
    await nativeInvoke('api_get_meeting_metadata', { meetingId: state.id });
    assert(calls.includes('api_get_meeting_metadata'), 'Native IPC barrier was not installed');
    const edit = async (node, marker, target) => {
      node.focus(); const range = document.createRange(); range.selectNodeContents(node.querySelector('.bn-inline-content') || node); range.collapse(false);
      const selection = getSelection(); selection.removeAllRanges(); selection.addRange(range);
      // The companion driver types through macOS, exercising the real editor
      // input pipeline rather than a DOM-only execCommand mutation.
      await report({ phase: 'native-input', target, marker });
      await wait(() => node.textContent.includes(marker), `${target} native keyboard input`);
      await wait(() => calls.includes(target === 'original' ? 'api_save_meeting_notes' : 'api_save_enhancement_version'), `${target} pending save`);
    };
    let originalText = original.textContent;
    let enhancedText = enhanced.textContent;
    const identity = () => {
      assert(document.querySelector('#original-notes [contenteditable="true"]') === original, 'Original editor remounted');
      assert(document.querySelector('#enhanced-notes [contenteditable="true"]') === enhanced, 'Enhanced editor remounted');
      assert(original.textContent === originalText && enhanced.textContent === enhancedText, 'Pending editor content changed');
      const versionSelect = [...document.querySelectorAll('section[aria-label="Enhanced notes"] select')].at(-1);
      assert(versionSelect.value === 'responsive-approved', 'Selected enhanced document changed');
    };
    const matrix = async tab => {
      for (const [width, height] of sizes) for (const expanded of [false, true]) for (const drawer of [false, true]) {
        await resize(width, height); await setSidebar(expanded); await setDrawer(drawer); await sleep(100);
        if (drawer) {
          click('Summary'); await sleep(100); assert(document.querySelector('.details-summary').dataset.state === 'active', 'Summary tab did not activate'); layoutCheck();
          click('Transcript'); await sleep(100); assert(document.querySelector('.details-transcript').dataset.state === 'active', 'Transcript tab did not activate');
        }
        layoutCheck(); identity();
        state.cases.push({ route: 'workspace', width, height, viewport: { width: innerWidth, height: innerHeight }, expanded, drawer, tab, workspace: bounds(document.querySelector('.meeting-workspace')), transcript: drawer ? bounds(document.querySelector('.workspace-transcript')) : null });
        await report({ phase: 'case', cases: state.cases });
        if (!expanded && ((width === 1440 && drawer && tab === 'My notes') || (width === 480 && tab === 'My notes') || (width === 600 && !drawer && tab === 'Enhanced notes') || (width === 1100 && height === 400 && drawer && tab === 'My notes'))) await snapshot(state, `workspace-${width}x${height}-${drawer ? 'transcript' : tab === 'My notes' ? 'original' : 'enhanced'}.png`);
      }
    };
    hold(); await edit(original, ' Original pending edit - cafe', 'original'); await sleep(100); originalText = original.textContent;
    await matrix('My notes');
    await resize(1440, 900); await setSidebar(false); await setDrawer(true);
    click('Transcript'); await sleep(100);
    const context = document.querySelector('[aria-label="Context for AI summary"]');
    Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set.call(context, 'Context retained across continuous resize — 日本語');
    context.dispatchEvent(new Event('input', { bubbles: true }));
    const transcriptNode = [...document.querySelectorAll('.transcript-scroll')].find(visible);
    transcriptNode.scrollTop = 240;
    for (let width = 1440; width >= 480; width -= 16) await resize(width, Math.round(400 + (width - 480) * 500 / 960), 20);
    for (let width = 480; width <= 1440; width += 16) await resize(width, Math.round(400 + (width - 480) * 500 / 960), 20);
    await sleep(350); layoutCheck();
    assert(document.querySelector('#original-notes [contenteditable="true"]') === original && document.querySelector('#enhanced-notes [contenteditable="true"]') === enhanced, 'Continuous resize remounted an editor');
    assert(context.value === 'Context retained across continuous resize — 日本語', 'Transcript context reset');
    assert([...document.querySelectorAll('.transcript-scroll')].find(visible) === transcriptNode, 'Transcript scroller remounted');
    assert(transcriptNode.scrollTop > 0, `Transcript scroll position reset: ${transcriptNode.scrollTop}`);
    assert(!calls.some(command => /^(start_recording|stop_recording|pause_recording|resume_recording)/.test(command)), 'Resizing invoked recording lifecycle');
    assert(calls.includes('api_save_meeting_notes'), 'Original pending save was not exercised');
    assert(!(await nativeInvoke('api_get_meeting_notes', { meetingId: state.id })).notes_json.includes('Original pending edit'), 'Original edit was saved before the pending-save barrier');
    releaseSaves();
    await wait(async () => (await nativeInvoke('api_get_meeting_notes', { meetingId: state.id })).notes_json.includes('Original pending edit'), 'original pending save');
    await setDrawer(false);
    original.focus(); original.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', code: 'KeyZ', metaKey: true, bubbles: true })); await sleep(100);
    assert(!original.textContent.includes('Original pending edit'), 'Original undo history lost after resize');
    original.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', code: 'KeyZ', metaKey: true, shiftKey: true, bubbles: true })); await sleep(100);
    assert(original.textContent.includes('Original pending edit'), 'Original redo history lost after resize');
    click('Enhanced notes'); await wait(() => visible(enhanced), 'actual enhanced tab navigation');
    hold(); await edit(enhanced, ' Enhanced pending edit - Unicode', 'enhanced'); await sleep(100); enhancedText = enhanced.textContent;
    await matrix('Enhanced notes');
    assert(calls.includes('api_save_enhancement_version'), 'Enhanced pending save was not exercised');
    assert(!(await nativeInvoke('api_get_meeting_enhancements', { meetingId: state.id })).versions.find(version => version.id === 'responsive-approved').document_json.includes('Enhanced pending edit'), 'Enhanced edit was saved before the pending-save barrier');
    releaseSaves();
    await wait(async () => (await nativeInvoke('api_get_meeting_enhancements', { meetingId: state.id })).versions.some(version => version.id === 'responsive-approved' && version.document_json.includes('Enhanced pending edit')), 'enhanced pending save');
    await setDrawer(false); await resize(1440, 900);
    enhanced.focus(); enhanced.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', code: 'KeyZ', metaKey: true, bubbles: true })); await sleep(100);
    assert(!enhanced.textContent.includes('Enhanced pending edit'), 'Enhanced undo history lost after resize');
    enhanced.dispatchEvent(new KeyboardEvent('keydown', { key: 'z', code: 'KeyZ', metaKey: true, shiftKey: true, bubbles: true })); await sleep(100);
    assert(enhanced.textContent.includes('Enhanced pending edit'), 'Enhanced redo history lost after resize');
    await resize(480, 400); await setDrawer(true);
    window.dispatchEvent(new CustomEvent('workspace-note-source', { detail: { meetingId: state.id, blockId: 'intro' } }));
    await wait(() => visible(original), 'source navigation back to the original notes');
    assert(document.querySelector('[aria-controls="meeting-transcript-panel"]').getAttribute('aria-expanded') === 'false', 'Source navigation left notes hidden behind the drawer');
    state.continuity = { original_editor_identity: true, enhanced_editor_identity: true, pending_saves: true, original_undo_redo: true, enhanced_undo_redo: true, selected_version: true, transcript_context: true, transcript_scroll: true, note_source_navigation: true, recording_lifecycle_calls: 0, continuous_resize_steps: 122, physical_recording_tested: false };
    for (const [width, height] of sizes) {
      await resize(width, height); document.querySelector('[aria-label="Export meeting"]').click();
      await wait(() => document.querySelector('select[aria-label="Export format"]'), 'workspace export'); await sleep(250); layoutCheck();
      state.cases.push({ route: 'workspace-export', width, height }); click('Close'); await sleep(200);
    }
    state.phase = 'library'; sessionStorage.setItem('responsive-smoke', JSON.stringify(state));
    await sleep(1500); location.href = '/library';
  } catch (error) {
    releaseSaves();
    await report({ ok: false, phase: 'failed', screenshot: 'native-failure.png', error: String(error), native_webview: true, cases: state?.cases ?? [], interaction_cases: state?.interactionCases ?? [], runtime_errors: runtimeErrors, design_cases: state?.designCases ?? [], recording_visual_fixtures: state?.recordingVisualFixtures, settings_diagnostics: (() => {
      const panel = document.querySelector('[role="tabpanel"][data-state="active"]');
      if (!panel) return null;
      const r = panel.getBoundingClientRect();
      return { panel: bounds(panel), overflowing_descendants: [...panel.querySelectorAll('*')].filter(node => {
        const b = node.getBoundingClientRect();
        return visible(node) && (b.left < r.left - 1 || b.right > r.right + 1 || node.scrollWidth > node.clientWidth + 1);
      }).slice(0, 30).map(node => ({ tag: node.tagName, class: node.className, bounds: bounds(node), position: getComputedStyle(node).position, overflow_x: getComputedStyle(node).overflowX, text: node.textContent.slice(0, 120) })) };
    })(), diagnostics: [...document.querySelectorAll('.workspace-transcript, .meeting-details-layout, .details-transcript, .details-summary, .transcript-scroll')].map(node => ({ class: node.className, hidden: node.hidden, state: node.dataset.state, display: getComputedStyle(node).display, bounds: bounds(node) })), body: document.body.innerText.slice(0, 2000) }).catch(() => {});
  }
})();
