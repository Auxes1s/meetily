import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
const originals = {
  core: { ...await import('@tauri-apps/api/core') }, event: { ...await import('@tauri-apps/api/event') },
  navigation: { ...await import('next/navigation') }, transcript: { ...await import('../../src/contexts/TranscriptContext') },
  sidebar: { ...await import('../../src/components/Sidebar/SidebarProvider') }, state: { ...await import('../../src/contexts/RecordingStateContext') },
  service: { ...await import('../../src/services/recordingService') }, notes: { ...await import('../../src/lib/meeting-notes-session') },
  language: { ...await import('../../src/lib/summary-language-preferences') },
  window: globalThis.window, storage: globalThis.sessionStorage,
};
const deferred=()=>{let resolve!:(value?:any)=>void;const promise=new Promise<any>(r=>{resolve=r;});return {promise,resolve};};
let native:()=>Promise<any>, cache:()=>Promise<void>, refresh:()=>Promise<void>, renderer:ReactTestRenderer|undefined;
let segments:any[],statuses:string[], cleared:number, marked:number, navigated:string[];
mock.module('@tauri-apps/api/core',()=>({...originals.core,invoke:async()=>({status:'completed',recording_session_id:'session-A'})}));
mock.module('@tauri-apps/api/event',()=>({...originals.event,listen:async()=>()=>{}}));
mock.module('next/navigation',()=>({...originals.navigation,useRouter:()=>({push:(url:string)=>navigated.push(url)})}));
mock.module('../../src/contexts/TranscriptContext',()=>({...originals.transcript,useTranscriptActions:()=>({
  transcriptsRef:{get current(){return segments;}},flushBuffer(){},clearTranscripts(){cleared++;segments=[];},meetingTitle:'A',markMeetingAsSaved:()=>{marked++;return cache();},
})}));
mock.module('../../src/components/Sidebar/SidebarProvider',()=>({...originals.sidebar,useSidebar:()=>({refetchMeetings:()=>refresh(),setCurrentMeeting(){},setIsMeetingActive(){}})}));
mock.module('../../src/contexts/RecordingStateContext',()=>({...originals.state,useRecordingState:()=>({setStatus:(status:string)=>statuses.push(status)})}));
mock.module('../../src/services/recordingService',()=>({...originals.service,recordingService:{...originals.service.recordingService,getRecordingState:()=>native()}}));
mock.module('../../src/lib/meeting-notes-session',()=>({...originals.notes,flushAllNotes:async()=>{}}));
mock.module('../../src/lib/summary-language-preferences',()=>({...originals.language,applyPinnedSummaryLanguageToMeeting:async()=>true}));
const {useRecordingStop}=await import('../../src/hooks/useRecordingStop');
let stop:ReturnType<typeof useRecordingStop>;
function View(){stop=useRecordingStop(()=>{},()=>{});return null;}
async function show(){await act(async()=>{renderer=create(<View/>);});}
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
function session(id:string){sessionStorage.setItem('recording_session_id',`session-${id}`);sessionStorage.setItem('recording_meeting_id',`meeting-${id}`);}
beforeEach(()=>{
  const values=new Map<string,string>();
  Object.defineProperty(globalThis,'sessionStorage',{configurable:true,value:{getItem:(key:string)=>values.get(key)??null,setItem:(key:string,value:string)=>values.set(key,value),removeItem:(key:string)=>values.delete(key)}});
  Object.defineProperty(globalThis,'window',{configurable:true,value:{dispatchEvent(){}}});
  session('A');native=async()=>({is_recording:false});cache=async()=>{};refresh=async()=>{};statuses=[];cleared=0;marked=0;navigated=[];
  segments=[{id:'A',text:'A',is_partial:false,recording_session_id:'session-A'}];
});
afterEach(async()=>{if(renderer)await act(async()=>renderer!.unmount());renderer=undefined;});
afterAll(()=>{
  mock.module('@tauri-apps/api/core',()=>originals.core);mock.module('@tauri-apps/api/event',()=>originals.event);mock.module('next/navigation',()=>originals.navigation);
  mock.module('../../src/contexts/TranscriptContext',()=>originals.transcript);mock.module('../../src/components/Sidebar/SidebarProvider',()=>originals.sidebar);mock.module('../../src/contexts/RecordingStateContext',()=>originals.state);
  mock.module('../../src/services/recordingService',()=>originals.service);mock.module('../../src/lib/meeting-notes-session',()=>originals.notes);mock.module('../../src/lib/summary-language-preferences',()=>originals.language);
  Object.defineProperty(globalThis,'window',{configurable:true,value:originals.window});Object.defineProperty(globalThis,'sessionStorage',{configurable:true,value:originals.storage});
});
test('a delayed native read for A cannot change newly started B',async()=>{
  const gate=deferred();native=()=>gate.promise;await show();let pending!:Promise<void>;
  await act(async()=>{pending=stop.handleRecordingStop(true,'session-A');session('B');gate.resolve({is_recording:true,recording_session_id:'session-B'});await pending;});
  expect(statuses).toEqual([]);expect(cleared).toBe(0);expect(marked).toBe(0);expect(sessionStorage.getItem('recording_session_id')).toBe('session-B');
});
test('a delayed recovery acknowledgement for A cannot clear B',async()=>{
  const gate=deferred();cache=()=>gate.promise;await show();let pending!:Promise<void>;
  await act(async()=>{pending=stop.handleRecordingStop(true,'session-A');await tick();session('B');gate.resolve();await pending;});
  expect(marked).toBe(1);expect(cleared).toBe(0);expect(statuses).toEqual(['saving']);expect(sessionStorage.getItem('recording_session_id')).toBe('session-B');
});
test('an old sidebar refresh cannot clear B or navigate away after A cleanup',async()=>{
  const gate=deferred();refresh=()=>gate.promise;await show();let pending!:Promise<void>;
  await act(async()=>{pending=stop.handleRecordingStop(true,'session-A');await tick();session('B');segments=[{id:'B',text:'New live speech'}];gate.resolve();await pending;});
  expect(cleared).toBe(1);expect(segments[0].id).toBe('B');expect(statuses).toEqual(['saving']);expect(navigated).toEqual([]);expect(sessionStorage.getItem('recording_session_id')).toBe('session-B');
});

test('an unsuccessful Stop preserves the active capture and its recovery data',async()=>{
  native=async()=>({is_recording:true,recording_session_id:'session-A'});await show();
  await act(async()=>{await stop.handleRecordingStop(true,'session-A');});
  expect(statuses).toEqual([]);expect(cleared).toBe(0);expect(marked).toBe(0);expect(sessionStorage.getItem('recording_session_id')).toBe('session-A');
});
