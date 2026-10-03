import { createSignal, createEffect, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  repoPath: string | null;
}

export default function StatusBar(props: Props) {
  const [repoInfo, setRepoInfo] = createSignal<{
    branch: string;
    ahead: number;
    behind: number;
    state: string;
  } | null>(null);
  const [mousePosition, setMousePosition] = createSignal({ x: 0, y: 0 });

  const updateRepoInfo = async () => {
    if (!props.repoPath) {
      setRepoInfo(null);
      return;
    }
    try {
      const info = await invoke('repo:info', { path: props.repoPath }) as any;
      setRepoInfo({
        branch: info.branch || 'detached',
        ahead: info.ahead || 0,
        behind: info.behind || 0,
        state: info.state || 'clean',
      });
    } catch {
      setRepoInfo(null);
    }
  };

  createEffect(() => {
    if (props.repoPath) {
      updateRepoInfo();
      const interval = setInterval(updateRepoInfo, 5000);
      return () => clearInterval(interval);
    }
  });

  const handleMouseMove = (e: MouseEvent) => {
    setMousePosition({ x: e.clientX, y: e.clientY });
  };

  onMount(() => {
    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  });

  return (
    <div class="statusbar" onmousemove={handleMouseMove}>
      <div style={{ flex: 1, display: 'flex', gap: 16, alignItems: 'center' }}>
        <Show when={repoInfo()}>
          <span class="status-item">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ verticalAlign: 'middle', marginRight: '4px' }}>
              <path d="M6 3v13a2 2 0 0 0 2 2h14"/>
              <path d="M6 9a2 2 0 0 0 2 2h8"/>
              <path d="M6 15a2 2 0 0 0 2 2h8"/>
            </svg>
            {repoInfo().branch}
          </span>
          <Show when={repoInfo()?.ahead > 0 || repoInfo()?.behind > 0}>
            <span class="status-item" style={{ color: 'var(--warning)' }}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ verticalAlign: 'middle', marginRight: '2px' }}>
                <polyline points="18 15 12 9 6 15"/>
              </svg>
              ↑{repoInfo().ahead}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ verticalAlign: 'middle', marginLeft: '8px', marginRight: '2px' }}>
                <polyline points="6 9 12 15 18 9"/>
              </svg>
              ↓{repoInfo().behind}
            </span>
          </Show>
          <Show when={repoInfo()?.state !== 'clean'}>
            <span class="status-item" style={{ color: 'var(--accent)' }}>
              {repoInfo()?.state}
            </span>
          </Show>
        </Show>
        <Show when={!repoInfo()}>
          <span class="status-item" style={{ color: 'var(--fg-secondary)' }}>
            No repository open
          </span>
        </Show>
      </div>
      <div style={{ display: 'flex', gap: 16, alignItems: 'center', marginLeft: 'auto' }}>
        <span style={{ fontSize: '11px', color: 'var(--fg-secondary)', fontFamily: 'var(--font-mono)' }}>
          Ln {mousePosition().y}, Col {mousePosition().x}
        </span>
        <span style={{ fontSize: '11px', color: 'var(--fg-secondary)' }}>
          UTF-8
        </span>
        <span style={{ fontSize: '11px', color: 'var(--fg-secondary)' }}>
          LF
        </span>
      </div>
    </div>
  );
}

export default StatusBar;