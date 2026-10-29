'use client';

import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// Matches audio::recording_levels::RecordingAudioLevel in the Rust pipeline.
export interface RecordingAudioLevel {
  microphone: number;
  system: number;
}

const SILENT_BARS = ['4px', '4px', '4px'];

// Serialize visibility changes so a slow disable cannot overtake a new enable.
let monitoringQueue = Promise.resolve();
function setMonitoring(enabled: boolean) {
  monitoringQueue = monitoringQueue.then(async () => {
    await invoke('set_recording_level_monitoring', { enabled });
  }).catch(error => {
    console.error('Failed to update recording meter visibility:', error);
  });
}

function normalizedLevel(level: number): number {
  return Number.isFinite(level) ? Math.max(0, Math.min(1, level)) : 0;
}

function barHeightsForLevel(level: number): string[] {
  // Map -60 dBFS through 0 dBFS onto the existing 4–30 px bars.
  const scaled = level > 0 ? Math.max(0, (20 * Math.log10(level) + 60) / 60) : 0;
  // A fixed decorative profile, not a separate RMS/peak analysis.
  const side = `${Math.round(4 + 20 * scaled)}px`;
  return [side, `${Math.round(4 + 26 * scaled)}px`, side];
}

export function useRecordingAudioLevels(enabled: boolean): string[] {
  const [barHeights, setBarHeights] = useState(SILENT_BARS);
  const [visible, setVisible] = useState(() =>
    typeof document === 'undefined' || document.visibilityState !== 'hidden',
  );
  const active = enabled && visible;

  useEffect(() => {
    // A webview reload may skip the previous instance's React cleanup.
    // Clear any old native subscription before this instance registers its own.
    setMonitoring(false);
    const onVisibilityChange = () => setVisible(document.visibilityState !== 'hidden');
    document.addEventListener('visibilitychange', onVisibilityChange);
    return () => document.removeEventListener('visibilitychange', onVisibilityChange);
  }, []);

  useEffect(() => {
    setBarHeights(SILENT_BARS);
    if (!active) return;

    let disposed = false;
    let unlisten: (() => void) | undefined;
    let monitoringRequested = false;
    let staleTimer: ReturnType<typeof setTimeout> | undefined;

    listen<RecordingAudioLevel>('recording-audio-level', ({ payload }) => {
      if (disposed) return;

      const level = Math.max(
        normalizedLevel(payload.microphone),
        normalizedLevel(payload.system),
      );
      const nextHeights = barHeightsForLevel(level);
      setBarHeights(previous => previous.every((height, index) => height === nextHeights[index])
        ? previous
        : nextHeights);

      clearTimeout(staleTimer);
      staleTimer = setTimeout(() => {
        if (!disposed) setBarHeights(SILENT_BARS);
      }, 500);
    }).then(removeListener => {
      // Registration can finish after pause, navigation, or unmount.
      if (disposed) removeListener();
      else {
        unlisten = removeListener;
        monitoringRequested = true;
        setMonitoring(true);
      }
    }).catch(error => {
      if (!disposed) console.error('Failed to listen for recording audio levels:', error);
    });

    return () => {
      disposed = true;
      clearTimeout(staleTimer);
      unlisten?.();
      if (monitoringRequested) setMonitoring(false);
    };
  }, [active]);

  return active ? barHeights : SILENT_BARS;
}
