import { createSignal, createMemo } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  repoPath: string | null;
}

export default function SidebarLeft(props: Props) {
  const [activeTab, setActiveTab] = createSignal<'repos' | 'branches' | 'tags' | 'remotes' | 'stashes'>('repos');
  
  const tabs = [
    { id: 'repos', label: 'Repos', icon: 'repo' },
    { id: 'branches', label: 'Branches', icon: 'branch' },
    { id: 'tags', label: 'Tags', icon: 'tag' },
    { id: 'remotes', label: 'Remotes', icon: 'remote' },
    { id: 'stashes', label: 'Stashes', icon: 'stash' },
  ];

  return (
    <div class="sidebar-left">
      <div class="sidebar-tabs">
        {tabs.map(tab => (
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
        <Show when={activeTab() === 'repos'}>
          <RepoList />
        </Show>
        <Show when={activeTab() === 'branches'}>
          <BranchList />
        </Show>
        <Show when={activeTab() === 'tags'}>
          <TagList />
        </Show>
        <Show when={activeTab() === 'remotes'}>
          <RemoteList />
        </Show>
        <Show when={activeTab() === 'stashes'}>
          <StashList />
        </Show>
      </div>
    </div>
  );
}

function RepoList() {
  return (
    <div>
      <div class="repo-item" style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
        No repositories open. Click "Open Repo" to start.
      </div>
    </div>
  );
}

function BranchList() {
  return (
    <div>
      <div class="branch-item" style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
        No repository selected.
      </div>
    </div>
  );
}

function TagList() {
  return (
    <div>
      <div class="tag-item" style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
        No repository selected.
      </div>
    </div>
  );
}

function RemoteList() {
  return (
    <div>
      <div class="remote-item" style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
        No repository selected.
      </div>
    </div>
  );
}

function StashList() {
  return (
    <div>
      <div class="stash-item" style={{ padding: '8px', color: 'var(--fg-secondary)', fontSize: '12px' }}>
        No repository selected.
      </div>
    </div>
  );
}

export default SidebarLeft;