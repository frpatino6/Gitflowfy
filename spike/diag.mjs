import { spawn } from 'node:child_process'
import { setTimeout as sleep } from 'node:timers/promises'

const CHROME = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe'
const PORT = 9223
const chrome = spawn(CHROME, ['--headless=new', `--remote-debugging-port=${PORT}`,
  '--window-size=1920,1080', '--no-first-run',
  '--user-data-dir=C:\\Users\\fernando\\AppData\\Local\\Temp\\opencode\\chrome-diag', 'about:blank'],
  { stdio: ['ignore', 'ignore', 'ignore'] })

let ws, id = 0
const pending = new Map()
const send = (m, p = {}) => new Promise((res, rej) => {
  const i = ++id; pending.set(i, { res, rej })
  ws.send(JSON.stringify({ id: i, method: m, params: p }))
})

try {
  let target = null
  for (let i = 0; i < 40; i++) {
    await sleep(500)
    try { const l = await (await fetch(`http://127.0.0.1:${PORT}/json/list`)).json()
      target = l.find((t) => t.type === 'page'); if (target?.webSocketDebuggerUrl) break } catch {}
  }
  ws = new WebSocket(target.webSocketDebuggerUrl)
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j })
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data)
    if (m.id && pending.has(m.id)) { const { res, rej } = pending.get(m.id); pending.delete(m.id)
      m.error ? rej(new Error(m.error.message)) : res(m.result) }
    else if (m.method === 'Runtime.exceptionThrown') {
      const d = m.params.exceptionDetails
      console.log('EXCEPTION:', d.text, '|', d.exception?.description ?? d.exception?.value ?? '')
    } else if (m.method === 'Runtime.consoleAPICalled') {
      console.log('CONSOLE[' + m.params.type + ']:',
        m.params.args.map((a) => a.value ?? a.description ?? a.type).join(' '))
    } else if (m.method === 'Log.entryAdded') {
      console.log('LOG[' + m.params.entry.level + ']:', m.params.entry.text)
    }
  }
  const ev = async (e) => (await send('Runtime.evaluate', { expression: e, returnByValue: true, awaitPromise: true }))

  await send('Runtime.enable'); await send('Log.enable'); await send('Page.enable')
  await send('Page.navigate', { url: 'http://localhost:5178/' })
  await sleep(8000)
  const r = await ev(`(()=>({
    title: document.title,
    nCommits: document.getElementById('nCommits')?.textContent,
    hasLoad: typeof load,
    graphN: (typeof n!=='undefined')?n:'undef',
    offLen: (typeof off!=='undefined'&&off)?off.length:'undef',
    subjectProbe: (typeof subjects!=='undefined')?subjects.size:'undef',
    bodyLen: document.body.innerHTML.length
  }))()`)
  console.log('STATE:', JSON.stringify(r.result.value ?? r, null, 2))
} catch (e) { console.error('DIAG FAILED:', e.message) }
finally { try { ws?.close() } catch {} ; chrome.kill() }
