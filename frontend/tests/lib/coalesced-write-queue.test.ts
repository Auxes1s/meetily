import { expect, test } from 'bun:test';
import { CoalescedWriteQueue } from '../../src/lib/coalesced-write-queue';
const deferred = () => { let resolve!: () => void; const promise = new Promise<void>(r => { resolve = r; }); return { promise, resolve }; };
test('partial bursts coalesce and final acknowledgements wait for the serial commit', async () => {
  const gate = deferred(); const writes: number[] = []; let active = 0, peak = 0;
  const queue = new CoalescedWriteQueue<number>(async n => { active++; peak=Math.max(peak,active); writes.push(n); await gate.promise; active--; }, Math.max);
  const first = queue.enqueue('passage',1); await Promise.resolve();
  const revisions = Array.from({length:100},(_,i)=>queue.enqueue('passage',i+2));
  let acknowledged=false; void revisions.at(-1)!.then(()=>{acknowledged=true;});
  await Promise.resolve();expect(acknowledged).toBe(false);gate.resolve();
  await Promise.all([first,...revisions]);await queue.flush();expect(writes).toEqual([1,101]);expect(peak).toBe(1);
});
test('failed writes reject acknowledgements and remain available for an explicit retry', async () => {
  let failed=true;const writes:number[]=[];
  const queue=new CoalescedWriteQueue<number>(async n=>{if(failed)throw Error('disk failure');writes.push(n);},Math.max);
  await expect(queue.enqueue('passage',4)).rejects.toThrow('disk failure');
  await Promise.resolve();await Promise.resolve();failed=false;await queue.flush();expect(writes).toEqual([4]);
});
