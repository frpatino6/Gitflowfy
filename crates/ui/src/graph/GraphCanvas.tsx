import { createSignal, createEffect, onMount, onCleanup, useRef, createMemo } from 'solid-js';
import { invoke } from '@tauri-apps/api/core';
import { decodeGraph, GraphData } from './decode';

interface Props {
  repoPath: string;
}

interface Commit {
  oid: string;
  x: number;
  y: number;
  lane: number;
  subject: string;
  author: string;
  date: number;
  parents: number[];
  children: number[];
}

interface Viewport {
  x: number;
  y: number;
  zoom: number;
}

export default function GraphCanvas(props: { repoPath: string }) {
  const [commits, setCommits] = createSignal<Commit[]>([]);
  const [viewport, setViewport] = createSignal<Viewport>({ x: 0, y: 0, zoom: 1 });
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedCommit, setSelectedCommit] = createSignal<number | null>(null);
  const [viewportSize, setViewportSize] = createSignal({ width: 800, height: 600 });
  const [graphData, setGraphData] = createSignal<GraphData | null>(null);
  
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const minimapRef = useRef<HTMLCanvasElement>(null);

  // Load graph data
  const loadGraph = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const graphData = await invoke('graph:load', { repo: props.repoPath }) as ArrayBuffer;
      const graph = decodeGraph(new Uint8Array(graphData));
      setGraphData(graph);
      setCommits(buildCommits(graph));
      setLoading(false);
    } catch (e) {
      console.error('Failed to load graph:', e);
      setError('Failed to load graph');
      setLoading(false);
    }
  };

  // Build commit objects from graph data using CSR arrays
  const buildCommits = (graph: GraphData): Commit[] => {
    const { n, m, parentIndex, parentList, commitTime, lane, parentCount, subjects, authors, refs } = graph;
    
    // Build children array (reverse of parents)
    const children: number[][] = new Array(n).fill(0).map(() => []);
    for (let i = 0; i < n; i++) {
      const start = parentIndex[i];
      const end = parentIndex[i + 1];
      for (let j = start; j < end; j++) {
        const parentIdx = parentList[j];
        if (parentIdx < n) {
          children[parentIdx].push(i);
        }
      }
    }
    
    // Calculate layout using CSR lanes (topological order)
    // Commits are already in topo-order (newest first) from git log
    // Lane assignment is already done by the builder
    
    const COMMIT_SPACING = 100;
    const LANE_SPACING = 80;
    const MARGIN = 100;
    
    const maxLane = Math.max(...lane.slice(0, n), 0);
    
    return Array.from({ length: n }, (_, i) => ({
      oid: '',
      x: MARGIN + i * COMMIT_SPACING,
      y: MARGIN + lane[i] * LANE_SPACING,
      lane: lane[i],
      subject: subjects[i] || '',
      author: authors[i] || '',
      date: Number(commitTime[i]),
      parents: Array.from({ length: parentCount[i] }, (_, j) => parentList[parentIndex[i] + j]),
      children: children[i],
    }));
  };

  // Compute layout bounds
  const layout = createMemo(() => {
    const commits = getCommits();
    if (commits.length === 0) {
      return { width: 1000, height: 1000, lanes: 0 };
    }
    const lanes = Math.max(...commits.map(c => c.lane), 0) + 1;
    const width = commits.length * 100 + 200;
    const height = lanes * 80 + 200;
    
    return { commits, width, height, lanes };
  });

  const draw = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d')!;
    const commits = getCommits();
    const vp = viewport();
    const size = viewportSize();
    
    canvas.width = size.width * window.devicePixelRatio;
    canvas.height = size.height * window.devicePixelRatio;
    canvas.style.width = `${size.width}px`;
    canvas.style.height = `${size.height}px`;
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    
    // Clear
    ctx.fillStyle = '#1e1e1e';
    ctx.fillRect(0, 0, size.width, size.height);
    
    // Apply viewport transform
    ctx.save();
    ctx.translate(-vp.x, -vp.y);
    ctx.scale(vp.zoom, vp.zoom);
    
    // Draw edges first (behind nodes)
    drawEdges(ctx);
    
    // Draw nodes
    drawNodes(ctx);
    
    // Draw selection highlight
    const selected = selectedCommit();
    if (selected !== null) {
      drawSelection(ctx, selected);
    }
    
    ctx.restore();
    
    // Draw minimap
    drawMinimap();
  };

  const getCommits = () => {
    return layout().commits;
  };

  const drawEdges = (ctx: CanvasRenderingContext2D) => {
    const commits = getCommits();
    ctx.strokeStyle = '#555';
    ctx.lineWidth = 1.5;
    
    commits.forEach((commit, i) => {
      commit.parents.forEach(parentIdx => {
        if (parentIdx < commits.length) {
          const parent = commits[parentIdx];
          ctx.beginPath();
          ctx.moveTo(commit.x, commit.y);
          ctx.lineTo(parent.x, parent.y);
          ctx.stroke();
        }
      });
    });
    
    // Draw merge edges (children pointing to this)
    commits.forEach((commit, i) => {
      commit.children.forEach(childIdx => {
        if (childIdx < commits.length) {
          const child = commits[childIdx];
          ctx.beginPath();
          ctx.moveTo(commit.x, commit.y);
          ctx.lineTo(child.x, child.y);
          ctx.stroke();
        }
      });
    });
  }

  const drawNodes = (ctx: CanvasRenderingContext2D) => {
    const commits = getCommits();
    const COMMIT_RADIUS = 8;
    
    commits.forEach((commit, i) => {
      const selected = selectedCommit() === i;
      
      ctx.beginPath();
      ctx.arc(commit.x, commit.y, COMMIT_RADIUS, 0, Math.PI * 2);
      
      if (i === selectedCommit()) {
        ctx.fillStyle = '#007acc';
        ctx.strokeStyle = '#0098ff';
        ctx.lineWidth = 2;
      } else {
        ctx.fillStyle = '#3c3c3c';
        ctx.strokeStyle = '#555';
        ctx.lineWidth = 1;
      }
      
      ctx.fill();
      ctx.stroke();
      
      // Draw subject text
      ctx.fillStyle = '#ccc';
      ctx.font = '11px monospace';
      ctx.textAlign = 'center';
      ctx.fillText(commit.subject.substring(0, 30), commit.x, commit.y - COMMIT_RADIUS - 4);
    });
  };

  const drawSelection = (ctx: CanvasRenderingContext2D, index: number) => {
    const commit = getCommits()[index];
    const COMMIT_RADIUS = 8;
    ctx.beginPath();
    ctx.arc(commit.x, commit.y, COMMIT_RADIUS + 3, 0, Math.PI * 2);
    ctx.strokeStyle = '#0098ff';
    ctx.lineWidth = 2;
    ctx.stroke();
  };

  const drawMinimap = () => {
    const canvas = minimapRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d')!;
    const layout = layout();
    
    canvas.width = 200 * window.devicePixelRatio;
    canvas.height = 200 * window.devicePixelRatio;
    canvas.style.width = '200px';
    canvas.style.height = '200px';
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    
    // Clear
    ctx.fillStyle = '#1a1a1a';
    ctx.fillRect(0, 0, 200, 200);
    
    const scaleX = 200 / layout.width;
    const scaleY = 200 / layout.height;
    
    // Draw commits
    getCommits().forEach(commit => {
      ctx.fillStyle = '#555';
      ctx.beginPath();
      ctx.arc(commit.x * scaleX, commit.y * scaleY, 1, 0, Math.PI * 2);
      ctx.fill();
    });
    
    // Draw viewport rectangle
    const vp = viewport();
    ctx.strokeStyle = '#0098ff';
    ctx.lineWidth = 1;
    ctx.strokeRect(
      -vp.x * scaleX / vp.zoom,
      -vp.y * scaleY / vp.zoom,
      viewportSize().width * scaleX / vp.zoom,
      viewportSize().height * scaleY / vp.zoom
    );
  };

  // Viewport interaction
  let isPanning = false;
  let lastMouseX = 0;
  let lastMouseY = 0;

  const handleMouseDown = (e: MouseEvent) => {
    if (e.button === 1 || (e.button === 0 && e.shiftKey)) {
      isPanning = true;
      lastMouseX = e.clientX;
      lastMouseY = e.clientY;
      e.preventDefault();
    } else if (e.button === 0) {
      // Click to select
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      const x = (e.clientX - rect.left + viewport().x) / viewport().zoom;
      const y = (e.clientY - rect.top + viewport().y) / viewport().zoom;
      
      // Find closest commit
      const commits = getCommits();
      let closest = -1;
      let minDist = Infinity;
      getCommits().forEach((commit, i) => {
        const dx = commit.x - x;
        const dy = commit.y - y;
        const dist = dx * dx + dy * dy;
        if (dist < minDist && dist < 100) {
          minDist = dist;
          closest = i;
        }
      });
      setSelectedCommit(closest >= 0 ? closest : null);
    }
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isPanning) return;
    const dx = e.clientX - lastMouseX;
    const dy = e.clientY - lastMouseY;
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
    setViewport(v => ({ ...v, x: v.x - dx / v.zoom, y: v.y - dy / v.zoom }));
  };

  const handleMouseUp = () => {
    isPanning = false;
  };

  const handleWheel = (e: WheelEvent) => {
    e.preventDefault();
    const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.min(3, Math.max(0.2, viewport().zoom * zoomFactor));
    setViewport(v => ({ ...v, zoom: newZoom }));
  };

  // Canvas resize observer
  const resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const { width, height } = entry.contentRect;
      setViewportSize({ width, height });
      if (canvasRef.current) {
        canvasRef.current.width = width * window.devicePixelRatio;
        canvasRef.current.height = height * window.devicePixelRatio;
        canvasRef.current.style.width = `${width}px`;
        canvasRef.current.style.height = `${height}px`;
      }
      draw();
    }
  });

  onMount(() => {
    loadGraph();
    if (canvasRef.current) {
      resizeObserver.observe(canvasRef.current);
    }
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
    window.addEventListener('wheel', handleWheel, { passive: false });
  });

  onCleanup(() => {
    resizeObserver.disconnect();
    window.removeEventListener('mousemove', handleMouseMove);
    window.removeEventListener('mouseup', handleMouseUp);
    window.removeEventListener('wheel', handleWheel);
  });

  // Draw loop
  createEffect(() => {
    draw();
  });

  return (
    <div class="graph-canvas-wrapper">
      <canvas
        ref={canvasRef}
        class="graph-canvas"
        onmousedown={handleMouseDown}
        style={{ width: '100%', height: '100%' }}
      />
      <canvas 
        ref={minimapRef} 
        class="graph-minimap" 
        width={200} 
        height={200}
      />
    </div>
  );
}

interface Commit {
  oid: string;
  x: number;
  y: number;
  lane: number;
  subject: string;
  author: string;
  date: number;
  parents: number[];
  children: number[];
}

interface Viewport {
  x: number;
  y: number;
  zoom: number;
}

export default GraphCanvas;