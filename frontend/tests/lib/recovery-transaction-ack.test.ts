import { afterAll, beforeEach, expect, test } from 'bun:test';
const originalWindow = globalThis.window;
beforeEach(() => Object.defineProperty(globalThis, 'window', { configurable: true, value: { localStorage: { getItem: () => null } } }));
afterAll(() => Object.defineProperty(globalThis, 'window', { configurable: true, value: originalWindow }));
import { indexedDBService } from '../../src/services/indexedDBService';
const tick=()=>new Promise(resolve=>setTimeout(resolve,0));
function fixture() {
  const service = new (indexedDBService.constructor as any)();
  const transaction:any={error:null};
  const request=(result:any)=>{const value:any={result};queueMicrotask(()=>value.onsuccess?.());return value;};
  transaction.objectStore=(name:string)=>({get:()=>request(name==='meetings'?{meetingId:'cache',transcriptCount:0}:undefined),put:()=>request(undefined)});
  service.db={transaction:()=>transaction};return {service,transaction};
}
test('transcript recovery waits for transaction commit after request success',async()=>{
  const {service,transaction}=fixture();let acknowledged=false;
  const saved=service.saveTranscript('cache',{id:'row',sequence_id:0,is_partial:false,text:'Final',audio_source:'system'}).then(()=>{acknowledged=true;});
  await tick();expect(acknowledged).toBe(false);transaction.oncomplete();await saved;expect(acknowledged).toBe(true);
});
test('transaction abort rejects recovery acknowledgement after successful put',async()=>{
  const {service,transaction}=fixture();
  const saved=service.saveTranscript('cache',{id:'row',sequence_id:0,is_partial:false,text:'Final'});
  await tick();transaction.onabort();
  await expect(saved).rejects.toThrow('Transcript recovery transaction aborted');
});
test('saved-to-SQLite marker waits for its metadata transaction to commit',async()=>{
  const {service,transaction}=fixture();let acknowledged=false;
  const saved=service.markMeetingSaved('cache').then(()=>{acknowledged=true;});
  await tick();expect(acknowledged).toBe(false);transaction.oncomplete();await saved;expect(acknowledged).toBe(true);
});
