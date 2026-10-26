import { useRef, useState, useEffect, useLayoutEffect, useCallback, RefObject } from 'react';
import { Virtualizer } from '@tanstack/react-virtual';
interface UseAutoScrollProps {
  scrollRef: RefObject<HTMLDivElement | null>; segments: any[]; isRecording: boolean; isPaused: boolean;
  activeSegmentId?: string; virtualizer?: Virtualizer<HTMLDivElement, Element>;
  virtualizationThreshold?: number; disableAutoScroll?: boolean;
}
/** One owner follows the actual scroll container, with no delayed scroll jobs. */
export function useAutoScroll({ scrollRef, segments, isRecording, isPaused, activeSegmentId,
  virtualizer, virtualizationThreshold = 10, disableAutoScroll = false }: UseAutoScrollProps) {
  const [autoScroll, setAutoScroll] = useState(true);
  const following = useRef(true);
  const anchor = useRef<{ id: string; offset: number } | null>(null);
  const options = useRef({ isRecording, isPaused, disableAutoScroll });
  options.current = { isRecording, isPaused, disableAutoScroll };
  const remember = useCallback(() => {
    const element = scrollRef.current; if (!element) return;
    const top = element.getBoundingClientRect().top;
    const row = Array.from(element.querySelectorAll<HTMLElement>('[data-passage-id]')).find(row => row.getBoundingClientRect().bottom > top);
    anchor.current = row ? { id: row.dataset.passageId!, offset: row.getBoundingClientRect().top - top } : null;
  }, [scrollRef]);
  const setFollowing = useCallback((value: boolean) => { following.current = value; setAutoScroll(value); }, []);
  const scrollToBottom = useCallback(() => {
    const element = scrollRef.current; if (!element) return;
    setFollowing(true); element.scrollTop = element.scrollHeight; remember();
  }, [scrollRef, setFollowing, remember]);
  const adjust = useCallback(() => {
    const element = scrollRef.current; if (!element || options.current.disableAutoScroll) return;
    if (following.current && options.current.isRecording && !options.current.isPaused) element.scrollTop = element.scrollHeight;
    else if (anchor.current) {
      const saved = anchor.current;
      const row = Array.from(element.querySelectorAll<HTMLElement>('[data-passage-id]')).find(row => row.dataset.passageId === saved.id);
      if (row) element.scrollTop += row.getBoundingClientRect().top - element.getBoundingClientRect().top - saved.offset;
    }
    remember();
  }, [scrollRef, remember]);
  useEffect(() => {
    const element = scrollRef.current; if (!element) return;
    let frame = 0;
    const schedule = () => { if (!frame) frame = requestAnimationFrame(() => { frame = 0; adjust(); }); };
    const onScroll = () => {
      const near = element.scrollHeight - element.scrollTop - element.clientHeight <= 100;
      setFollowing(near); remember();
    };
    element.addEventListener('scroll', onScroll, { passive: true });
    const resize = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(schedule);
    resize?.observe(element);
    const observeContent = () => { for (const child of Array.from(element.children)) resize?.observe(child); };
    observeContent();
    const mutations = typeof MutationObserver === 'undefined' ? null : new MutationObserver(() => { observeContent(); schedule(); });
    mutations?.observe(element, { subtree: true, childList: true, characterData: true });
    remember();
    return () => { element.removeEventListener('scroll', onScroll); resize?.disconnect(); mutations?.disconnect(); if (frame) cancelAnimationFrame(frame); };
  }, [scrollRef, adjust, remember, setFollowing]);
  useLayoutEffect(adjust, [segments, isRecording, isPaused, adjust]);
  useLayoutEffect(() => {
    if (!activeSegmentId) return;
    setFollowing(false);
    if (virtualizer && segments.length >= virtualizationThreshold) {
      const index = segments.findIndex(segment => segment.id === activeSegmentId);
      if (index >= 0) virtualizer.scrollToIndex(index, { align: 'center', behavior: 'auto' });
    } else document.getElementById(`segment-${activeSegmentId}`)?.scrollIntoView({ behavior: 'auto', block: 'center' });
    remember();
  }, [activeSegmentId, virtualizer, segments, virtualizationThreshold, remember, setFollowing]);
  return { autoScroll, setAutoScroll: setFollowing, scrollToBottom };
}
