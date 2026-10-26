"use client";

import { useEffect } from "react";
import type { PartialBlock, Block } from "@blocknote/core";
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import type { BlockNoteEditor } from '@blocknote/core';
import "@blocknote/shadcn/style.css";

interface EditorProps {
  initialContent?: Block[];
  onChange?: (blocks: Block[]) => void;
  editable?: boolean;
  onReady?: (editor: BlockNoteEditor) => void | (() => void);
  onMarkerClick?: (id: string) => void;
}

export default function Editor({ initialContent, onChange, editable = true, onReady, onMarkerClick }: EditorProps) {
  const editor = useCreateBlockNote({
    initialContent: initialContent?.length ? initialContent as PartialBlock[] : undefined,
  });
  useEffect(() => onReady?.(editor), [editor, onReady]);

  // Handle content changes
  useEffect(() => {
    if (!onChange) return;

    const handleChange = () => {
      onChange(editor.document);
    };

    const unsubscribe = editor.onChange(handleChange);

    return () => {
      if (typeof unsubscribe === 'function') {
        unsubscribe();
      }
    };
  }, [editor, onChange]);

  return <div className="note-editor-shell min-w-0 max-w-full" onClickCapture={event => {
    const anchor = (event.target as HTMLElement).closest('a[href^="meetily-note://"]');
    if (anchor) {
      event.preventDefault();
      event.stopPropagation();
      onMarkerClick?.(anchor.getAttribute('href')!.slice('meetily-note://'.length));
    }
  }}><BlockNoteView editor={editor} editable={editable} theme="light" /></div>;
}
