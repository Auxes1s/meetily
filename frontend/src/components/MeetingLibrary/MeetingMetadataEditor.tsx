'use client';
import { useEffect, useRef, useState } from 'react';
import { Button } from '@/components/ui/button';
import { meetingLibraryService } from '@/services/meetingLibraryService';
import { manualMetadataLines } from '@/lib/meeting-library';

export function MeetingMetadataEditor({ meetingId, participants, tags, onSaved, onCancel, onSavingChange }: {
  meetingId: string; participants: string[]; tags: string[];
  onSaved: () => void; onCancel: () => void;
  onSavingChange?: (saving: boolean) => void;
}) {
  const [people, setPeople] = useState(participants.join('\n'));
  const [labels, setLabels] = useState(tags.join('\n'));
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const draft = useRef({ people, labels });
  const savingRef = useRef(false);
  const mounted = useRef(false);
  const identity = useRef(meetingId);
  identity.current = meetingId;
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);
  return <form className="space-y-4 rounded-xl border border-border bg-muted/40 p-4" onSubmit={async event => {
    event.preventDefault();
    if (savingRef.current) return;
    savingRef.current = true;
    const target = meetingId;
    const input = { people: manualMetadataLines(draft.current.people), labels: manualMetadataLines(draft.current.labels) };
    const owns = () => mounted.current && identity.current === target;
    onSavingChange?.(true);
    setSaving(true); setError(null);
    try {
      await meetingLibraryService.updateMetadata(target, input.people, input.labels);
      if (owns()) onSaved();
    } catch (failure) { if (owns()) setError(failure instanceof Error ? failure.message : String(failure)); }
    finally {
      savingRef.current = false;
      onSavingChange?.(false);
      if (mounted.current) setSaving(false);
    }
  }}>
    <p className="text-sm text-muted-foreground">Enter one participant or tag per line. Leave the field empty to clear it.</p>
    <div className="library-filters grid gap-3">
      <label className="text-sm font-medium">Manual participants
        <textarea value={people} disabled={saving} onChange={event => { if (!savingRef.current) { draft.current.people = event.target.value; setPeople(event.target.value); } }} rows={3} maxLength={6500} className="mt-1.5 w-full rounded-xl border border-input bg-card p-3 text-sm font-normal leading-relaxed focus:outline-none focus:ring-2 focus:ring-ring" />
      </label>
      <label className="text-sm font-medium">Tags
        <textarea value={labels} disabled={saving} onChange={event => { if (!savingRef.current) { draft.current.labels = event.target.value; setLabels(event.target.value); } }} rows={3} maxLength={4500} className="mt-1.5 w-full rounded-xl border border-input bg-card p-3 text-sm font-normal leading-relaxed focus:outline-none focus:ring-2 focus:ring-ring" />
      </label>
    </div>
    {error && <p role="alert" className="text-sm text-red-700">{error}</p>}
    <div className="flex flex-wrap gap-2"><Button size="sm" disabled={saving} type="submit">{saving ? 'Saving…' : 'Save metadata'}</Button><Button size="sm" variant="outline" disabled={saving} type="button" onClick={() => { if (!savingRef.current) onCancel(); }}>Cancel</Button></div>
  </form>;
}
