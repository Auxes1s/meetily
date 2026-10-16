import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import Analytics from '@/lib/analytics';
import { applyPinnedSummaryLanguageToMeeting } from '@/lib/summary-language-preferences';
import { toast } from 'sonner';

export interface AudioFileInfo {
  path: string;
  filename: string;
  duration_seconds: number;
  size_bytes: number;
  format: string;
}

export interface ImportProgress {
  stage: string;
  progress_percentage: number;
  message: string;
}

export interface ImportResult {
  meeting_id: string;
  title: string;
  segments_count: number;
  duration_seconds: number;
}

export interface ImportError {
  error: string;
}

export type ImportStatus = 'idle' | 'validating' | 'processing' | 'complete' | 'error';

export interface UseImportAudioOptions {
  onComplete?: (result: ImportResult) => void;
  onError?: (error: string) => void;
}

export interface UseImportAudioReturn {
  status: ImportStatus;
  fileInfo: AudioFileInfo | null;
  progress: ImportProgress | null;
  error: string | null;
  isProcessing: boolean;
  isBusy: boolean;
  isCancelling: boolean;
  selectFile: () => Promise<AudioFileInfo | null>;
  validateFile: (path: string) => Promise<AudioFileInfo | null>;
  startImport: (
    sourcePath: string,
    title: string,
    language?: string | null,
    model?: string | null,
    provider?: string | null
  ) => Promise<void>;
  cancelImport: () => Promise<void>;
  reset: () => void;
}

export function useImportAudio({
  onComplete,
  onError,
}: UseImportAudioOptions = {}): UseImportAudioReturn {
  const [status, setStatus] = useState<ImportStatus>('idle');
  const [fileInfo, setFileInfo] = useState<AudioFileInfo | null>(null);
  const [progress, setProgress] = useState<ImportProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isCancelling, setIsCancelling] = useState(false);
  const generation = useRef(0);
  const validating = useRef(false);
  const mounted = useRef(true);
  const activeImport = useRef<{ submitted: boolean; submission: Promise<unknown> | null; terminal: boolean } | null>(null);
  const cancellation = useRef<Promise<void> | null>(null);

  // Stable refs for callbacks to avoid listener re-registration on every render
  const onCompleteRef = useRef(onComplete);
  const onErrorRef = useRef(onError);
  useEffect(() => { onCompleteRef.current = onComplete; }, [onComplete]);
  useEffect(() => { onErrorRef.current = onError; }, [onError]);

  // Cancellation guard: prevents late events from updating state after cancel
  const isCancelledRef = useRef(false);

  // Set up event listeners (registered once, use refs for callbacks)
  useEffect(() => {
    mounted.current = true;
    const unlisteners: UnlistenFn[] = [];
    const cleanedUpRef = { current: false };

    const setupListeners = async () => {
      // Progress events
      const unlistenProgress = await listen<ImportProgress>(
        'import-progress',
        (event) => {
          if (isCancelledRef.current || !activeImport.current || activeImport.current.terminal) return;
          setProgress(event.payload);
          setStatus('processing');
        }
      );
      if (cleanedUpRef.current) {
        unlistenProgress();
        return;
      }
      unlisteners.push(unlistenProgress);

      // Completion event
      const unlistenComplete = await listen<ImportResult>(
        'import-complete',
        async (event) => {
          const owner = activeImport.current;
          if (isCancelledRef.current || !owner || owner.terminal) return;
          owner.terminal = true;
          try {
            await Analytics.track('import_audio_completed', {
              success: 'true',
              duration_seconds: event.payload.duration_seconds.toString(),
              segments_count: event.payload.segments_count.toString()
            });
          } catch (cause) { console.warn('Failed to track completed import:', cause); }
          if (!mounted.current || isCancelledRef.current || activeImport.current !== owner) return;
          try {
            await applyPinnedSummaryLanguageToMeeting(event.payload.meeting_id);
          } catch (cause) {
            console.warn('Failed to apply pinned summary language to imported meeting:', cause);
            toast.warning('Could not apply default summary language', {
              description: 'The imported meeting was saved, but the default summary language was not applied.',
            });
          }
          if (!mounted.current || isCancelledRef.current || activeImport.current !== owner) return;
          activeImport.current = null;
          setStatus('complete');
          setProgress(null);
          onCompleteRef.current?.(event.payload);
        }
      );
      if (cleanedUpRef.current) {
        unlistenComplete();
        unlisteners.forEach(u => u());
        return;
      }
      unlisteners.push(unlistenComplete);

      // Error event
      const unlistenError = await listen<ImportError>(
        'import-error',
        async (event) => {
          const owner = activeImport.current;
          if (isCancelledRef.current || !owner || owner.terminal) return;
          owner.terminal = true;
          activeImport.current = null;
          setStatus('error');
          setError(event.payload.error);
          onErrorRef.current?.(event.payload.error);
          void Analytics.trackError('import_audio_failed', event.payload.error).catch(cause => console.warn('Failed to track import error:', cause));
        }
      );
      if (cleanedUpRef.current) {
        unlistenError();
        unlisteners.forEach(u => u());
        return;
      }
      unlisteners.push(unlistenError);
    };

    setupListeners();

    return () => {
      cleanedUpRef.current = true;
      mounted.current = false;
      generation.current++;
      isCancelledRef.current = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  const validate = useCallback(async (command: string, args?: { path: string }): Promise<AudioFileInfo | null> => {
    if (validating.current || activeImport.current) return null;
    validating.current = true;
    const owner = ++generation.current;
    setStatus('validating');
    setError(null);
    try {
      const result = await invoke<AudioFileInfo | null>(command, args);
      if (!mounted.current || generation.current !== owner) return null;
      if (result) setFileInfo(result);
      setStatus('idle');
      return result;
    } catch (cause: any) {
      if (!mounted.current || generation.current !== owner) return null;
      const message = typeof cause === 'string' ? cause : cause?.message || String(cause);
      setStatus('error');
      setError(message);
      onErrorRef.current?.(message);
      return null;
    } finally { validating.current = false; }
  }, []);

  const selectFile = useCallback(() => validate('select_and_validate_audio_command'), [validate]);
  const validateFile = useCallback((path: string) => validate('validate_audio_file_command', { path }), [validate]);

  // Start the import process
  const startImport = useCallback(
    async (
      sourcePath: string,
      title: string,
      language?: string | null,
      model?: string | null,
      provider?: string | null
    ) => {
      if (validating.current || activeImport.current || cancellation.current) return;
      const owner = { submitted: false, submission: null as Promise<unknown> | null, terminal: false };
      activeImport.current = owner;
      isCancelledRef.current = false;
      setStatus('processing');
      setError(null);
      setProgress(null);

      try {
        if (fileInfo) {
          await Analytics.track('import_audio_started', {
            file_size_bytes: fileInfo.size_bytes.toString(),
            duration_seconds: fileInfo.duration_seconds.toString(),
            language: language || 'auto',
            model_provider: provider || '',
            model_name: model || ''
          }).catch(cause => console.warn('Failed to track import start:', cause));
        }

        if (isCancelledRef.current || activeImport.current !== owner || !mounted.current) return;
        owner.submitted = true;
        owner.submission = invoke('start_import_audio_command', {
          sourcePath,
          title,
          language: language || null,
          model: model || null,
          provider: provider || null,
        });
        await owner.submission;
      } catch (err: any) {
        if (activeImport.current !== owner || !mounted.current) return;
        activeImport.current = null;
        if (isCancelledRef.current) return;
        setStatus('error');
        const errorMsg = typeof err === 'string' ? err : (err?.message || String(err) || 'Failed to start import');
        setError(errorMsg);

        void Analytics.trackError('import_audio_failed', errorMsg).catch(cause => console.warn('Failed to track import error:', cause));

        onErrorRef.current?.(errorMsg);
      }
    },
    [fileInfo]
  );

  // Cancellation retains admission until the native worker has released its owner.
  const cancelImport = useCallback((): Promise<void> => {
    if (cancellation.current) return cancellation.current;
    const owner = activeImport.current;
    if (!owner) return Promise.resolve();
    isCancelledRef.current = true;
    setIsCancelling(true);
    const cancel = async () => {
      try {
        if (owner.submitted) {
          await owner.submission;
          await invoke('cancel_import_command');
          const deadline = Date.now() + 30000;
          while (await invoke<boolean>('is_import_in_progress_command')) {
            if (Date.now() >= deadline) throw new Error('Import cancellation is still pending. Try cancelling again.');
            await new Promise(resolve => setTimeout(resolve, 100));
          }
        }
        if (activeImport.current === owner) activeImport.current = null;
        if (mounted.current) {
          setStatus('idle');
          setProgress(null);
          setError(null);
        }
      } catch (cause) {
        isCancelledRef.current = false;
        if (mounted.current) {
          if (!activeImport.current) setStatus('error');
          setError(`Could not cancel import: ${String(cause)}`);
        }
        throw cause;
      } finally {
        cancellation.current = null;
        if (mounted.current) setIsCancelling(false);
      }
    };
    // Deferral lets duplicate same-tick callers observe this shared promise.
    cancellation.current = Promise.resolve().then(cancel);
    return cancellation.current;
  }, []);

  const reset = useCallback(() => {
    if (activeImport.current || cancellation.current) return;
    generation.current++;
    isCancelledRef.current = true;
    setStatus('idle');
    setFileInfo(null);
    setProgress(null);
    setError(null);
  }, []);

  return {
    status,
    fileInfo,
    progress,
    error,
    isProcessing: status === 'processing',
    isBusy: status === 'processing' || status === 'validating',
    isCancelling,
    selectFile,
    validateFile,
    startImport,
    cancelImport,
    reset,
  };
}
