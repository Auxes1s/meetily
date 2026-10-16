import { afterAll, afterEach, describe, expect, mock, test } from 'bun:test';
import { createRef } from 'react';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';
import type { BlockNoteSummaryViewRef } from '../../src/components/AISummary/BlockNoteSummaryView';

const originals = {
  dynamic: { ...await import('next/dynamic') },
  blocknote: { ...await import('@blocknote/react') },
  view: { ...await import('@blocknote/shadcn') },
  markdown: { ...await import('../../src/lib/blocknote-markdown') },
  summary: { ...await import('../../src/components/AISummary/index') },
};
afterAll(() => {
  mock.module('next/dynamic', () => originals.dynamic);
  mock.module('@blocknote/react', () => originals.blocknote);
  mock.module('@blocknote/shadcn', () => originals.view);
  mock.module('../../src/lib/blocknote-markdown', () => originals.markdown);
  mock.module('../../src/components/AISummary/index', () => originals.summary);
});
let editorChange!: (blocks: unknown[]) => void;
let renderedLegacy: unknown;
mock.module('next/dynamic', () => ({ default: () => (props: any) => { editorChange = props.onChange; return <div />; } }));
mock.module('@blocknote/react', () => ({ useCreateBlockNote: () => ({ document: [], replaceBlocks() {} }) }));
mock.module('@blocknote/shadcn', () => ({ BlockNoteView: () => <div /> }));
mock.module('../../src/lib/blocknote-markdown', () => ({ blocksToMarkdownSafely: async (_editor: unknown, blocks: unknown[]) => ({ markdown: JSON.stringify(blocks) }) }));
mock.module('../../src/components/AISummary/index', () => ({ AISummary: (props: any) => { renderedLegacy = props.summary; return <div />; } }));
const { BlockNoteSummaryView } = await import('../../src/components/AISummary/BlockNoteSummaryView');
let renderer: ReactTestRenderer | undefined;
afterEach(async () => { if (renderer) await act(async () => renderer!.unmount()); renderer = undefined; });
const blocks = (text: string) => [{ id: 'b', type: 'paragraph', content: [{ type: 'text', text }], children: [] }];

describe('manual summary save acknowledgement', () => {
  test('typing during an in-flight save stays dirty and the next save contains the newer edits', async () => {
    let finish!: () => void;
    const firstAck = new Promise<void>(resolve => { finish = resolve; });
    const calls: any[] = [];
    const onSave = async (value: unknown) => { calls.push(structuredClone(value)); if (calls.length === 1) await firstAck; };
    const ref = createRef<BlockNoteSummaryViewRef>();
    await act(async () => { renderer = create(<BlockNoteSummaryView ref={ref} summaryData={{ summary_json: blocks('Original') } as any} onSave={onSave} />); });
    await act(async () => { editorChange(blocks('Edit A')); });
    let save!: Promise<void>;
    await act(async () => { save = ref.current!.saveSummary(); await Promise.resolve(); });
    await act(async () => { editorChange(blocks('Edit B')); finish(); await save; });
    expect(ref.current!.isDirty).toBe(true);
    expect(calls[0].summary_json).toEqual(blocks('Edit A'));
    await act(async () => { await ref.current!.saveSummary(); });
    expect(calls[1].summary_json).toEqual(blocks('Edit B'));
    expect(ref.current!.isDirty).toBe(false);
  });
  test('legacy rendering excludes revision metadata and unknown extension fields', async () => {
    await act(async () => { renderer = create(<BlockNoteSummaryView summaryData={{
      Agenda: { title: 'Agenda', blocks: [{ id: 'Agenda-1', content: 'Proposal' }] },
      _enhancement_version_id: 'legacy-a', _enhancement_revision: 3, extension: { private: 17 },
    } as any} />); });
    expect(Object.keys(renderedLegacy as object)).toEqual(['Agenda']);
  });
});
