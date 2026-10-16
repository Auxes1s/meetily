import { afterAll, afterEach, beforeEach, expect, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import { useAutoScroll } from '../../src/hooks/useAutoScroll';
const original = { raf: globalThis.requestAnimationFrame, cancel: globalThis.cancelAnimationFrame, resize: globalThis.ResizeObserver, mutation: globalThis.MutationObserver };
let frames = new Map<number, FrameRequestCallback>(), serial=0, changed: () => void, renderer: ReactTestRenderer | undefined;
let rowOffset=200;
const handlers = new Map<string, () => void>();
const row = { dataset: { passageId: 'reading' }, getBoundingClientRect: () => ({top:rowOffset-element.scrollTop,bottom:rowOffset-element.scrollTop+100}) };
const element = { scrollTop:200,scrollHeight:400,clientHeight:200,children:[],
  getBoundingClientRect:()=>({top:0}),querySelectorAll:()=>[row],
  addEventListener:(name:string, fn:()=>void)=>handlers.set(name,fn),removeEventListener:(name:string)=>handlers.delete(name) };
const ref={current:element as unknown as HTMLDivElement};
function View({revision=0}:{revision?:number}) { useAutoScroll({scrollRef:ref,segments:[{id:`revision-${revision}`}],isRecording:true,isPaused:false});return null; }
async function paint() { await act(async()=>{const pending=[...frames.values()];frames.clear();pending.forEach(fn=>fn(0));}); }
beforeEach(()=>{
  rowOffset=200;element.scrollTop=200;element.scrollHeight=400;frames.clear();handlers.clear();
  globalThis.requestAnimationFrame=fn=>{frames.set(++serial,fn);return serial;};globalThis.cancelAnimationFrame=id=>{frames.delete(id);};
  globalThis.ResizeObserver=class {constructor(fn:()=>void){changed=fn;}observe(){}disconnect(){}} as any;
  globalThis.MutationObserver=class {constructor(fn:()=>void){changed=fn;}observe(){}disconnect(){}} as any;
});
afterEach(async()=>{if(renderer)await act(async()=>renderer!.unmount());renderer=undefined;});
afterAll(()=>{globalThis.requestAnimationFrame=original.raf;globalThis.cancelAnimationFrame=original.cancel;globalThis.ResizeObserver=original.resize;globalThis.MutationObserver=original.mutation;});
test('follows a large content update using prior intent, including partial-only DOM changes',async()=>{
  await act(async()=>{renderer=create(<View/>);});
  element.scrollHeight=1000;changed();await paint();expect(element.scrollTop).toBe(1000);
  element.scrollHeight=1200;changed();await paint();expect(element.scrollTop).toBe(1200);
});
test('manual reading remains anchored through earlier insertions and cancels deferred work on unmount',async()=>{
  await act(async()=>{renderer=create(<View/>);});
  element.scrollTop=50;rowOffset=50;handlers.get('scroll')!();
  rowOffset=200;element.scrollHeight=1000;
  await act(async()=>renderer!.update(<View revision={1}/>));expect(element.scrollTop).toBe(200);
  changed();expect(frames.size).toBe(1);await act(async()=>renderer!.unmount());renderer=undefined;expect(frames.size).toBe(0);
});
