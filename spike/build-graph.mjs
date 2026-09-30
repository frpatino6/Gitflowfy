import { spawn } from 'node:child_process'
import { writeFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const REPO = process.argv[2] ?? process.env.REPO
if (!REPO) { console.error('usage: node build-graph.mjs <repo-path>'); process.exit(1) }

const CACHE = join(HERE, 'cache')
mkdirSync(CACHE, { recursive: true })

const US = '\x1f'
const t0 = performance.now()

// ---- Phase 1: stream. Row index == position in stream (git --topo-order guarantees
// children come before parents). Parent rows are NOT known yet; we defer resolution.
const rowOf = new Map()
const parentOidFlat = []
const parentOffset = [0]
const oidByRow = []
const subjectByRow = []
let n = 0
let byteCount = 0

await new Promise((resolve, reject) => {
  const proc = spawn('git', [
    '-C', REPO, 'log', '--all', '--topo-order',
    `--format=%H${US}%P${US}%ct${US}%an${US}%s`
  ], { stdio: ['ignore', 'pipe', 'inherit'] })

  let tail = ''
  proc.stdout.setEncoding('utf8')

  proc.stdout.on('data', (chunk) => {
    byteCount += Buffer.byteLength(chunk)
    const buf = tail + chunk
    let start = 0
    let nl
    while ((nl = buf.indexOf('\n', start)) !== -1) {
      const line = buf.slice(start, nl)
      start = nl + 1
      if (!line) continue
      // format is: %H US %P US %ct US %an US %s  -> exactly 4 separators
      const f1 = line.indexOf(US)
      const oid = line.slice(0, f1)
      const f2 = line.indexOf(US, f1 + 1)
      const parentsStr = line.slice(f1 + 1, f2)
      const f3 = line.indexOf(US, f2 + 1)
      const f4 = line.indexOf(US, f3 + 1)
      const subject = line.slice(f4 + 1).replace(/[\r\n\t]+/g, ' ').trim()

      rowOf.set(oid, n)
      oidByRow[n] = oid
      subjectByRow[n] = subject

      if (parentsStr.length) {
        for (const p of parentsStr.split(' ')) parentOidFlat.push(p)
      }
      parentOffset.push(parentOidFlat.length)
      n++
    }
    tail = buf.slice(start)
  })

  proc.on('error', reject)
  proc.on('close', (code) => code === 0 ? resolve() : reject(new Error('git exit ' + code)))
})

const tParse = performance.now()
const totalParents = parentOidFlat.length
const t1 = performance.now()

// ---- Phase 2: resolve parent OIDs -> row indices (map is now complete)
const parentRow = new Uint32Array(totalParents)
let unresolved = 0
for (let k = 0; k < totalParents; k++) {
  const r = rowOf.get(parentOidFlat[k])
  if (r === undefined) { unresolved++; parentRow[k] = 0 }
  else parentRow[k] = r
}
const t2 = performance.now()

// ---- Phase 3: lane assignment. Walk top-down; a lane's next expected commit is
// tracked in `pending`. First parent continues straight down; extra (merge) parents
// branch into their own lane.
const lane = new Int32Array(n)
const pending = new Map()
const freeLanes = []
let laneCount = 0
let peakLanes = 0
const newLane = () => (freeLanes.length ? freeLanes.pop() : laneCount++)

for (let r = 0; r < n; r++) {
  const s = parentOffset[r]
  const e = parentOffset[r + 1]
  const np = e - s
  let L
  if (pending.has(r)) { L = pending.get(r); pending.delete(r) }
  else L = newLane()
  lane[r] = L
  if (peakLanes < laneCount) peakLanes = laneCount
  if (np > 0) {
    const p0 = parentRow[s]
    if (pending.has(p0)) freeLanes.push(L)
    else pending.set(p0, L)
    for (let k = s + 1; k < e; k++) {
      const p = parentRow[k]
      if (!pending.has(p)) pending.set(p, newLane())
    }
  }
}
const t3 = performance.now()

// ---- Phase 4: lane each parent edge occupies (for curve drawing)
const parentLane = new Int32Array(totalParents)
let merges = 0
for (let r = 0; r < n; r++) {
  const s = parentOffset[r]
  const e = parentOffset[r + 1]
  if (e - s > 1) merges++
  for (let k = s; k < e; k++) parentLane[k] = lane[parentRow[k]]
}
const t4 = performance.now()

const parts = [
  ['n', Int32Array.of(n)],
  ['offset', Uint32Array.from(parentOffset)],
  ['prow', parentRow],
  ['plane', parentLane],
  ['lane', lane]
]
let total = 0
for (const p of parts) total += p[1].byteLength
const buf = new ArrayBuffer(total)
let cursor = 0
for (const [name, a] of parts) {
  const view = new a.constructor(buf, cursor, a.length)
  view.set(a)
  cursor += a.byteLength
  console.log(`  ${name.padEnd(7)} ${String(a.length).padStart(9)} elems  ${(a.byteLength / 1e6).toFixed(2)} MB`)
}
writeFileSync(join(CACHE, 'graph.bin'), Buffer.from(buf))
writeFileSync(join(CACHE, 'oids.txt'), oidByRow.join('\n'))
writeFileSync(join(CACHE, 'subjects.txt'), subjectByRow.join('\n'))

const tEnd = performance.now()
console.log(`\n  git -> JS stream : ${(tParse - t0).toFixed(0)} ms   (${(byteCount / 1e6).toFixed(1)} MB stdout)`)
console.log(`  commits          : ${n.toLocaleString()}`)
console.log(`  parent edges     : ${totalParents.toLocaleString()}`)
console.log(`  merge commits    : ${merges.toLocaleString()}`)
console.log(`  unresolved edges : ${unresolved}`)
console.log(`  parent resolve   : ${(t2 - t1).toFixed(0)} ms`)
console.log(`  lane assignment  : ${(t3 - t2).toFixed(0)} ms   peak lanes = ${peakLanes}`)
console.log(`  edge lane map    : ${(t4 - t3).toFixed(0)} ms`)
console.log(`  binary pack+write: ${(tEnd - t4).toFixed(0)} ms   -> cache/graph.bin (${(total / 1e6).toFixed(1)} MB)`)
console.log(`\n  TOTAL BUILD      : ${(tEnd - t0).toFixed(0)} ms`)
console.log(`  peak rss         : ${(process.memoryUsage().rss / 1e6).toFixed(0)} MB`)
