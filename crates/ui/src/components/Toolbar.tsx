import { createSignal } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  onOpenRepo: () => void;
  repoPath: string | null;
}

export default function Toolbar(props: Props) {
  const [showCommitDialog, setShowCommitDialog] = createSignal(false);

  const handleCommit = async () => {
    if (!showCommitDialog()) return;
    try {
      await invoke('git:command', { 
        repo: props.repoPath, 
        args: ['commit', '-m', 'commit message'] 
      });
    } catch (e) {
      console.error('Commit failed:', e);
    }
  };

  const handlePush = async () => {
    try {
      await invoke('git:command', { 
        repo: props.repoPath, 
        args: ['push'] 
      });
    } catch (e) {
      console.error('Push failed:', e);
    }
  };

  const handlePull = async () => {
    try {
      await invoke('git:command', { 
        repo: props.repoPath, 
        args: ['pull'] 
      });
    } catch (e) {
      console.error('Pull failed:', e);
    }
  };

  const handleFetch = async () => {
    try {
      await invoke('git:command', { 
        repo: props.repoPath, 
        args: ['fetch', '--all', '--prune'] 
      });
    } catch (e) {
      console.error('Fetch failed:', e);
    }
  };

  const handleOpenRepo = () => {
    props.onOpenRepo();
  };

  return (
    <div class="toolbar">
      <button class="toolbar-btn" onClick={props.onOpenRepo} title="Open Repository (Ctrl+O)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        Open Repo
      </button>
      
      <div class="toolbar-separator"></div>
      
      <button class="toolbar-btn" onClick={handleCommit} title="Commit (Ctrl+S)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
          <polyline points="9 20 9 10 15 10"/>
        </svg>
        Commit
      </button>
      
      <button class="toolbar-btn" onClick={handlePush} title="Push (Ctrl+P)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M18 15l-6-6-6 6"/>
          <path d="M12 19V9"/>
        </svg>
        Push
      </button>
      
      <button class="toolbar-btn" onClick={handlePull} title="Pull (Ctrl+Shift+P)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6 9l6 6 6-6"/>
          <path d="M12 15V3"/>
        </svg>
        Pull
      </button>
      
      <button class="toolbar-btn" onClick={handleFetch} title="Fetch (Ctrl+Shift+F)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="10"/>
          <polyline points="12 6 12 12 16 14"/>
        </svg>
        Fetch
      </button>
      
      <div class="toolbar-separator"></div>
      
      <button class="toolbar-btn" title="Branch (Ctrl+B)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6 3v13a2 2 0 0 0 2 2h14"/>
          <path d="M6 9a2 2 0 0 0 2 2h8"/>
          <path d="M6 15a2 2 0 0 0 2 2h8"/>
        </svg>
        Branch
      </button>
      
      <button class="toolbar-btn" title="Merge (Ctrl+M)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 3v18"/>
          <path d="M3 12h18"/>
        </svg>
        Merge
      </button>
      
      <button class="toolbar-btn" title="Rebase (Ctrl+R)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M12 5v14"/>
          <path d="M19 12H5"/>
          <path d="M12 5l-7 7 7 7"/>
        </svg>
        Rebase
      </button>
      
      <button class="toolbar-btn" title="Stash (Ctrl+T)" disabled={!props.repoPath}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M21 6H3a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h18a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2z"/>
          <polyline points="4 12 12 20 20 12"/>
        </svg>
        Stash
      </button>
      
      <div class="toolbar-separator"></div>
      
      <button class="toolbar-btn" title="Settings (Ctrl+,)">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 1-1.82-.33l-.06-.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83 1.65 1.65 0 0 1 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83 1.65 1.65 0 0 1 1.82.33l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06.06a1.65 1.65 0 0 1-1.82.33H21a2 2 0 0 1 2 2v.01"/>
        </svg>
      </button>
    </div>
  );
}

export default Toolbar;