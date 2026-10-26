(async () => {
  if (window.__recordingSmokeRunning) return;
  window.__recordingSmokeRunning = true;
  const invoke = (command, args) => window.__TAURI_INTERNALS__.invoke(command, args);
  const report = value => invoke('notepad_smoke_report', { report: { native_webview: true, ...value } });
  const wait = async (check, label, seconds = 40) => {
    const deadline = Date.now() + seconds * 1000;
    while (Date.now() < deadline) {
      const result = await check(); if (result) return result;
      await new Promise(resolve => setTimeout(resolve, 150));
    }
    throw Error(`Timed out: ${label}`);
  };
  const click = label => {
    const button = [...document.querySelectorAll('button')].find(b => b.textContent.trim() === label);
    if (!button || button.disabled) throw Error(`Unavailable control: ${label}`);
    button.click();
  };
  try {
    if (sessionStorage.getItem('recording-smoke-finished')) return;
    await report({ ok: false, phase: 'setup' });
    const onboarding = await invoke('get_onboarding_status');
    if (!onboarding?.completed) {
      await invoke('complete_notes_only_setup'); location.reload(); return;
    }
    await wait(() => invoke('api_get_meetings').catch(() => null), 'native database ready');
    const probe = await invoke('recording_smoke_probe', {meetingId:null,playFixture:false});
    const provider = probe.fixture_provider || 'localWhisper';
    const model = provider === 'parakeet' ? 'parakeet-tdt-0.6b-v3-int8' : 'large-v3';
    await invoke('api_save_transcript_config', { provider, model, apiKey: null });
    await wait(() => [...document.querySelectorAll('button')].some(b => b.textContent.trim() === 'New meeting' && Object.keys(b).some(k => k.startsWith('__reactProps$'))), 'hydrated workspace');
    const dismissRecovery = async () => {
      const dialog=[...document.querySelectorAll('[role="dialog"]')].find(d=>d.innerText.includes('Recover Interrupted Meetings'));
      if (dialog) {
        [...dialog.querySelectorAll('button')].find(b=>b.textContent.trim()==='Cancel')?.click();
        await wait(()=>!document.querySelector('[role="dialog"]'), 'dismissed recovery');
      }
    };
    await dismissRecovery();
    const priorId = sessionStorage.getItem('workspace_meeting_id');
    click('New meeting');
    const id = await wait(() => {const value=sessionStorage.getItem('workspace_meeting_id');return value && value!==priorId ? value:null;}, 'new draft selection');
    const draft = await invoke('api_get_meeting_metadata', {meetingId:id});
    const title = await wait(() => {const input=document.querySelector('[aria-label="Meeting title"]');return input?.value===draft.title ? input:null;}, 'new draft editor');
    const props = title[Object.keys(title).find(k => k.startsWith('__reactProps$'))];
    props.onChange({ target: { value: 'Recording continuation fixture' } });
    await dismissRecovery();
    const editor = await wait(() => {const value=document.querySelector('.bn-editor[contenteditable="true"]');return value && !value.innerText.trim() ? value:null;}, 'empty draft notes editor');
    editor.focus(); document.execCommand('insertText', false, 'Retain these original notes across both recordings.');
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 's', metaKey: true, bubbles: true }));
    const original = await wait(async () => { const doc = await invoke('api_get_meeting_notes', { meetingId: id }); return doc?.notes_json?.includes('Retain these original notes') ? doc : null; }, 'saved notes');
    const before = await invoke('api_get_meetings');
    const startArgs = {micDeviceName:null,systemDeviceName:null,meetingName:null,meetingId:id};
    const cancelledId = crypto.randomUUID();
    await invoke('cancel_recording_start', {recordingSessionId:cancelledId});
    let cancelled=false;
    try { await invoke('start_recording_with_devices_and_meeting', {...startArgs,recordingSessionId:cancelledId}); }
    catch (error) { cancelled=String(error).toLowerCase().includes('cancel'); }
    if (!cancelled || (await invoke('get_recording_state')).phase!=='idle') throw Error('Cancelled start acquired recording ownership');
    // This isolated profile contains only Whisper; Parakeet has no fallback model.
    if (provider === 'localWhisper') {
    await invoke('api_save_transcript_config', {provider:'parakeet',model:'recording-smoke-missing',apiKey:null});
    let failed=false;
    try { await invoke('start_recording_with_devices_and_meeting', {...startArgs,recordingSessionId:crypto.randomUUID()}); }
    catch { failed=true; }
    if (!failed || (await invoke('get_recording_state')).phase!=='idle') throw Error('Failed model start retained recording ownership');
    }
    await invoke('api_save_transcript_config', {provider,model,apiKey:null});
    const sessions = [];
    const captures=[];
    let controls = null;
    for (let index = 0; index < 2; index++) {
      await report({ ok: false, phase: `starting-${index + 1}` });
      const button = await wait(() => { const b = document.querySelector('[aria-label="Start recording"]'); return b && !b.disabled ? b : null; }, 'enabled Record');
      button.click(); button.click();
      const native = await wait(async () => {
        const state = await invoke('get_recording_state');
        if (state.is_recording && state.phase === 'recording') return state;
        const alert = document.querySelector('.workspace-recording [role="alert"]');
        if (alert && state.phase === 'idle') throw Error(alert.innerText);
        return null;
      }, 'recording start', 135);
      captures.push(native.capture_sources);
      if (native.meeting_id !== id) throw Error('Record created a different meeting');
      const stop = await wait(() => { const b = document.querySelector('[aria-label="Stop recording"]'); return b && !b.disabled ? b : null; }, 'enabled Stop');
      if (index === 0) controls = await invoke('recording_smoke_probe', {meetingId:id,playFixture:false,exerciseControls:true});
      await invoke('recording_smoke_probe', {meetingId:id,playFixture:true});
      await new Promise(resolve => setTimeout(resolve, 1000));
      const advancing = await invoke('get_recording_state');
      if (!(advancing.audio_position > 0)) throw Error('No captured audio frames');
      await report({ ok: false, phase: `recording-${index + 1}`, screenshot: `recording-${index + 1}.png` });
      stop.click();
      await wait(async () => {
        const metadata = await invoke('api_get_meeting_metadata', { meetingId: id });
        const b = document.querySelector('[aria-label="Start recording"]');
        return metadata.status === 'completed' && b && !b.disabled;
      }, 'saved recording and reusable Record', 180);
      sessions.push((await invoke('api_get_recording_sessions', { meetingId: id })).filter(s => s.status === 'completed').at(-1));
      if (sessionStorage.getItem('workspace_meeting_id') !== id) throw Error('Saving changed meeting selection');
    }
    const after = await invoke('api_get_meetings');
    const finalNotes = await invoke('api_get_meeting_notes', { meetingId: id });
    const metadata = await invoke('api_get_meeting_metadata', { meetingId: id });
    const identities=items=>items.map(item=>({id:item.id,title:item.title})).sort((a,b)=>a.id.localeCompare(b.id));
    if (JSON.stringify(identities(before)) !== JSON.stringify(identities(after))) throw Error('Record changed the meeting list');
    if (finalNotes.notes_json !== original.notes_json) throw Error('Recording changed original notes');
    if (metadata.title !== 'Recording continuation fixture') throw Error('Recording changed the title');
    if (sessions.length !== 2 || sessions[0].id === sessions[1].id || sessions[0].folder_path === sessions[1].folder_path) throw Error('Recordings did not get separate session/audio ownership');
    if (!(sessions[1].timeline_offset >= sessions[0].duration_seconds && sessions[1].timeline_offset > 0)) throw Error('Continuous transcript offset is missing');
    const files=await invoke('recording_smoke_probe',{meetingId:id,playFixture:false});
    if (files.files.length!==2 || files.files.some(f=>!f.audio_saved)) throw Error('Separate audio files are unavailable');
    const passages=await invoke('api_get_meeting_transcripts',{meetingId:id,limit:500,offset:0});
    const channels=[...new Set(passages.transcripts.map(p=>p.audio_source))];
    sessionStorage.setItem('recording-smoke-finished', 'true');
    await report({ ok: true, provider, model, native_device_pause_resume: controls?.native_device_pause_resume === true, native_hide_reopen: controls?.native_hide_reopen === true, tray_control_tested: false, cancelled_start_admission: true, model_failure_recovery: provider === 'localWhisper', repeated_start_stop: true, rapid_click_admission: true, notes_preserved: true, title_preserved: true, same_meeting: true, separate_audio_files: true, continuous_timeline: true, session_count: 2, transcript_count:passages.total_count,captured_channels:channels,capture_sources:captures, screenshot: 'recording-completed.png' });
  } catch (error) {
    const state = await invoke('get_recording_state').catch(() => null);
    if (state?.is_recording) await invoke('stop_recording', { args: { save_path: '' } }).catch(() => {});
    else if (state?.recording_session_id) await invoke('cancel_recording_start', { recordingSessionId: state.recording_session_id }).catch(() => {});
    await report({ ok: false, phase: 'failed', error: String(error), screenshot: 'recording-failed.png' }).catch(() => {});
  }
})();
