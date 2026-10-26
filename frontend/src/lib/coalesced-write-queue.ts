/** Serial durable writes; superseded values share the latest commit acknowledgement. */
export class CoalescedWriteQueue<T> {
  private pending = new Map<string, { value: T; waiters: { resolve(): void; reject(error: unknown): void }[] }>();
  private running: Promise<void> | null = null;
  constructor(private write: (value: T) => Promise<void>, private merge: (old: T, incoming: T) => T) {}
  enqueue(key: string, value: T): Promise<void> {
    const existing = this.pending.get(key);
    const item = existing ?? { value, waiters: [] };
    if (existing) item.value = this.merge(item.value, value);
    this.pending.set(key, item);
    const result = new Promise<void>((resolve, reject) => item.waiters.push({ resolve, reject }));
    this.start(); return result;
  }
  private start() {
    if (this.running) return;
    let failed = false;
    this.running = Promise.resolve().then(async () => {
      while (this.pending.size) {
        const [key, item] = this.pending.entries().next().value!;
        this.pending.delete(key);
        try { await this.write(item.value); item.waiters.forEach(waiter => waiter.resolve()); }
        catch (error) {
          failed = true;
          item.waiters.forEach(waiter => waiter.reject(error));
          const next = this.pending.get(key);
          this.pending.set(key, { value: next ? this.merge(item.value, next.value) : item.value, waiters: next?.waiters ?? [] });
          throw error;
        }
      }
    }).finally(() => { this.running = null; if (!failed && this.pending.size) this.start(); });
    // enqueue callers receive their individual errors; flush can retry retained work.
    void this.running.catch(() => {});
  }
  async flush(): Promise<void> { do { this.start(); await this.running; } while (this.pending.size); }
}
