import { afterAll, afterEach, beforeEach, expect, mock, test } from 'bun:test';
import { act, create, type ReactTestRenderer } from 'react-test-renderer';

const navigation = { ...await import('next/navigation') };
const sidebar = { ...await import('../../src/components/Sidebar/SidebarProvider') };
const recording = { ...await import('../../src/contexts/RecordingStateContext') };
const config = { ...await import('../../src/contexts/ConfigContext') };
const imports = { ...await import('../../src/contexts/ImportDialogContext') };
const save = { ...await import('../../src/lib/workspace-save') };
const events = { ...await import('@tauri-apps/api/event') };
const tooltip = { ...await import('../../src/components/ui/tooltip') };
const info = { ...await import('../../src/components/Info') };
const logo = { ...await import('../../src/components/Logo') };
const dialog = { ...await import('../../src/components/ui/dialog') };
const originalWindow = globalThis.window;
const destinations: string[] = [];
const pending: Array<() => void> = [];
let renderer: ReactTestRenderer | undefined;
const Wrapper = ({ children }: any) => <>{children}</>;
mock.module('next/navigation', () => ({ ...navigation, useRouter: () => ({ push: (path: string) => destinations.push(path) }), usePathname: () => '/' }));
mock.module('../../src/components/Sidebar/SidebarProvider', () => ({ ...sidebar, useSidebar: () => ({
  currentMeeting: null, setCurrentMeeting() {}, sidebarItems: [], isCollapsed: true, toggleCollapse() {},
  handleRecordingToggle() {}, searchTranscripts: async () => {}, searchResults: [], isSearching: false,
  meetings: [], setMeetings() {}, serverAddress: '',
}) }));
mock.module('../../src/contexts/RecordingStateContext', () => ({ ...recording, useRecordingState: () => ({ isRecording: false }) }));
mock.module('../../src/contexts/ConfigContext', () => ({ ...config, useConfig: () => ({ betaFeatures: {} }) }));
mock.module('../../src/contexts/ImportDialogContext', () => ({ ...imports, useImportDialog: () => ({ openImportDialog() {} }) }));
mock.module('../../src/lib/workspace-save', () => ({ ...save, flushWorkspaceDocuments: () => new Promise<void>(resolve => pending.push(resolve)) }));
mock.module('@tauri-apps/api/event', () => ({ ...events, listen: async () => () => {} }));
mock.module('../../src/components/ui/tooltip', () => ({ ...tooltip, Tooltip: Wrapper, TooltipProvider: Wrapper, TooltipContent: () => null, TooltipTrigger: Wrapper }));
mock.module('../../src/components/Info', () => ({ ...info, default: () => null }));
mock.module('../../src/components/Logo', () => ({ ...logo, default: () => null }));
mock.module('../../src/components/ui/dialog', () => ({ ...dialog, Dialog: ({ open, children }: any) => open ? children : null }));
const { default: Sidebar } = await import('../../src/components/Sidebar');
beforeEach(() => {
  destinations.length = 0; pending.length = 0;
  Object.defineProperty(globalThis, 'window', { configurable: true, value: { addEventListener() {}, removeEventListener() {} } });
});
afterEach(async () => { await act(async () => renderer?.unmount()); renderer = undefined; });
afterAll(() => {
  mock.module('next/navigation', () => navigation); mock.module('../../src/components/Sidebar/SidebarProvider', () => sidebar);
  mock.module('../../src/contexts/RecordingStateContext', () => recording); mock.module('../../src/contexts/ConfigContext', () => config);
  mock.module('../../src/contexts/ImportDialogContext', () => imports); mock.module('../../src/lib/workspace-save', () => save);
  mock.module('@tauri-apps/api/event', () => events); mock.module('../../src/components/ui/tooltip', () => tooltip);
  mock.module('../../src/components/Info', () => info); mock.module('../../src/components/Logo', () => logo);
  mock.module('../../src/components/ui/dialog', () => dialog);
  Object.defineProperty(globalThis, 'window', { configurable: true, value: originalWindow });
});

test('late pending library navigation cannot override a more recent Settings choice', async () => {
  await act(async () => { renderer = create(<Sidebar />); });
  await act(async () => { void renderer!.root.findByProps({ 'aria-label': 'Meeting library' }).props.onClick(); });
  await act(async () => { void renderer!.root.findByProps({ 'aria-label': 'Settings' }).props.onClick(); });
  // New navigation also flushes safely; acknowledge the latest selection first.
  await act(async () => { pending.at(-1)!(); });
  await act(async () => { pending[0](); });
  expect(destinations.at(-1)).toBe('/settings');
  expect(destinations).not.toContain('/library');
});
