import { useState, useCallback, useRef, useEffect } from 'react';
import { MeetingSummary, Summary, Transcript } from '@/types';
import { BlockNoteSummaryViewRef } from '@/components/AISummary/BlockNoteSummaryView';
import { CurrentMeeting, useSidebar } from '@/components/Sidebar/SidebarProvider';
import { invoke as invokeTauri } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import { hasVisibleSummaryContent } from '@/lib/summary-content';
import { LegacySummarySaveQueue, readDisplayedSummary } from '@/lib/legacy-summary-save';
import { registerLegacySummaryFlusher } from '@/lib/legacy-summary-flush';

interface UseMeetingDataProps {
  meeting: any;
  summaryData: MeetingSummary | null;
  onMeetingUpdated?: () => Promise<void>;
}

export function useMeetingData({ meeting, summaryData, onMeetingUpdated }: UseMeetingDataProps) {
  // State
  // Use prop directly since summary generation fetches transcripts independently
  const transcripts = meeting.transcripts;
  const [meetingTitle, setMeetingTitle] = useState(meeting.title || '+ New Call');
  const [aiSummary, setAiSummaryState] = useState<MeetingSummary | null>(summaryData);
  const [isSaving, setIsSaving] = useState(false);
  const [isSummaryDirty, setSummaryDirtyState] = useState(false);

  // Ref for BlockNoteSummaryView
  const blockNoteSummaryRef = useRef<BlockNoteSummaryViewRef>(null);
  const retainedEditor = useRef<BlockNoteSummaryViewRef | null>(null);
  const saveQueues = useRef(new Map<string, LegacySummarySaveQueue>());
  const legacyDrafts = useRef(new Map<LegacySummarySaveQueue, Record<string, unknown>>());
  const displayedQueue = useRef<LegacySummarySaveQueue | null>(null);
  const displayOwner = useRef<string | null>(null);
  const rememberDisplayed = useCallback((value: MeetingSummary | null) => {
    const displayed = readDisplayedSummary(value);
    if (!displayed) { displayedQueue.current = null; return; }
    const key = `${meeting.id}:${displayed.versionId}`;
    let queue = saveQueues.current.get(key);
    if (!queue) {
      queue = new LegacySummarySaveQueue(meeting.id, displayed.versionId, displayed,
        input => invokeTauri<{ revision: number }>('api_save_meeting_summary', { ...input }));
      saveQueues.current.set(key, queue);
    }
    displayedQueue.current = queue;
  }, [meeting.id]);
  if (displayOwner.current !== meeting.id) {
    displayOwner.current = meeting.id;
    rememberDisplayed(summaryData);
  }
  const setAiSummary = useCallback((value: MeetingSummary | null) => {
    rememberDisplayed(value);
    setAiSummaryState(value);
  }, [rememberDisplayed]);
  const setIsSummaryDirty = useCallback((value: boolean) => {
    if (blockNoteSummaryRef.current) retainedEditor.current = blockNoteSummaryRef.current;
    setSummaryDirtyState(value || legacyDrafts.current.size > 0);
  }, []);
  useEffect(() => registerLegacySummaryFlusher(meeting.id, async () => {
    const editor = blockNoteSummaryRef.current ?? retainedEditor.current;
    while (editor?.isDirty) await editor.saveSummary();
    while (legacyDrafts.current.size > 0) {
      for (const [queue, draft] of legacyDrafts.current) {
        await queue.save(draft);
        if (legacyDrafts.current.get(queue) === draft) legacyDrafts.current.delete(queue);
      }
    }
    for (const queue of saveQueues.current.values()) await queue.retryPending();
  }, () => legacyDrafts.current.size > 0
    || !!(blockNoteSummaryRef.current ?? retainedEditor.current)?.isDirty
    || [...saveQueues.current.values()].some(queue => queue.hasPending)), [meeting.id]);

  // Sidebar context
  const { setCurrentMeeting, setMeetings, meetings: sidebarMeetings } = useSidebar();

  // Sync aiSummary state when summaryData prop changes (fixes display of fetched summaries)
  useEffect(() => {
    console.log('[useMeetingData] Syncing summary data from prop:', summaryData ? 'present' : 'null');
    setAiSummary(summaryData);
  }, [summaryData, setAiSummary]);

  const handleSummaryChange = useCallback((newSummary: Summary) => {
    // Editing legacy sections does not change the identity of the displayed document.
    if (displayedQueue.current) legacyDrafts.current.set(displayedQueue.current,
      structuredClone({ MeetingName: meetingTitle, ...newSummary }));
    setSummaryDirtyState(true);
    setAiSummaryState(newSummary);
  }, [meetingTitle]);



  const queue = displayedQueue.current;
  const handleSaveSummary = useCallback(async (summary: MeetingSummary) => {
    if (!hasVisibleSummaryContent(summary)) {
      throw new Error('Summary contains no visible content to save.');
    }

    if (!queue) {
      throw new Error('Reopen the meeting to load its saved document version before editing.');
    }
    const formattedSummary = 'markdown' in summary || 'summary_json' in summary
      ? summary
      : { MeetingName: meetingTitle, ...summary };
    const legacyDraft = legacyDrafts.current.get(queue);
    await queue.save(formattedSummary as Record<string, unknown>);
    if (legacyDraft && legacyDrafts.current.get(queue) === legacyDraft) {
      legacyDrafts.current.delete(queue);
      setSummaryDirtyState(legacyDrafts.current.size > 0);
    }
    if (typeof window !== 'undefined') window.dispatchEvent(new Event('meeting-data-changed'));
  }, [meetingTitle, queue]);

  const saveAllChanges = useCallback(async () => {
    setIsSaving(true);
    try {
      // Save BlockNote editor changes if dirty
      if (blockNoteSummaryRef.current?.isDirty) {
        console.log('💾 Saving BlockNote editor changes...');
        await blockNoteSummaryRef.current.saveSummary();
      } else if (aiSummary) {
        await handleSaveSummary(aiSummary);
      }

      toast.success("Changes saved successfully");
    } catch (error) {
      console.error('Failed to save changes:', error);
      toast.error("Failed to save changes", { description: String(error) });
    } finally {
      setIsSaving(false);
    }
  }, [aiSummary, handleSaveSummary]);

  // Update meeting title from external source (e.g., AI summary)
  const updateMeetingTitle = useCallback((newTitle: string) => {
    setMeetingTitle(newTitle);
    const updatedMeetings = sidebarMeetings.map((m: CurrentMeeting) =>
      m.id === meeting.id ? { id: m.id, title: newTitle } : m
    );
    setMeetings(updatedMeetings);
    setCurrentMeeting({ id: meeting.id, title: newTitle });
  }, [meeting.id, sidebarMeetings, setMeetings, setCurrentMeeting]);

  return {
    // State
    transcripts,
    meetingTitle,
    aiSummary,
    isSaving,
    isSummaryDirty,
    blockNoteSummaryRef,

    // Setters
    setMeetingTitle,
    setAiSummary,
    setIsSummaryDirty,

    // Handlers
    handleSummaryChange,
    handleSaveSummary,
    saveAllChanges,
    updateMeetingTitle,
  };
}
