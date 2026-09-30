// Audits text files for non-ASCII characters and mojibake.
// Companion to fix-encoding.mjs: this one only reports, never writes.
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const SKIP = new Set(['node_modules', 'cache', 'target', '.git'])
const EXT = /\.(md|json|mjs|js|ts|html)$/

const MOJI_MARK = /\u00E2[\u0080-\u00BF\u20AC]|.||\uFFFD/

const walk = (dir, out = []) => {
  for (const name of readdirSync(dir)) {
    if (SKIP.has(name)) continue
    const full = join(dir, name)
    if (statSync(full).isDirectory()) walk(full, out)
    else if (EXT.test(full)) out.push(full)
  }
  return out
}

let clean = 0
const flagged = []

for (const path of walk(process.cwd())) {
  const text = readFileSync(path, 'utf8')
  const nonAscii = [...text].filter(c => c.codePointAt(0) > 127)
  const rel = relative(process.cwd(), path)

  if (nonAscii.length === 0) { clean++; continue }

  const isMojibake = MOJI_MARK.test(text)
  const codes = [...new Set(nonAscii)]
    .map(c => 'U+' + c.codePointAt(0).toString(16).toUpperCase().padStart(4, '0'))
  flagged.push({
    rel,
    count: nonAscii.length,
    chars: codes.join(' '),
    mojibake: isMojibake
  })
}

for (const f of flagged) {
  const kind = f.mojibake ? 'MOJIBAKE' : 'non-ascii'
  console.log(`  ${kind}  ${f.rel}  (${f.count})  ${f.chars}`)
}
console.log(`\nclean: ${clean}   flagged: ${flagged.length}`)
console.log(flagged.length === 0
  ? 'All text files are pure ASCII.'
  : 'Run: node .claude/fix-encoding.mjs')
