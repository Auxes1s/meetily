// Browser regression fixture; bundle with `bun build --target browser` and load
// over localhost in a disposable browser profile. The host needs <pre id="result">.
import { EditorView } from '@tiptap/pm/view';
import { EditorState } from '@tiptap/pm/state';
import { Schema } from '@tiptap/pm/model';
import { BlockNoteEditor } from '@blocknote/core';

const results = [];
function assert(condition, message) { if (!condition) throw new Error(message); }
function paste(context) {
  let checks = 0, slice;
  const schema = new Schema({ nodes: {
    doc: { content: 'block+' },
    paragraph: { group: 'block', content: 'inline*', parseDOM: [{ tag: 'p' }], toDOM: () => ['p', 0] },
    blockquote: {
      group: 'block', content: 'block+',
      attrs: { label: { default: 'safe', validate(value) { checks++; if (typeof value !== 'string') throw new Error('Invalid label'); } } },
      parseDOM: [{ tag: 'blockquote' }], toDOM: node => ['blockquote', { 'data-label': node.attrs.label }, 0],
    },
    text: { group: 'inline' },
  } });
  const target = document.body.appendChild(document.createElement('div'));
  const view = new EditorView(target, { state: EditorState.create({ schema }), handlePaste(_view, _event, value) { slice = value.toJSON(); return true; } });
  try {
    checks = 0;
    view.pasteHTML(`<p data-pm-slice='1 1 ${JSON.stringify(context)}'>payload</p>`);
    return { checks, slice };
  } finally { view.destroy(); target.remove(); }
}
async function run() {
  const valid = paste(['blockquote', { label: 'safe' }]);
  assert(valid.checks > 0 && valid.slice.content[0].type === 'blockquote', 'Valid context paste changed');
  for (const context of [
    ['blockquote', { label: ['invalid'] }],
    ['blockquote', { label: 'safe', unknown: 'invalid' }],
    ['blockquote', { label: 'safe' }, 'blockquote', { label: ['invalid'] }],
  ]) {
    const result = paste(context);
    assert(result.slice.content[0].type === 'paragraph', 'Invalid clipboard context survived');
  }
  results.push('valid, invalid, unknown and nested clipboard attributes');
  const element = document.body.appendChild(document.createElement('div'));
  const editor = BlockNoteEditor.create({ initialContent: [{ type: 'paragraph', content: 'Start' }] });
  editor.mount(element);
  try {
    editor.setTextCursorPosition(editor.document[0].id, 'end');
    editor.pasteHTML('<p>Hello <strong>world</strong> <a href="https://example.com">link</a></p>');
    editor.pasteText(' plain 日本語');
    const pasted = JSON.stringify(editor.document);
    assert(['Hello', 'world', 'example.com', '日本語'].every(text => pasted.includes(text)), 'Normal paste lost content');
    editor.insertInlineContent([{ type: 'link', href: 'meetily-note://fixture-marker', content: '[00:12]' }]);
    const withMarker = JSON.stringify(editor.document);
    assert(withMarker.includes('meetily-note://fixture-marker'), 'Timestamp link lost');
    editor.undo(); assert(JSON.stringify(editor.document) !== withMarker, 'Undo failed');
    editor.redo(); assert(JSON.stringify(editor.document) === withMarker, 'Redo failed');
    const markdown = await editor.blocksToMarkdownLossy(editor.document);
    assert(markdown.includes('Hello') && markdown.includes('日本語'), 'Markdown conversion failed');
    const reloaded = BlockNoteEditor.create({ initialContent: JSON.parse(withMarker) });
    assert(JSON.stringify(reloaded.document) === withMarker, 'Document reload changed JSON');
    results.push('BlockNote HTML, text, Unicode, links, timestamps, undo, redo, Markdown and reload');
  } finally { editor.mount(undefined); element.remove(); }
}
run().then(() => { document.getElementById('result').textContent = JSON.stringify({ ok: true, results }); })
  .catch(error => { document.getElementById('result').textContent = JSON.stringify({ ok: false, error: String(error), results }); });
