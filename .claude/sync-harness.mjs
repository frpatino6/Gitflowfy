// Wires the SDD harness so Claude Code and opencode read the SAME files.
// Claude Code natively loads .claude/agents, .claude/skills, .claude/commands.
// opencode loads .opencode/agent|.opencode/command|.opencode/skills.
// Symlinks need admin on Windows, so we use NTFS hardlinks: one file on disk,
// two directory entries, zero drift possible.
//
// Hardlinks work per-file and need both targets on the same volume. Re-running
// is safe. If a hardlink fails (cross-volume, or a filesystem without support),
// the script falls back to a copy and says so loudly.
import { linkSync, copyFileSync, existsSync, unlinkSync, statSync, readdirSync, mkdirSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const CLAUDE = join(ROOT, '.claude')
const OPENCODE = join(ROOT, '.opencode')

const LINKS = [
  // agents: .claude/agents/*.md  ->  .opencode/agent/*.md
  { from: join(CLAUDE, 'agents'), to: join(OPENCODE, 'agent'), ext: '.md' },
  // commands
  { from: join(CLAUDE, 'commands'), to: join(OPENCODE, 'command'), ext: '.md' },
  // skills: whole SKILL.md-bearing folders
  { from: join(CLAUDE, 'skills'), to: join(OPENCODE, 'skills'), ext: 'SKILL.md' }
]

let linked = 0
let copied = 0
const problems = []

for (const { from, to, ext } of LINKS) {
  if (!existsSync(from)) { problems.push(`missing source: ${from}`); continue }
  const names = readdirSync(from)
  for (const name of names) {
    if (ext === 'SKILL.md') {
      // A skill is a FOLDER containing SKILL.md, not a file. Link the file, not
      // the folder: NTFS refuses to hardlink a directory.
      const src = join(from, name, 'SKILL.md')
      if (!existsSync(src)) continue
      const dir = join(to, name)
      mkdirSync(dir, { recursive: true })
      const dst = join(dir, 'SKILL.md')
      if (existsSync(dst)) { try { unlinkSync(dst) } catch { } }
      try {
        linkSync(src, dst)
        linked++
      } catch (e) {
        try { copyFileSync(src, dst); copied++; problems.push(`COPIED (not linked): ${name} - ${e.code ?? e.message}`) }
        catch (e2) { problems.push(`FAILED: ${name} - ${e2.message}`) }
      }
      continue
    }

    const src = join(from, name)
    if (!existsSync(src)) continue
    const dst = join(to, name)
    if (existsSync(dst)) { try { unlinkSync(dst) } catch { } }
    try {
      linkSync(src, dst)
      linked++
    } catch (e) {
      try { copyFileSync(src, dst); copied++; problems.push(`COPIED (not linked): ${name} - ${e.code ?? e.message}`) }
      catch (e2) { problems.push(`FAILED: ${name} - ${e2.message}`) }
    }
  }
}

// Verify a couple of links actually share a file identity, so a broken link
// cannot masquerade as a working harness.
let verified = 0
try {
  const a = statSync(join(CLAUDE, 'agents', 'sdd-spec.md'))
  const b = statSync(join(OPENCODE, 'agent', 'sdd-spec.md'))
  if (a.ino === b.ino && a.dev === b.dev) verified++
  else problems.push('sdd-spec.md is NOT the same file on both sides')
} catch { problems.push('could not stat sdd-spec.md on both sides') }

console.log(`hardlinks : ${linked}`)
console.log(`copies    : ${copied}`)
console.log(`verified  : ${verified > 0 ? 'same file identity confirmed' : 'FAILED'}`)
if (problems.length) {
  console.log('\nPROBLEMS:')
  for (const p of problems) console.log('  - ' + p)
} else {
  console.log('\nNo problems. One source of truth, two readers.')
}
