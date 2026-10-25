'use client';

import { invoke } from '@tauri-apps/api/core';
import { appDataDir } from '@tauri-apps/api/path';
import { useCallback, useEffect, useState, useRef, useSyncExternalStore } from 'react';
import { Play, Pause, Square, Mic, AlertCircle, X } from 'lucide-react';
import { ProcessRequest, SummaryResponse } from '@/types/summary';
import { listen } from '@tauri-apps/api/event';
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import Analytics from '@/lib/analytics';
import { useRecordingControls, RecordingStatus } from '@/contexts/RecordingStateContext';
import type { TranscriptionErrorPayload } from '@/services/transcriptService';
import { recordingService } from '@/services/recordingService';

interface RecordingControlsProps {
  isRecording: boolean;
  barHeights: string[];
  onRecordingStop: (callApi?: boolean, recordingSessionId?: string | null) => void | Promise<void>;
  onRecordingStart: () => void | Promise<void>;
  onTranscriptReceived: (summary: SummaryResponse) => void;
  onTranscriptionError?: (message: string) => void;
  onStopInitiated?: () => void; // Called immediately when stop button is clicked
  isRecordingDisabled: boolean;
  isParentProcessing: boolean;
  selectedDevices?: {
    micDevice: string | null;
    systemDevice: string | null;
  };
  meetingName?: string;
}

export const RecordingControls: React.FC<RecordingControlsProps> = ({
  isRecording,
  barHeights,
  onRecordingStop,
  onRecordingStart,
  onTranscriptReceived,
  onTranscriptionError,
  onStopInitiated,
  isRecordingDisabled,
  isParentProcessing,
  selectedDevices,
  meetingName,
}) => {
  // Use global recording state context for pause state (syncs with tray operations)
  const recordingState = useRecordingControls();
  const sharedPending = useSyncExternalStore(recordingService.subscribeLifecycle, recordingService.getLifecyclePending, recordingService.getLifecyclePending);
  const isPaused = recordingState.isPaused;
  const isStartingRecording = recordingState.isStartingRecording;

  const [showPlayback, setShowPlayback] = useState(false);
  const [recordingPath, setRecordingPath] = useState<string | null>(null);
  const [transcript, setTranscript] = useState<string>('');
  const [isProcessing, setIsProcessing] = useState(false);
  const [isStarting, setIsStarting] = useState(false);
  const [isStopping, setIsStopping] = useState(false);
  const [isPausing, setIsPausing] = useState(false);
  const [isResuming, setIsResuming] = useState(false);
  const MIN_RECORDING_DURATION = 2000; // 2 seconds minimum recording time
  const [transcriptionErrors, setTranscriptionErrors] = useState(0);
  const [isValidatingModel, setIsValidatingModel] = useState(false);
  const [speechDetected, setSpeechDetected] = useState(false);
  const [deviceError, setDeviceError] = useState<{ title: string, message: string } | null>(null);
  // DOM disabled state is committed on the next render. Admit only one native
  // lifecycle action synchronously, including competing pause/stop clicks.
  const pendingAction = useRef<'start' | 'stop' | 'pause' | 'resume' | null>(null);

  const currentTime = 0;
  const duration = 0;
  const isPlaying = false;
  const progress = 0;

  const formatTime = (time: number) => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  useEffect(() => {
    const checkTauri = async () => {
      try {
        const result = await invoke('is_recording');
        console.log('Tauri is initialized and ready, is_recording result:', result);
      } catch (error) {
        console.error('Tauri initialization error:', error);
        alert('Failed to initialize recording. Please check the console for details.');
      }
    };
    checkTauri();
  }, []);

  const handleStartRecording = useCallback(async () => {
    if (pendingAction.current || recordingService.getLifecyclePending() || isRecording || isRecordingDisabled || isParentProcessing || recordingState.isStopping || isStarting || isValidatingModel || isStartingRecording) return;
    pendingAction.current = 'start';
    setIsStarting(true);
    console.log('Starting recording...');
    console.log('Selected devices:', selectedDevices);
    console.log('Meeting name:', meetingName);
    console.log('Current isRecording state:', isRecording);

    setShowPlayback(false);
    setTranscript(''); // Clear any previous transcript
    setSpeechDetected(false); // Reset speech detection on new recording
    setDeviceError(null);

    try {
      // Call the validation callback which will:
      // 1. Check if model is ready
      // 2. Show appropriate toast/modal
      // 3. Call backend if valid
      // 4. Update UI state
      await onRecordingStart();
    } catch (error) {
      console.error('Failed to start recording:', error);
      console.error('Error details:', {
        message: error instanceof Error ? error.message : String(error),
        name: error instanceof Error ? error.name : 'Unknown',
        stack: error instanceof Error ? error.stack : undefined
      });

      // Parse error message to provide user-friendly feedback
      const errorMsg = error instanceof Error ? error.message : String(error);

      // Check for device-related errors
      if (errorMsg.includes('microphone') || errorMsg.includes('mic') || errorMsg.includes('input')) {
        setDeviceError({
          title: 'Microphone Not Available',
          message: 'Unable to access your microphone. Please check that:\n• Your microphone is connected\n• The app has microphone permissions\n• No other app is using the microphone'
        });
      } else if (errorMsg.includes('system audio') || errorMsg.includes('speaker') || errorMsg.includes('output')) {
        setDeviceError({
          title: 'System Audio Not Available',
          message: 'Unable to capture system audio. Please check that:\n• A virtual audio device (like BlackHole) is installed\n• The app has screen recording permissions (macOS)\n• System audio is properly configured'
        });
      } else if (errorMsg.includes('permission')) {
        setDeviceError({
          title: 'Permission Required',
          message: 'Recording permissions are required. Please:\n• Grant microphone access in System Settings\n• Grant screen recording access for system audio (macOS)\n• Restart the app after granting permissions'
        });
      } else {
        setDeviceError({
          title: 'Recording Failed',
          message: errorMsg
        });
      }
    } finally {
      pendingAction.current = null;
      setIsStarting(false);
    }
  }, [onRecordingStart, isStarting, isValidatingModel, isStartingRecording, selectedDevices, meetingName, isRecording, isRecordingDisabled, isParentProcessing, recordingState.isStopping]);

  const stopRecordingAction = useCallback(async () => {
    setIsProcessing(true);
    const recordingSessionId = globalThis.sessionStorage?.getItem('recording_session_id');
    try {
      await recordingService.runLifecycleAction(async () => {
      let stoppedNatively = false;
      try {
        const dataDir = await appDataDir();
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        const savePath = `${dataDir}/recording-${timestamp}.wav`;
        await invoke('stop_recording', { args: { save_path: savePath, recording_session_id: recordingSessionId } });
        setRecordingPath(savePath);
        stoppedNatively = true;
        Analytics.trackTranscriptionSuccess();
      } catch (error) {
        console.error('Failed to stop recording:', error);
      }
      // Reconcile even if native capture already stopped. Finalize at most once,
      // and keep the controls busy until the parent's durable save settles.
      await onRecordingStop(stoppedNatively, recordingSessionId);
      });
    } finally {
      setIsProcessing(false);
      setIsStopping(false);
    }
  }, [onRecordingStop]);

  const handleStopRecording = useCallback(async () => {
    console.log('handleStopRecording called - isRecording:', isRecording, 'isStarting:', isStarting, 'isStopping:', isStopping, 'isStartingRecording:', isStartingRecording);
    if (pendingAction.current || recordingService.getLifecyclePending() || !isRecording || isStarting || isStopping || isStartingRecording) {
      console.log('Early return from handleStopRecording due to state check');
      return;
    }

    console.log('Stopping recording...');

    pendingAction.current = 'stop';
    try {
      // Notify parent immediately (for UI state updates)
      onStopInitiated?.();
      setIsStopping(true);
      await stopRecordingAction();
    } catch (error) {
      setDeviceError({ title: 'Recording could not be finalized', message: String(error) });
    } finally {
      pendingAction.current = null;
      setIsStopping(false);
    }
  }, [isRecording, isStarting, isStopping, isStartingRecording, stopRecordingAction, onStopInitiated]);

  const handlePauseRecording = useCallback(async () => {
    if (pendingAction.current || recordingService.getLifecyclePending() || !isRecording || isPaused || isPausing || isStopping || recordingState.isStopping) return;
    pendingAction.current = 'pause';

    console.log('Pausing recording...');
    setIsPausing(true);

    try {
      const recordingSessionId = globalThis.sessionStorage?.getItem('recording_session_id');
      await recordingService.runLifecycleAction(() => invoke('pause_recording', { recordingSessionId }));
      // isPaused state now managed by RecordingStateContext via events
      console.log('Recording paused successfully');
    } catch (error) {
      console.error('Failed to pause recording:', error);
      alert('Failed to pause recording. Please check the console for details.');
    } finally {
      pendingAction.current = null;
      setIsPausing(false);
    }
  }, [isRecording, isPaused, isPausing, isStopping, recordingState.isStopping]);

  const handleResumeRecording = useCallback(async () => {
    if (pendingAction.current || recordingService.getLifecyclePending() || !isRecording || !isPaused || isResuming || isStopping || recordingState.isStopping) return;
    pendingAction.current = 'resume';

    console.log('Resuming recording...');
    setIsResuming(true);

    try {
      const recordingSessionId = globalThis.sessionStorage?.getItem('recording_session_id');
      await recordingService.runLifecycleAction(() => invoke('resume_recording', { recordingSessionId }));
      // isPaused state now managed by RecordingStateContext via events
      console.log('Recording resumed successfully');
    } catch (error) {
      console.error('Failed to resume recording:', error);
      alert('Failed to resume recording. Please check the console for details.');
    } finally {
      pendingAction.current = null;
      setIsResuming(false);
    }
  }, [isRecording, isPaused, isResuming, isStopping, recordingState.isStopping]);

  useEffect(() => {
    return () => {
      // Cleanup on unmount if needed
    };
  }, []);

  useEffect(() => {
    console.log('Setting up recording event listeners');
    const unsubscribes: (() => void)[] = [];
    let disposed = false;
    const retain = (unlisten: () => void) => { if (disposed) unlisten(); else unsubscribes.push(unlisten); };

    const setupListeners = async () => {
      try {
        // Transcript error listener - handles both regular and actionable errors
        const transcriptErrorUnsubscribe = await listen('transcript-error', (event) => {
          console.log('transcript-error event received:', event);
          console.error('Transcription error received:', event.payload);
          const errorMessage = event.payload as string;

          Analytics.trackTranscriptionError(errorMessage);
          console.log('Tracked transcription error:', errorMessage);

          setTranscriptionErrors(prev => {
            const newCount = prev + 1;
            console.log('Transcription error count incremented:', newCount);
            return newCount;
          });
          setIsProcessing(false);
          console.log('Calling onRecordingStop(false) due to transcript error');
          onRecordingStop(false);
          if (onTranscriptionError) {
            onTranscriptionError(errorMessage);
          }
        });

        retain(transcriptErrorUnsubscribe);

        // Transcription error listener - handles structured error objects with actionable flag
        const transcriptionErrorUnsubscribe = await listen<TranscriptionErrorPayload>('transcription-error', (event) => {
          console.log('transcription-error event received:', event);
          console.error('Transcription error received:', event.payload);

          const errorMessage = event.payload.userMessage || event.payload.error;

          Analytics.trackTranscriptionError(errorMessage);
          console.log('Tracked transcription error:', errorMessage);

          setTranscriptionErrors(prev => {
            const newCount = prev + 1;
            console.log('Transcription error count incremented:', newCount);
            return newCount;
          });
          setIsProcessing(false);

          if (event.payload.phase === 'active') {
            console.log('Calling onRecordingStop(false) due to active transcription error');
            onRecordingStop(false);
          }

          // For actionable errors (like model loading failures), the main page will handle showing the model selector
          // For regular errors, they are handled by useModalState global listener which shows a toast
        });

        // Pause/Resume events are now handled by RecordingStateContext
        // No need for duplicate listeners here

        // Speech detected listener - for UX feedback when VAD detects speech
        retain(transcriptionErrorUnsubscribe);
        const speechDetectedUnsubscribe = await listen('speech-detected', (event) => {
          console.log('speech-detected event received:', event);
          setSpeechDetected(true);
        });

        retain(speechDetectedUnsubscribe);
        console.log('Recording event listeners set up successfully');
      } catch (error) {
        console.error('Failed to set up recording event listeners:', error);
      }
    };

    setupListeners();

    return () => {
      disposed = true;
      console.log('Cleaning up recording event listeners');
      unsubscribes.forEach(unsubscribe => {
        if (unsubscribe && typeof unsubscribe === 'function') {
          unsubscribe();
        }
      });
    };
  }, [onRecordingStop, onTranscriptionError]);

  return (
    <TooltipProvider>
      <div role="group" aria-label="Recording controls" className="min-w-0 max-w-full flex flex-col space-y-2">
        <div className="recording-dock glass-surface flex flex-wrap justify-center items-center">
          {isProcessing && !isParentProcessing ? (
            <div className="flex items-center space-x-2">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-primary"></div>
              <span className="text-sm text-muted-foreground">Processing recording...</span>
            </div>
          ) : (
            <>
              {showPlayback ? (
                <>
                  <button
                    onClick={handleStartRecording}
                    aria-label="Start another recording"
                    className="w-10 h-10 flex items-center justify-center bg-primary rounded-full text-primary-foreground hover:bg-primary/90 transition-colors"
                  >
                    <Mic size={16} />
                  </button>

                  <div className="w-px h-6 bg-border mx-1" />

                  <div className="flex items-center space-x-1 mx-2">
                    <div className="text-sm text-muted-foreground min-w-[40px]">
                      {formatTime(currentTime)}
                    </div>
                    <div
                      className="relative w-24 h-1 bg-border rounded-full"
                    >
                      <div
                        className="absolute h-full bg-primary rounded-full"
                        style={{ width: `${progress}%` }}
                      />
                    </div>
                    <div className="text-sm text-muted-foreground min-w-[40px]">
                      {formatTime(duration)}
                    </div>
                  </div>

                  <button
                    className="w-10 h-10 flex items-center justify-center bg-input rounded-full text-white cursor-not-allowed"
                    disabled
                  >
                    <Play size={16} />
                  </button>
                </>
              ) : (
                <>
                  {!isRecording ? (
                    // Start recording button
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <button
                          onClick={() => {
                            Analytics.trackButtonClick('start_recording', 'recording_controls');
                            handleStartRecording();
                          }}
                          disabled={sharedPending || isStarting || isStopping || recordingState.isStopping || isProcessing || isParentProcessing || isRecordingDisabled || isValidatingModel || isStartingRecording}
                          aria-label="Start recording"
                          className={`w-12 h-12 shrink-0 flex items-center justify-center ${isStarting || isProcessing || isValidatingModel || isStartingRecording ? 'bg-muted-foreground' : 'bg-primary hover:bg-primary/90'
                            } rounded-full text-white transition-colors relative`}
                        >
                          {isValidatingModel || isStartingRecording ? (
                            <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
                          ) : (
                            <Mic size={20} />
                          )}
                        </button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>Start recording</p>
                      </TooltipContent>
                    </Tooltip>
                  ) : (
                    // Recording controls (pause/resume + stop)
                    <>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <button
                            onClick={() => {
                              if (isPaused) {
                                Analytics.trackButtonClick('resume_recording', 'recording_controls');
                                handleResumeRecording();
                              } else {
                                Analytics.trackButtonClick('pause_recording', 'recording_controls');
                                handlePauseRecording();
                              }
                            }}
                            disabled={sharedPending || isPausing || isResuming || isStopping}
                            aria-label={isPaused ? 'Resume recording' : 'Pause recording'}
                            className={`w-10 h-10 shrink-0 flex items-center justify-center ${isPausing || isResuming || isStopping
                              ? 'bg-border border-2 border-input text-muted-foreground'
                              : 'bg-card border-2 border-input text-muted-foreground hover:border-input hover:bg-background'
                              } rounded-full transition-colors relative`}
                          >
                            {isPaused ? <Play size={16} /> : <Pause size={16} />}
                            {(isPausing || isResuming) && (
                              <div className="absolute -top-8 text-muted-foreground font-medium text-xs">
                                {isPausing ? 'Pausing...' : 'Resuming...'}
                              </div>
                            )}
                          </button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>{isPaused ? 'Resume recording' : 'Pause recording'}</p>
                        </TooltipContent>
                      </Tooltip>

                      <Tooltip>
                        <TooltipTrigger asChild>
                          <button
                            onClick={() => {
                              Analytics.trackButtonClick('stop_recording', 'recording_controls');
                              handleStopRecording();
                            }}
                            disabled={sharedPending || isStopping || isPausing || isResuming || isStartingRecording}
                            aria-label="Stop recording"
                            className={`w-10 h-10 shrink-0 flex items-center justify-center ${isStopping || isPausing || isResuming || isStartingRecording ? 'bg-muted-foreground' : 'bg-red-500 hover:bg-red-600'
                              } rounded-full text-white transition-colors relative`}
                          >
                            <Square size={16} />
                            {isStopping && (
                              <div className="absolute -top-8 text-muted-foreground font-medium text-xs">
                                Stopping...
                              </div>
                            )}
                          </button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>Stop recording</p>
                        </TooltipContent>
                      </Tooltip>
                    </>
                  )}

                  <span className="text-xs font-medium text-foreground" role="status">
                    {recordingState.status === RecordingStatus.ERROR ? 'Check recording status' : recordingState.isStopping || isStopping ? 'Stopping…' : isParentProcessing ? recordingState.isSaving ? 'Saving…' : 'Finishing…' : isStartingRecording ? 'Starting…' : isRecording ? isPaused ? 'Paused' : 'Recording' : 'Ready to record'}
                  </span>
                  <div className="flex items-center space-x-1 mx-1" aria-hidden="true">
                    {barHeights.map((height, index) => (
                      <div
                        key={index}
                        className={`w-1 rounded-full transition-all duration-200 ${isRecording ? isPaused ? 'bg-orange-500' : 'bg-red-500' : 'bg-primary/50'
                          }`}
                        style={{
                          height: isRecording && !isPaused ? height : '4px',
                          opacity: isPaused ? 0.6 : 1,
                        }}
                      />
                    ))}
                  </div>
                </>
              )}
            </>
          )}
        </div>

        {/* Show validation status only */}
        {isValidatingModel && (
          <div className="text-xs text-muted-foreground text-center mt-2">
            Validating speech recognition...
          </div>
        )}

        {/* Device error alert */}
        {isStartingRecording && <button className="text-sm underline mt-2" onClick={async () => {
          if (recordingService.cancelPendingStart()) return;
          try {
            const native = await recordingService.getRecordingState();
            if (native.recording_session_id) await recordingService.cancelRecordingStart(native.recording_session_id);
            recordingState.setStatus(RecordingStatus.ERROR, 'Cancellation requested. Check status before retrying.');
          } catch (error) { recordingState.setStatus(RecordingStatus.ERROR, String(error)); }
        }}>Cancel start</button>}
        {recordingState.status === 'error' && recordingState.statusMessage && !deviceError && (
          <Alert variant="destructive" className="mt-2"><AlertTitle>Recording could not start</AlertTitle><AlertDescription>{recordingState.statusMessage}</AlertDescription>
            <button className="mt-2 underline text-sm" onClick={() => window.dispatchEvent(new Event('recording-state-refresh'))}>Check status</button>
          </Alert>
        )}
        {deviceError && (
          <Alert variant="destructive" className="mt-4 border-red-300 bg-red-50">
            <AlertCircle className="h-5 w-5 text-red-600" />
            <button
              onClick={() => setDeviceError(null)}
              className="absolute right-3 top-3 text-red-600 hover:text-red-800 transition-colors"
              aria-label="Close alert"
            >
              <X className="h-4 w-4" />
            </button>
            <AlertTitle className="text-red-800 font-semibold mb-2">
              {deviceError.title}
            </AlertTitle>
            <AlertDescription className="text-red-700">
              {deviceError.message.split('\n').map((line, i) => (
                <div key={i} className={i > 0 ? 'ml-2' : ''}>
                  {line}
                </div>
              ))}
            </AlertDescription>
          </Alert>
        )}

        {/* {showPlayback && recordingPath && (
        <div className="text-sm text-muted-foreground px-4">
          Recording saved to: {recordingPath}
        </div>
      )} */}
      </div>
    </TooltipProvider>
  );
};
