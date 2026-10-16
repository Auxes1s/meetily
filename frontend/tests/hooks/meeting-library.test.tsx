import { afterAll, describe, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { LibraryPage, LibrarySearchRequest } from '../../src/types/meeting-library';

const originalService = { ...await import('../../src/services/meetingLibraryService') };
const replies = new Map<string, (page: LibraryPage) => void>();
mock.module('../../src/services/meetingLibraryService', () => ({
  meetingLibraryService: {
    search: (request: LibrarySearchRequest) => new Promise<LibraryPage>(resolve => replies.set(request.query!, resolve)),
    list: async () => ({ meetings: [], total_count: 0, next_cursor: null, index_updating: false, pending_index_count: 0 }),
  },
}));
afterAll(() => mock.module('../../src/services/meetingLibraryService', () => originalService));
const { useMeetingLibrary } = await import('../../src/hooks/useMeetingLibrary');
function View({ query }: { query: string }) {
  const { page, loading } = useMeetingLibrary({ query });
  return <output>{loading ? 'loading' : page?.total_count}</output>;
}
const page = (count: number): LibraryPage => ({ meetings: [], total_count: count, next_cursor: null, index_updating: false, pending_index_count: 0 });

describe('meeting library request ownership', () => {
  test('a slow earlier query cannot replace results of the current query', async () => {
    let renderer!: ReactTestRenderer;
    try {
      await act(async () => { renderer = create(<View query="first" />); });
      await act(async () => { renderer.update(<View query="second" />); });
      await act(async () => { replies.get('second')!(page(2)); });
      expect(renderer.root.findByType('output').children).toEqual(['2']);
      await act(async () => { replies.get('first')!(page(1)); });
      expect(renderer.root.findByType('output').children).toEqual(['2']);
    } finally {
      await act(async () => renderer?.unmount());
    }
  });
});
