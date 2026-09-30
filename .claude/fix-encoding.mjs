// Normalizes all text files to pure ASCII.
//
// Why: em-dashes and smart quotes get double-encoded into mojibake when text
// passes through a Windows shell, and a model reading "a - b" as "a " b"
// is reading corruption. ASCII cannot be corrupted this way.
//
// This script rewrites ITSELF only if it contains non-ASCII, so it is written
// with escape sequences rather than literal characters.
import { readFileSync, writeFileSync, readdirSync, statSync } from 'node:fs'
import { join } from 'node:path'

const SKIP = new Set(['node_modules', 'cache', 'target', '.git', '.opencode'])
const EXT = /\.(md|json|mjs|js|ts|html)$/

// Escape sequences, not literals: keeps this file pure ASCII so it can safely
// run itself.
const EMDASH = '\u2014'
const ENDASH = '\u2013'
const LSQUO = '\u2018'
const RSQUO = '\u2019'
const LDQUO = '\u201C'
const RDQUO = '\u201D'
const SECT = '\u00A7'
const ELLIP = '\u2026'
const TIMES = '\u00D7'
const GE = '\u2265'
const LE = '\u2264'
const ARROW = '\u2192'
const DEG = '\u00B0'
const NBSP = '\u00A0'

// Residue from a partial double-encode: the three-char sequence that an em-dash
// collapses into after two bad round trips.
const MOJI = [
  ['\u00E2\u20AC\u201D', '"'],
  ['\u00E2\u20AC\u2122', '"'],
  ['\u00E2\u20AC\u009D', "'"],
  ['\u00E2\u20AC\u0093', '"'],
  ['\u00E2\u20AC\u2122', '"'],
  ['\u00E2\u20AC\u201C', '"'],
  ['\u00E2\u20AC\u00A6', '...'],
  ['\u00E2\u0080', ''],
  ['\u00C2\u00A0', ' '],
  ['\u00C2', ''],
  ['\u00E2', ''],
  ['\u20AC', ''],
  ['\uFFFD', '']
]

const REAL = new Map([
  [EMDASH, '-'], [ENDASH, '-'], [LSQUO, "'"], [RSQUO, "'"],
  [LDQUO, '"'], [RDQUO, '"'], [SECT, 'SS'], [ELLIP, '...'],
  [TIMES, 'x'], [GE, '>='], [LE, '<='], [ARROW, '->'], [DEG, ' deg'],
  [NBSP, ' ']
])

function toAscii (text) {
  let out = text
  for (const [from, to] of MOJI) out = out.split(from).join(to)
  out = [...out].map(c => REAL.get(c) ?? c).join('')
  return out.replace(/[^\x00-\x7F]/g, '')
}

const walk = (dir, out = []) => {
  for (const name of readdirSync(dir)) {
    if (SKIP.has(name)) continue
    const full = join(dir, name)
    if (statSync(full).isDirectory()) walk(full, out)
    else if (EXT.test(full)) out.push(full)
  }
  return out
}

let changed = 0
for (const path of walk(process.cwd())) {
  const before = readFileSync(path, 'utf8')
  const after = toAscii(before)
  if (after !== before) {
    writeFileSync(path, after, 'utf8')
    changed++
    console.log('  ' + path.slice(process.cwd().length + 1))
  }
}
console.log(changed + ' files normalized to ASCII')
