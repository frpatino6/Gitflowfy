import { createSignal, createMemo } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  repoPath: string | null;
}

export default function SidebarRight(props: Props) {
  const [activeTab, setActiveTab] = createSignal<'diff' | 'commit' | 'files'>('diff');

  const tabs = [
    { id: 'diff', label: 'Diff', icon: 'diff' },
    { id: 'commit', label: 'Commit', icon: 'commit' },
    { id: 'files', label: 'Files', icon: 'file' },
  ];

  return (
    <div class="sidebar-right">
      <div class="sidebar-tabs">
        {[
          { id: 'diff', label: 'Diff' },
          { id: 'commit', label: 'Commit' },
          { id: 'files', label: 'Files' },
          { id: 'history', label: 'History' },
        ].map(tab => (
          <button
            class={`sidebar-tab ${activeTab() === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id as any)}
            title={tab.label}
          >
            {tab.label}
          </button>
        ))}
      </div>
      
      <div class="sidebar-content">
        <Show when={activeTab() === 'diff'}>
          <DiffViewer />
        </Show>
        <Show when={activeTab() === 'commit'}>
          <CommitPanel />
        </Show>
        <Show when={activeTab() === 'files'}>
          <FileTree />
        </Show>
        <Show when={activeTab() === 'history'}>
          <HistoryView />
        </Show>
      </div>
    </div>
  );
}

function DiffViewer() {
  return (
    <div style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
      Select a commit to view diff.
    </div>
  );
}

function CommitPanel() {
  return (
    <div style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
      Select a commit to view details.
    </div>
  );
}

function FileTree() {
  return (
    <div style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
      Select a commit to view files.
    </div>
  );
}

function HistoryView() {
  return (
    <div style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
      Select a file to view history.
    </div>
  );
}

export default SidebarRight;