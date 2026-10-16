'use client';
import { createContext, useContext, useState } from 'react';
import { useSidebar } from '@/components/Sidebar/SidebarProvider';
import { useRecordingStart } from '@/hooks/useRecordingStart';
import { useRecordingStop } from '@/hooks/useRecordingStop';
import { useRecordingStateSync } from '@/hooks/useRecordingStateSync';
import { useModalState } from '@/hooks/useModalState';
import { useConfig } from '@/contexts/ConfigContext';
import { SettingsModals } from '@/app/_components/SettingsModal';

const Context = createContext<{
  handleRecordingStart: (meetingId?: string | null) => Promise<void>;
  handleRecordingStop: (callApi: boolean, recordingSessionId?: string | null) => Promise<void>;
  setIsStopping: (value: boolean) => void;
  isRecordingDisabled: boolean;
  showModal: ReturnType<typeof useModalState>['showModal'];
} | null>(null);
export function RecordingCoordinatorProvider({ children }: { children: React.ReactNode }) {
  const [isRecording, setIsRecording] = useState(false);
  const { setIsMeetingActive } = useSidebar();
  const { transcriptModelConfig } = useConfig();
  const { modals, messages, showModal, hideModal } = useModalState(transcriptModelConfig);
  const { isRecordingDisabled, setIsRecordingDisabled } = useRecordingStateSync(isRecording, setIsRecording, setIsMeetingActive);
  const { handleRecordingStart } = useRecordingStart(isRecording, setIsRecording, showModal);
  const { handleRecordingStop, setIsStopping } = useRecordingStop(setIsRecording, setIsRecordingDisabled);
  return <Context.Provider value={{ handleRecordingStart, handleRecordingStop, setIsStopping, isRecordingDisabled, showModal }}>
    <SettingsModals modals={modals} messages={messages} onClose={hideModal} />{children}
  </Context.Provider>;
}
export function useRecordingCoordinator() {
  const value = useContext(Context);
  if (!value) throw new Error('Recording coordinator is unavailable');
  return value;
}
