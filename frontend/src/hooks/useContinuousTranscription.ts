'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** Native persistence also controls recordings started outside the WebView. */
export function useContinuousTranscription() {
  const [enabled, setEnabled] = useState(true);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [loadFailed, setLoadFailed] = useState(false);
  const mounted = useRef(false);
  const pending = useRef(false);

  const reload = useCallback(async () => {
    if (pending.current) return;
    pending.current = true;
    setLoading(true);
    setError(null);
    setLoadFailed(false);
    try {
      const saved = await invoke<boolean>('get_continuous_transcription');
      if (mounted.current) setEnabled(saved);
    } catch (cause) {
      if (mounted.current) {
        setLoadFailed(true);
        setError(`Could not load this setting: ${String(cause)}`);
      }
    } finally {
      pending.current = false;
      if (mounted.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    mounted.current = true;
    void reload();
    return () => { mounted.current = false; };
  }, [reload]);

  const update = useCallback(async (value: boolean) => {
    if (pending.current) return;
    pending.current = true;
    setLoading(true);
    setError(null);
    try {
      await invoke('set_continuous_transcription', { enabled: value });
      // Only show the new choice after native disk persistence acknowledges it.
      if (mounted.current) setEnabled(value);
    } catch (cause) {
      if (mounted.current) setError(`Could not save this setting: ${String(cause)}`);
    } finally {
      pending.current = false;
      if (mounted.current) setLoading(false);
    }
  }, []);

  return { enabled, loading, error, loadFailed, reload, update };
}
