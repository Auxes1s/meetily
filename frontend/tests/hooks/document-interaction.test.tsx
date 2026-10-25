import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import { lazy, Suspense } from 'react';
const originals = {
 dynamic: { ...await import('next/dynamic') },
 navigation: { ...await import('next/navigation') }, notes: { ...await import('../../src/lib/meeting-notes-session') },
 enhancements: { ...await import('../../src/lib/enhancement-session') }, panel: { ...await import('../../src/components/MeetingWorkspace/MeetingNotesPanel') },
 enhancedPanel: { ...await import('../../src/components/MeetingWorkspace/EnhancementPanel') }, exports: { ...await import('../../src/components/MeetingWorkspace/MeetingExportButton') }, window: globalThis.window,
};
let flush: () => Promise<void>, calls=0, events: Map<string, (event:any) => void>;
mock.module('next/dynamic', () => ({ default: (loader: () => Promise<any>) => {
 const Component = lazy(async () => { const loaded = await loader(); return { default: loaded.default ?? loaded }; });
 return (props: any) => <Suspense fallback={null}><Component {...props} /></Suspense>;
} }));
mock.module('next/navigation', () => ({ ...originals.navigation, useSearchParams: () => new URLSearchParams() }));
mock.module('../../src/lib/meeting-notes-session', () => ({ ...originals.notes, noteSessions: { get: () => ({ flush: () => { calls++; return flush(); } }) } }));
mock.module('../../src/lib/enhancement-session', () => ({ ...originals.enhancements, flushAllEnhancements: async () => {} }));
mock.module('../../src/components/MeetingWorkspace/MeetingNotesPanel', () => ({ MeetingNotesPanel: () => <textarea aria-label="Original editor" defaultValue="Retain original" /> }));
mock.module('../../src/components/MeetingWorkspace/EnhancementPanel', () => ({ EnhancementPanel: () => <textarea aria-label="Enhanced editor" defaultValue="Retain enhanced" /> }));
mock.module('../../src/components/MeetingWorkspace/MeetingExportButton', () => ({ MeetingExportButton: () => null }));
const { MeetingDocuments } = await import('../../src/components/MeetingWorkspace/MeetingDocuments');
let renderer:ReactTestRenderer|undefined;
const tab=(label:string) => renderer!.root.findAllByType('button').find(node => node.children.join('')===label)!;
beforeEach(() => { calls=0;flush=async () => {};events=new Map();Object.defineProperty(globalThis,'window',{ configurable:true,value:{addEventListener:(name:string,cb:any)=>events.set(name,cb),removeEventListener:(name:string)=>events.delete(name)}}); });
afterEach(async () => { if(renderer) await act(async () => renderer!.unmount());renderer=undefined; });
afterAll(() => {
 mock.module('next/dynamic', () => originals.dynamic);
 mock.module('next/navigation', () => originals.navigation);mock.module('../../src/lib/meeting-notes-session', () => originals.notes);mock.module('../../src/lib/enhancement-session', () => originals.enhancements);
 mock.module('../../src/components/MeetingWorkspace/MeetingNotesPanel', () => originals.panel);mock.module('../../src/components/MeetingWorkspace/EnhancementPanel', () => originals.enhancedPanel);mock.module('../../src/components/MeetingWorkspace/MeetingExportButton', () => originals.exports);
 Object.defineProperty(globalThis,'window',{configurable:true,value:originals.window});
});
async function show() { await act(async () => { renderer=create(<MeetingDocuments meetingId="m" onCopyDraft={async()=>{}} />); }); }
test('rapid tab switching flushes once, preserves both editor instances, and retries after failure',async()=>{
 let reject!:(e:Error)=>void;flush=()=>new Promise((_,b)=>{reject=b;});await show();
 const original=renderer!.root.findByProps({'aria-label':'Original editor'});
 expect(renderer!.root.findAllByProps({'aria-label':'Enhanced editor'})).toHaveLength(0);
 const click=tab('Enhanced notes').props.onClick;
 await act(async()=>{for(let i=0;i<30;i++)click();});expect(calls).toBe(1);
 await act(async()=>reject(new Error('Save unavailable')));expect(tab('My notes').props['aria-selected']).toBe(true);
 expect(renderer!.root.findByProps({role:'alert'}).children.join('')).toContain('Save unavailable');flush=async()=>{};
 await act(async()=>tab('Enhanced notes').props.onClick());expect(calls).toBe(2);expect(tab('Enhanced notes').props['aria-selected']).toBe(true);
 const enhanced=renderer!.root.findByProps({'aria-label':'Enhanced editor'});
 await act(async()=>tab('My notes').props.onClick());await act(async()=>tab('Enhanced notes').props.onClick());
 expect(renderer!.root.findByProps({'aria-label':'Original editor'})).toBe(original);expect(renderer!.root.findByProps({'aria-label':'Enhanced editor'})).toBe(enhanced);
});
test('source navigation supersedes a pending tab switch',async()=>{
 let finish!:()=>void;flush=()=>new Promise(a=>{finish=a;});await show();
 await act(async()=>tab('Enhanced notes').props.onClick());
 await act(async()=>events.get('workspace-note-source')!({detail:{meetingId:'m'}}));
 await act(async()=>finish());expect(tab('My notes').props['aria-selected']).toBe(true);
});
test('tab arrow navigation selects after save and keeps accessible panel links',async()=>{
 await show();let focused='';
 await act(async()=>tab('My notes').props.onKeyDown({key:'ArrowRight',preventDefault(){},currentTarget:{parentElement:{querySelectorAll:()=>[{focus(){focused='original';}},{focus(){focused='enhanced';}}]}}}));
 expect(focused).toBe('enhanced');expect(tab('Enhanced notes').props['aria-selected']).toBe(true);
 expect(tab('Enhanced notes').props.tabIndex).toBe(0);expect(tab('My notes').props.tabIndex).toBe(-1);
 expect(renderer!.root.findByProps({id:'enhanced-notes'}).props['aria-labelledby']).toBe(tab('Enhanced notes').props.id);
});
