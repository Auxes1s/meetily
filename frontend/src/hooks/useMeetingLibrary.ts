'use client';
import { useCallback, useEffect, useRef, useState } from 'react';
import { meetingLibraryService } from '@/services/meetingLibraryService';
import type { LibraryPage, LibrarySearchRequest } from '@/types/meeting-library';

const message = (error: unknown) => error instanceof Error ? error.message : String(error);

export function useMeetingLibrary(request: LibrarySearchRequest) {
  const [page, setPage] = useState<LibraryPage | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const ownership = useRef(0);
  const requestKey = JSON.stringify(request);
  const refresh = useCallback(() => setRevision(value => value + 1), []);

  useEffect(() => {
    const owner = ++ownership.current;
    const input: LibrarySearchRequest = JSON.parse(requestKey);
    let live = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const owns = () => live && ownership.current === owner;
    const read = async (initial: boolean) => {
      if (initial) { setLoading(true); setError(null); setPage(null); }
      try {
        const result = input.query?.trim()
          ? await meetingLibraryService.search(input)
          : await meetingLibraryService.list(input);
        if (!owns()) return;
        setPage(result); setError(null);
        if (result.index_updating) {
          timer = setTimeout(async () => {
            try {
              await meetingLibraryService.syncIndex(25);
              if (owns()) await read(false);
            } catch (failure) {
              if (owns()) setError(`Search indexing remains pending: ${message(failure)}`);
            }
          }, 1500);
        }
      } catch (failure) {
        if (owns()) setError(message(failure));
      } finally {
        if (owns()) setLoading(false);
      }
    };
    void read(true);
    return () => { live = false; if (timer) clearTimeout(timer); };
  }, [requestKey, revision]);

  return { page, loading, error, refresh };
}
