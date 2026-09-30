import fs from "fs";

const dir = "specs/001-headless-git-operations/";
const spec = fs.readFileSync(dir + "spec.md", "utf8");
const plan = fs.readFileSync(dir + "plan.md", "utf8");
const tasks = fs.readFileSync(dir + "tasks.md", "utf8");
const research = fs.readFileSync(dir + "research.md", "utf8");
const all = spec + plan + tasks + research;

let fail = 0;
const check = (label, ok, detail) => {
  if (!ok) fail++;
  console.log((ok ? "  OK   " : "  FALLA") + "  " + label + (detail ? "  -> " + detail : ""));
};

console.log("--- integridad de tasks.md ---");
const defined = [...tasks.matchAll(/^- \[ \] (T\d{3})/gm)].map((m) => m[1]);
const nums = defined.map((x) => parseInt(x.slice(1), 10)).sort((a, b) => a - b);
const gaps = [];
for (let i = 1; i <= Math.max(...nums); i++) if (!nums.includes(i)) gaps.push(i);
check("numeracion contigua 1.." + Math.max(...nums), gaps.length === 0, gaps.join(",") || "sin huecos");
check("sin duplicados", defined.length === new Set(defined).size);
check("total tareas = 78", defined.length === 78, String(defined.length));

const verif = ["T071","T072","T073","T074","T076"];
const declared = verif.filter((v) => tasks.includes("| " + v + " |"));
check("tareas de verificacion declaradas", declared.length === verif.length, declared.length + "/" + verif.length);

console.log("--- trazabilidad spec -> plan -> tasks ---");
const reqs = [...new Set([...spec.matchAll(/\*\*(FR-\d+|NFR-\d+)\*\*/g)].map((m) => m[1]))];
const missPlan = reqs.filter((r) => !plan.includes(r));
const missTasks = reqs.filter((r) => !tasks.includes(r));
check("requisitos: " + reqs.length, missPlan.length === 0, missPlan.join(",") || "todos en plan");
check("requisitos en tasks.md", missTasks.length === 0, missTasks.join(",") || "todos en tasks");

console.log("--- Article III: test antes que implementacion ---");
const lines = tasks.split("\n").filter((l) => l.startsWith("- [ ] T"));
const order = (id) => lines.findIndex((l) => l.includes(id + " "));
const tests = lines.filter((l) => /\bTest:/.test(l)).map((l) => l.match(/(T\d{3})/)[1]);
const impls = lines.filter((l) => /\bImplement:/.test(l));
const orphan = tests.filter((t) => !verif.includes(t) && !impls.some((l) => order(l.match(/(T\d{3})/)[1]) > order(t)));
check("todo test tiene implementacion posterior", orphan.length === 0, orphan.join(",") || "0 huerfanos (5 de verificacion excluidas y declaradas)");

let bad = [];
for (const im of impls) {
  const id = im.match(/(T\d{3})/)[1];
  const m = im.match(/Makes (T[\d,\s-]+) pass/);
  if (!m) continue;
  for (const ref of m[1].split(",")) {
    const r = ref.trim().match(/^T(\d{3})-T(\d{3})$/);
    const ids = r
      ? Array.from({ length: +r[2] - +r[1] + 1 }, (_, k) => "T" + String(+r[1] + k).padStart(3, "0"))
      : [ref.trim()];
    for (const x of ids) {
      if (!tests.includes(x)) bad.push(id + "->" + x + " no es test");
      else if (order(x) > order(id)) bad.push(id + "->" + x + " test posterior");
    }
  }
}
check("referencias 'Makes Txxx pass'", bad.length === 0, bad.join("; ") || "coherentes");

console.log("--- referencias de fase ---");
const phaseSection = (plan.split("## Implementation Phases")[1] || "").split("## Technical Approach")[0];
const rows = [...phaseSection.matchAll(/^\| (0|1a|1b|1c|1d|1e|1f|2|3|4|5) \|/gm)].map((m) => m[1]);
const refs = [...new Set([...plan.matchAll(/Phase (\d+[a-f]?)/g)].map((m) => m[1]))];
check("tabla de fases completa (" + rows.length + ")", rows.length === 11, rows.join(","));
check("sin referencias colgantes", refs.every((r) => rows.includes(r)), refs.filter((r) => !rows.includes(r)).join(",") || "resueltas");

console.log("--- Article VI: claims de performance ---");
// The review section quotes the phrases it removed, and the amendment log names
// them. Scan the technical prose only, and drop anything inside quotes.
const scan = plan.split("## Constitutional Review Outcome")[0] + research;
const unquoted = scan.replace(/"[^"]*"/g, '""');
const banned = /measurably higher|becomes faster rather than|is measurably|faster rather than a regression/;
check("ningun claim de perf sin medir", !banned.test(unquoted), banned.test(unquoted) ? "encontrado" : "0 coincidencias");
check("expectativas cualificadas como tales", /is expected to have/.test(research) && /not claimed/.test(all), "marcadas como expectation");

console.log("--- decision audit ---");
check("Q5 ya no dice in-memory", !/In-memory for this feature/.test(spec));
check("FR-027 y FR-028 en spec", /FR-027/.test(spec) && /FR-028/.test(spec));
const frOrder = [...spec.matchAll(/\*\*(FR-\d+)\*\*/g)].map((m) => m[1]);
check("FR ordenadas y FR-027/028 al final",
  JSON.stringify(frOrder) === JSON.stringify([...frOrder].sort((a, b) => +a.slice(3) - +b.slice(3))) &&
  frOrder[frOrder.length - 2] === "FR-027" && frOrder[frOrder.length - 1] === "FR-028",
  frOrder.slice(-3).join(","));
check("plan justifica la correccion", /Corrected during review/.test(plan));
check("audit --clear en la CLI", /audit <repo-path> --clear/.test(plan));
check("plan nombra el coste de la escritura", /not free/.test(plan));
check("FR-028 obliga a medir con la escritura", /FR-028/.test(tasks) && /in place/.test(tasks));
check("T053 prueba el proceso separado", /readable by a \*\*second/.test(tasks));
check("T068 prueba audit end-to-end", /fresh/.test(tasks));
check("T022 sin red (promisor local)", /local path/.test(tasks));
check("reftable skipped-with-reason", /skipped-with-reason/.test(tasks));

console.log("--- ASCII ---");
check("todo en ASCII puro", !/[\x80-\xFF]/.test(all));

console.log("");
console.log(fail === 0 ? "TODAS LAS COMPROBACIONES PASAN" : fail + " COMPROBACIONES FALLAN");
process.exit(fail === 0 ? 0 : 1);
