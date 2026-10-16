import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
const originals={core:{...await import('@tauri-apps/api/core')},dynamic:{...await import('next/dynamic')},notes:{...await import('../../src/hooks/useMeetingNotes')},menus:{...await import('../../src/components/ui/dropdown-menu')},window:globalThis.window};
let calls:string[]=[],flush:()=>Promise<void>,native:(cmd:string)=>Promise<any>,inserts=0;
let ready:(value:any)=>void;
const editor={blocksToMarkdownLossy:async()=>'',getTextCursorPosition:()=>({block:{id:'block'}}),insertInlineContent(){inserts++;},document:[],focus(){}};
const session={setConverter(){},flush:()=>flush(),edit(){},getSnapshot:()=>({links:[]}),setTitle(){}};
const wrap=({children,...props}:any)=><div {...props}>{children}</div>;
mock.module('@tauri-apps/api/core',()=>({...originals.core,invoke:async(cmd:string)=>{calls.push(cmd);return native(cmd);}}));
mock.module('next/dynamic',()=>({...originals.dynamic,default:()=>({onReady}:any)=>{ready=onReady;return <div/>;}}));
mock.module('../../src/hooks/useMeetingNotes',()=>({...originals.notes,useMeetingNotes:()=>({session,snapshot:{blocks:[],title:'Fixture',status:'Saved'},error:null,reload:async()=>{}})}));
mock.module('../../src/components/ui/dropdown-menu',()=>({DropdownMenu:wrap,DropdownMenuContent:wrap,DropdownMenuLabel:wrap,DropdownMenuTrigger:wrap,DropdownMenuItem:({children,onSelect}:any)=><button onClick={onSelect}>{children}</button>}));
// Other suites import this module with a null dynamic editor. Give this harness
// its own module instance so its onReady fixture also runs in the combined suite.
const {MeetingNotesPanel}=await import('../../src/components/MeetingWorkspace/MeetingNotesPanel' + '?note-action-interaction');
let renderer:ReactTestRenderer|undefined;
const deferred=()=>{let resolve!:()=>void,reject!:(e:Error)=>void;const promise=new Promise<void>((a,b)=>{resolve=a;reject=b;});return{promise,resolve,reject};};
beforeEach(()=>{calls=[];inserts=0;flush=async()=>{};native=async()=>({is_recording:true,meeting_id:'m',recording_session_id:'r',audio_position:4,is_paused:false});Object.defineProperty(globalThis,'window',{configurable:true,value:{addEventListener(){},removeEventListener(){}}});});
afterEach(async()=>{if(renderer)await act(async()=>renderer!.unmount());renderer=undefined;});
afterAll(()=>{mock.module('@tauri-apps/api/core',()=>originals.core);mock.module('next/dynamic',()=>originals.dynamic);mock.module('../../src/hooks/useMeetingNotes',()=>originals.notes);mock.module('../../src/components/ui/dropdown-menu',()=>originals.menus);Object.defineProperty(globalThis,'window',{configurable:true,value:originals.window});});
async function show(){await act(async()=>{renderer=create(<MeetingNotesPanel meetingId="m" onCopyDraft={async()=>{}}/>);});await act(async()=>{ready(editor);});}
test('timestamp burst admits one query and insertion, then allows another timestamp',async()=>{
 const gate=deferred();native=async()=>{await gate.promise;return{is_recording:true,meeting_id:'m',recording_session_id:'r',audio_position:4,is_paused:false};};await show();
 const click=renderer!.root.findByProps({'aria-label':'Insert recording timestamp'}).props.onClick;
 await act(async()=>{for(let i=0;i<30;i++)void click();});expect(calls).toEqual(['get_recording_state']);
 await act(async()=>gate.resolve());expect(inserts).toBe(1);
 await act(async()=>click());expect(inserts).toBe(2);
});
test('original-note export burst opens one export after flush and unlocks after failure',async()=>{
 const gate=deferred();flush=()=>gate.promise;await show();
 const markdown=renderer!.root.findAllByType('button').find(node=>node.children.includes('Markdown'))!.props.onClick;
 await act(async()=>{for(let i=0;i<30;i++)markdown();});expect(calls).toEqual([]);
 native=async()=>{throw Error('Export failed');};await act(async()=>gate.resolve());expect(calls).toEqual(['api_export_meeting_notes']);
 flush=async()=>{};native=async()=>{};await act(async()=>markdown());expect(calls).toEqual(['api_export_meeting_notes','api_export_meeting_notes']);
});
