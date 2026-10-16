import { useEffect, useState } from 'react';
import { transcriptService, type TranscriptionStatus } from '@/services/transcriptService';
export function usePipelineHealth() {
  const [health, setHealth] = useState<TranscriptionStatus | null>(null);
  useEffect(() => {
    let disposed = false, pending = false;
    const poll = async () => {
      if (pending) return; pending = true;
      try {
        const value = await transcriptService.getTranscriptionStatus();
        const expected = sessionStorage.getItem('recording_session_id');
        if (!disposed && value && (!value.recording_session_id || value.recording_session_id === expected)) setHealth(value);
      } catch { if (!disposed) setHealth(null); }
      finally { pending = false; }
    };
    void poll(); const timer = setInterval(() => { void poll(); }, 500);
    return () => { disposed = true; clearInterval(timer); };
  }, []);
  return health;
}
