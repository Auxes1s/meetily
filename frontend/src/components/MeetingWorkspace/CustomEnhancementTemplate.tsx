'use client';
import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@/components/ui/button';

const field = 'mt-1 block w-full min-w-0 rounded-xl border border-input bg-card px-3 py-2 text-sm font-normal text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring';

interface TemplateSection { title: string; instruction: string; format: string }
interface TemplateDraft { id: string; name: string; description: string; sections: TemplateSection[] }
export function CustomEnhancementTemplate({ selectedId, onSaved }: { selectedId: string; onSaved: (id: string) => Promise<void> }) {
  const [draft, setDraft] = useState<TemplateDraft>({
    id: selectedId.startsWith('user_') ? selectedId : 'user_meeting_notes',
    name: 'My meeting notes',
    description: 'Organize the meeting around my priorities.',
    sections: [{ title: 'Priority topics', instruction: 'Organize the main topics and preserve qualifications.', format: 'paragraph' }],
  });
  const draftRef = useRef(draft);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const mounted = useRef(false);
  const selection = useRef({ id: selectedId });
  if (selection.current.id !== selectedId) selection.current = { id: selectedId };
  useEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; };
  }, []);
  const edit = (update: (current: TemplateDraft) => TemplateDraft) => {
    if (inFlight.current) return;
    draftRef.current = update(draftRef.current);
    setDraft(draftRef.current);
  };
  const action = async (work: (input: TemplateDraft, owns: () => boolean) => Promise<void>) => {
    if (inFlight.current) return;
    inFlight.current = true;
    const owner = selection.current;
    const owns = () => mounted.current && selection.current === owner;
    setBusy(true); setError(null);
    try { await work(draftRef.current, owns); }
    catch (failure) { if (owns()) setError(String(failure)); }
    finally {
      inFlight.current = false;
      if (mounted.current) setBusy(false);
    }
  };
  const load = () => action(async (input, owns) => {
    const template = await invoke<{ name: string; description: string; section_definitions: TemplateSection[] }>('api_get_template_details', { templateId: input.id });
    if (!owns()) return;
    draftRef.current = { id: input.id, name: template.name, description: template.description, sections: template.section_definitions };
    setDraft(draftRef.current);
  });
  const save = () => action(async (input, owns) => {
    const { id, name, description, sections } = input;
    await invoke('api_save_custom_template', { templateId: id, templateJson: JSON.stringify({ name, description, sections }) });
    if (owns()) await onSaved(id);
  });
  const { id, name, description, sections } = draft;
  return <div className="space-y-4 rounded-2xl border border-border bg-background p-4" aria-label="Custom template editor">
    <label className="block text-sm font-medium">Template ID<input disabled={busy} className={field} value={id} onChange={event => edit(current => ({ ...current, id: event.target.value }))} placeholder="user_meeting_notes" /></label>
    <label className="block text-sm font-medium">Name<input disabled={busy} className={field} value={name} onChange={event => edit(current => ({ ...current, name: event.target.value }))} /></label>
    <label className="block text-sm font-medium">Description<input disabled={busy} className={field} value={description} onChange={event => edit(current => ({ ...current, description: event.target.value }))} /></label>
    {sections.map((section, index) => <fieldset key={index} disabled={busy} className="min-w-0 space-y-3 rounded-xl border border-border bg-card p-3">
      <legend className="px-1.5 text-xs font-semibold text-muted-foreground">Section {index + 1}</legend>
      <label className="block text-sm font-medium">Title<input className={field} value={section.title} onChange={event => edit(current => ({ ...current, sections: current.sections.map(item => item === section ? { ...item, title: event.target.value } : item) }))} /></label>
      <label className="block text-sm font-medium">Instructions<textarea className={field} value={section.instruction} onChange={event => edit(current => ({ ...current, sections: current.sections.map(item => item === section ? { ...item, instruction: event.target.value } : item) }))} /></label>
      <label className="block text-sm font-medium">Format<select className="ml-2 max-w-full rounded-xl border border-input bg-card p-2 text-sm font-normal focus:outline-none focus:ring-2 focus:ring-ring" value={section.format} onChange={event => edit(current => ({ ...current, sections: current.sections.map(item => item === section ? { ...item, format: event.target.value } : item) }))}><option value="paragraph">Paragraph</option><option value="list">List</option><option value="string">Text</option></select></label>
      <Button size="sm" variant="ghost" disabled={busy || sections.length === 1} onClick={() => edit(current => current.sections.length > 1 ? { ...current, sections: current.sections.filter(item => item !== section) } : current)}>Remove section</Button>
      <Button size="sm" variant="ghost" disabled={busy || index === 0} onClick={() => edit(current => {
        const position = current.sections.indexOf(section);
        if (position <= 0) return current;
        const next = [...current.sections];
        [next[position - 1], next[position]] = [next[position], next[position - 1]];
        return { ...current, sections: next };
      })}>Move up</Button>
    </fieldset>)}
    <div className="flex flex-wrap gap-2">
      <Button size="sm" variant="outline" disabled={busy || sections.length >= 32} onClick={() => edit(current => current.sections.length < 32 ? { ...current, sections: [...current.sections, { title: '', instruction: '', format: 'paragraph' }] } : current)}>Add section</Button>
      <Button size="sm" variant="outline" disabled={busy} onClick={() => { void load(); }}>Load saved template</Button>
      <Button size="sm" disabled={busy} onClick={() => { void save(); }}>Save and use</Button>
    </div>
    <p className="break-words text-xs leading-relaxed text-muted-foreground">Preview: {sections.map(section => section.title || 'Untitled section').join(' → ')}</p>
    {error && <p role="alert" className="text-sm text-red-700">{error}</p>}
  </div>;
}
