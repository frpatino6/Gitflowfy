import { createSignal, createEffect, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';

interface Repo {
  id: string;
  path: string;
  name: string;
  state: 'clean' | 'dirty' | 'rebasing' | 'merging' | 'bisecting';
  currentBranch: string;
  ahead: number;
  behind: number;
  stashes: number;
}

export function useRepos() {
  const [repos, setRepos] = createSignal<Repo[]>([]);
  const [activeRepoId, setActiveRepoId] = createSignal<string | null>(null);

  const addRepo = async (path: string) => {
    try {
      const repoInfo = await invoke('repo:open', { path }) as any;
      const newRepo: Repo = {
        id: repoInfo.id,
        path: repoInfo.path,
        name: repoInfo.name,
        state: repoInfo.state,
        currentBranch: repoInfo.currentBranch,
        ahead: repoInfo.ahead,
        behind: repoInfo.behind,
        stashes: repoInfo.stashes,
      };
      setRepos(prev => [...prev, newRepo]);
      setActiveRepoId(newRepo.id);
    } catch (e) {
      console.error('Failed to open repo:', e);
    }
  };

  const removeRepo = (id: string) => {
    setRepos(prev => prev.filter(r => r.id !== id));
    if (activeRepoId() === id) {
      setActiveRepoId(null);
    }
  };

  const setActive = (id: string) => {
    setActiveRepoId(id);
  };

  return {
    repos: repos,
    activeRepoId,
    activeRepo: () => repos().find(r => r.id === activeRepoId()) || null,
    addRepo,
    removeRepo,
    setActive,
  };
}