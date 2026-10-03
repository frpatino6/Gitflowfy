import { createSignal } from 'solid-js';

export function useShortcuts() {
  const [shortcuts, setShortcuts] = createSignal<Record<string, string>>({
    'ctrl+s': 'commit',
    'ctrl+p': 'push',
    'ctrl+shift+p': 'pull',
    'ctrl+b': 'branch',
    'ctrl+shift+b': 'branch:new',
    'ctrl+m': 'merge',
    'ctrl+r': 'rebase',
    'ctrl+t': 'stash',
    'ctrl+shift+f': 'fetch',
    'ctrl+g': 'graph:focus',
    'ctrl+1': 'sidebar:repos',
    'ctrl+2': 'sidebar:branches',
    'ctrl+3': 'sidebar:tags',
    'ctrl+4': 'sidebar:remotes',
    'ctrl+5': 'sidebar:stashes',
    'f11': 'fullscreen',
    'ctrl+,': 'settings',
  });

  const setShortcut = (action: string, shortcut: string) => {
    setShortcuts(prev => ({ ...prev, [shortcut]: action }));
  };

  const getShortcut = (action: string) => {
    const entries = Object.entries(shortcuts());
    const found = entries.find(([, a]) => a === action);
    return found ? found[0] : null;
  };

  return {
    shortcuts,
    setShortcut,
    getShortcut,
  };
}