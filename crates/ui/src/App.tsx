import { createSignal, onMount, Show } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import GraphCanvas from './graph/GraphCanvas';
import Toolbar from './components/Toolbar';
import SidebarLeft from './components/SidebarLeft';
import SidebarRight from './components/SidebarRight';
import StatusBar from './components/StatusBar';
import { useRepos } from './state/repos';
import { useTheme } from './state/theme';
import { useShortcuts } from './state/shortcuts';
import './styles.css';

function App() {
  const [repoPath, setRepoPath] = createSignal<string | null>(null);
  const [graphLoaded, setGraphLoaded] = createSignal(false);
  const repos = useRepos();
  const theme = useTheme();
  const shortcuts = useShortcuts();

  const handleOpenRepo = async () => {
    try {
      const path = await invoke('dialog:open', { directory: true });
      if (path) {
        setRepoPath(path);
        await loadGraph(path);
      }
    } catch (e) {
      console.error('Failed to open repo:', e);
    }
  };

  const loadGraph = async (repoPath: string) => {
    try {
      const graphData = await invoke('graph:load', { repo: repoPath });
      // TODO: Load graph into canvas
      setGraphLoaded(true);
    } catch (e) {
      console.error('Failed to load graph:', e);
    }
  };

  return (
    <div class="app" data-theme={theme()}>
      <div class="toolbar-container">
        <Toolbar onOpenRepo={handleOpenRepo} repoPath={repoPath()} />
      </div>
      <div class="main-content">
        <SidebarLeft repoPath={repoPath()} />
        <div class="graph-container">
          {repoPath() && (
            <GraphCanvas repoPath={repoPath()} />
          )}
        </div>
        <SidebarRight repoPath={repoPath()} />
      </div>
      <StatusBar repoPath={repoPath()} />
    </div>
  );
}

export default App;