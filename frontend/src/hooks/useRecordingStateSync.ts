import { useState, useEffect } from 'react';
import { useRecordingControls, RecordingStatus } from '@/contexts/RecordingStateContext';

interface UseRecordingStateSyncReturn {
  isBackendRecording: boolean;
  isRecordingDisabled: boolean;
  setIsRecordingDisabled: (value: boolean) => void;
}

/** Compatibility state for the coordinator; native polling belongs to its provider. */
export function useRecordingStateSync(
  isRecording: boolean,
  setIsRecording: (value: boolean) => void,
  setIsMeetingActive: (value: boolean) => void
): UseRecordingStateSyncReturn {
  const [isRecordingDisabled, setIsRecordingDisabled] = useState(false);

  const backend = useRecordingControls();
  useEffect(() => {
    // RecordingStateProvider owns native reconciliation and lifecycle events.
    // Reconcile the coordinator's compatibility state without another timer.
    if (backend.status === RecordingStatus.STARTING) return;
    if (backend.isRecording !== isRecording) setIsRecording(backend.isRecording);
    if (backend.isRecording) setIsMeetingActive(true);
  }, [backend.isRecording, backend.status, isRecording, setIsRecording, setIsMeetingActive]);

  return {
    isBackendRecording: backend.isRecording,
    isRecordingDisabled,
    setIsRecordingDisabled,
  };
}
