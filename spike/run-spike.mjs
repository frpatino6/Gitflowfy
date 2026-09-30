// Runs the spike in headless Chrome and pulls the real frame-time numbers.
// Uses the DevTools Protocol (CDP) so we can evaluate JS and read the results.
import { spawn } from 'node:child_process'
import { setTimeout as sleep } from 'node:timers/promises'
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const CHROME = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'
const PORT = 9222
const URL = 'http://localhost:5178/'
const REPO = process.env.REPO ?? 'C:\\Users\\fernando\\AppData\\Local\\Temp\\opencode\\llvm-spike'

// Own the server's lifetime so it can never outlive this run.
const server = spawn('node', ['server.mjs'], {
  cwd: HERE, env: { ...process.env, REPO }, stdio: ['ignore', 'pipe', 'pipe']
})
server.stdout.on('data', d => process.stdout.write('  [srv] ' + d))
server.stderr.on('data', d => process.stderr.write('  [srv!] ' + d))
for (let i = 0; i < 40; i++) {
  await sleep(250)
  try { await fetch('http://127.0.0.1:5178/graph.bin', { method: 'HEAD' }); break } catch { }
}

const chrome = spawn(CHROME, [
  '--headless=new',
  `--remote-debugging-port=${PORT}`,
  '--window-size=1920,1080',
  '--no-first-run', '--no-default-browser-check',
  '--disable-background-timer-throttling',
  '--disable-backgrounding-occluded-windows',
  '--disable-renderer-backgrounding',
  '--user-data-dir=C:\\Users\\fernando\\AppData\\Local\\Temp\\opencode\\chrome-spike',
  'about:blank'
], { stdio: ['ignore', 'ignore', 'ignore'] })

let ws, id = 0
const pending = new Map()
const send = (method, params = {}) => new Promise((res, rej) => {
  const mid = ++id
  pending.set(mid, { res, rej })
  ws.send(JSON.stringify({ id: mid, method, params }))
})

try {
  // wait for devtools
  let target = null
  for (let i = 0; i < 40; i++) {
    await sleep(500)
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}/json/list`)
      const list = await r.json()
      target = list.find((t) => t.type === 'page')
      if (target?.webSocketDebuggerUrl) break
    } catch { /* not up yet */ }
  }
  if (!target) throw new Error('could not reach Chrome DevTools')

  ws = new WebSocket(target.webSocketDebuggerUrl)
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j })
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data)
    if (m.id && pending.has(m.id)) {
      const { res, rej } = pending.get(m.id)
      pending.delete(m.id)
      m.error ? rej(new Error(m.error.message)) : res(m.result)
    }
  }

  const evaluate = async (expr) => {
    const r = await send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true })
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.text + ' ' + (r.exceptionDetails.exception?.description ?? ''))
    return r.result.value
  }

  await send('Page.enable')
  await send('Runtime.enable')

  const t0 = Date.now()
  await send('Page.navigate', { url: URL })

  // wait for the 9.6 MB graph to load and the render loop to start
  let ready = false
  for (let i = 0; i < 120; i++) {
    await sleep(500)
    const s = await evaluate(`(()=>{const e=document.getElementById('nCommits');
      return e ? e.textContent : ''})()`).catch(() => '')
    if (s && s.includes('599')) { ready = true; break }
  }
  const loadMs = Date.now() - t0
  if (!ready) throw new Error('graph never loaded')

  const gpu = await evaluate(`(()=>{try{const c=document.createElement('canvas');
    const g=c.getContext('webgl');const d=g.getExtension('WEBGL_debug_renderer_info');
    return d?g.getParameter(d.UNMASKED_RENDERER_WEBGL):'n/a'}catch(e){return 'n/a'}})()`)
  const heap = await evaluate(`performance.usedJSHeapSize`)
  console.log(`graph loaded in ${loadMs} ms`)
  console.log(`renderer       : ${gpu}`)
  console.log(`js heap        : ${(heap / 1048576).toFixed(0)} MB`)

  // baseline: idle FPS over 3 s
  await evaluate(`window.__f=[];(function L(){window.__f.push(performance.now());requestAnimationFrame(L)})()`)
  await sleep(3000)
  const idle = await evaluate(`(()=>{const f=window.__f;const d=[];for(let i=1;i<f.length;i++)d.push(f[i]-f[i-1]);
    d.sort((a,b)=>a-b);return {frames:d.length,median:d[d.length>>1]}})()`)
  console.log(`idle loop      : ${idle.frames} frames, median ${idle.median.toFixed(1)} ms (${(1000 / idle.median).toFixed(0)} fps)`)

  // the real test: full traverse of every commit
  await evaluate(`document.getElementById('stress').click()`)
  let res = null
  for (let i = 0; i < 60; i++) {
    await sleep(500)
    res = await evaluate(`window.__spike ?? null`).catch(() => null)
    if (res) break
  }
  if (!res) throw new Error('stress test never reported')

  console.log(`\n=== STRESS: traversed ${res.commits.toLocaleString()} commits ===`)
  console.log(`  frames drawn : ${res.frames.toLocaleString()}`)
  console.log(`  p50          : ${res.p50.toFixed(1)} ms  (${(1000 / res.p50).toFixed(0)} fps)`)
  console.log(`  p95          : ${res.p95.toFixed(1)} ms  (${(1000 / res.p95).toFixed(0)} fps)`)
  console.log(`  p99          : ${res.p99.toFixed(1)} ms  (${(1000 / res.p99).toFixed(0)} fps)`)
  console.log(`  max          : ${res.max.toFixed(1)} ms`)
  console.log(`  js heap      : ${res.heapMB.toFixed(0)} MB`)
  const verdict = res.p99 < 16.7 ? 'FLUIDO (60 fps)' : res.p99 < 33.4 ? 'ACEPTABLE (30 fps)' : 'STUTTER (visible jank)'
  console.log(`\n  VEREDICT     : ${verdict}`)
  console.log(JSON.stringify(res))
} catch (e) {
  console.error('FAILED:', e.message)
  process.exitCode = 1
} finally {
  try { ws?.close() } catch { }
  chrome.kill()
  server.kill()
}
