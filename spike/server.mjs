import { createServer } from 'node:http'
import { readFileSync, existsSync } from 'node:fs'
import { spawn } from 'node:child_process'
import { dirname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const PORT = Number(process.env.PORT ?? 5178)
const REPO = process.env.REPO ?? ''

const graphBuf = readFileSync(join(HERE, 'cache', 'graph.bin'))
// Node's readFileSync gives a Buffer (a Uint8Array). Passing a TypedArray to the
// Int32Array constructor CONVERTS VALUES instead of aliasing memory, so we must
// hand it a real ArrayBuffer slice.
const graph = graphBuf.buffer.slice(graphBuf.byteOffset, graphBuf.byteOffset + graphBuf.length)
const oidLines = readFileSync(join(HERE, 'cache', 'oids.txt'), 'utf8').split('\n')
const subjectLines = readFileSync(join(HERE, 'cache', 'subjects.txt'), 'utf8').split('\n')
const N = new Int32Array(graph, 0, 1)[0]
console.log(`serving graph: ${N.toLocaleString()} commits, ${(graph.byteLength / 1e6).toFixed(1)} MB binary`)

const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css' }

function gitShow(oid) {
  return new Promise((resolve) => {
    const p = spawn('git', ['-C', REPO, 'show', '-s',
      '--format=%H%x1f%an%x1f%ae%x1f%at%x1f%P%x1f%s%x1f%b', oid])
    let out = ''
    p.stdout.setEncoding('utf8')
    p.stdout.on('data', (c) => { out += c })
    p.on('close', () => {
      const f = out.split('\x1f')
      resolve({
        oid: f[0] ?? oid, author: f[1] ?? '', email: f[2] ?? '',
        date: Number(f[3] ?? 0) * 1000, parents: (f[4] ?? '').split(' ').filter(Boolean),
        subject: f[5] ?? '', body: (f[6] ?? '').trim()
      })
    })
    p.on('error', () => resolve(null))
  })
}

createServer(async (req, res) => {
  const u = new URL(req.url, 'http://localhost')

  if (u.pathname === '/graph.bin') {
    res.writeHead(200, { 'content-type': 'application/octet-stream', 'content-length': graph.byteLength })
    // res.end() takes a Buffer/string/Uint8Array, not a raw ArrayBuffer.
    // Buffer.from(arrayBuffer) is a zero-copy view, so nothing is duplicated.
    return res.end(Buffer.from(graph))
  }

  // Lazy label window: only the rows actually on screen need subjects.
  if (u.pathname === '/window') {
    const from = Math.max(0, Math.min(N - 1, Number(u.searchParams.get('from') ?? 0)))
    const count = Math.max(0, Math.min(N - from, Number(u.searchParams.get('count') ?? 200)))
    res.writeHead(200, { 'content-type': 'application/json' })
    return res.end(JSON.stringify({ from, subjects: subjectLines.slice(from, from + count) }))
  }

  if (u.pathname.startsWith('/commit/')) {
    const row = Number(u.pathname.slice(8))
    if (!Number.isInteger(row) || row < 0 || row >= N) {
      res.writeHead(404); return res.end('{}')
    }
    const d = await gitShow(oidLines[row])
    res.writeHead(200, { 'content-type': 'application/json' })
    return res.end(JSON.stringify(d))
  }

  let p = u.pathname === '/' ? '/index.html' : u.pathname
  const f = join(HERE, 'public', normalize(p).replace(/^(\.\.[/\\])+/, ''))
  if (!existsSync(f)) { res.writeHead(404); return res.end('not found') }
  const dot = f.lastIndexOf('.')
  res.writeHead(200, { 'content-type': MIME[f.slice(dot)] ?? 'application/octet-stream' })
  res.end(readFileSync(f))
}).listen(PORT, () => console.log(`http://localhost:${PORT}`))
