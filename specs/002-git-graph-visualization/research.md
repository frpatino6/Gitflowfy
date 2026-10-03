# Research: Graph Visualization Stack Decision

La evidencia detrs de las decisiones de stack en `plan.md`.

## Pregunta

Rust builder + TypeScript renderer en Tauri v2, vs otras opciones.

## Lo que el spike ya prob

`spike/` es TypeScript/JS y est validado contra 599,555 commits de `llvm/llvm-project` (partial clone, Intel HD 4600):

| Mtrica | Resultado |
| --- | --- |
| Extract y build | 9.7 s |
| Graph binary | 9.6 MB |
| Renderer load | 522 ms |
| Frame p50 | 16.7 ms |
| Frame p99 | 19.5 ms |
| Peor frame | 20.2 ms |

**Todos los presupuestos Article VI se cumplen con margen.** Tirar esto no es gratis.

## De qu est hecho el spike

Leyendo `spike/build-graph.mjs`:

1. `spawn git log --all --topo-order --format=...` con separador `0x1f`
2. Stream stdout, split lneas, llena `oidByRow`, `subjectByRow`, `parentOidFlat`
3. Resuelve parent OIDs a row indices en `Uint32Array`
4. Asigna lanes en `Int32Array` con free-lane pool
6. Empaqueta 5 typed arrays en un `ArrayBuffer` y escribe `graph.bin`

El renderer lee ese blob directamente. El diseo es layout CSR en typed arrays, y el binario es el contrato entre las dos mitades.

Dos observaciones siguen:

- **La mitad renderer es tecnologa web y se queda en web.** Canvas + typed arrays produjo 19.5ms p99. Tauri hostea webview, as que esta mitad sobrevive sin modificacin.
- **La mitad builder es proceso-y-parse**, que es la parte que el core eventualmente posee en Rust. Portear es mecnico: mismo algoritmo, mismos 5 arrays, mismo formato binario. Rust streamea bytes sin allocation de string JS por lnea, as que 9.7s se vuelve ms rpido, no regresin.

As que el descarte es menor: un script ~160 lneas de la feature 002, no un renderer validado.

## La restriccin vinculante

SC-010 pone presupuesto 5ms overhead por invocacin sobre `git` crudo. Nuestro overhead domina el spawn, porque el runtime de Git es la referencia y se excluye.

- Rust `std::process::Command` en Windows: spawn sub-milisegundo, sin runtime tax.
- Node `child_process.spawn` en Windows: overhead mediblemente mayor por llamada, y el harness diferencial spawnea cientos de procesos por corrida, patrn que amplifica la diferencia.

El presupuesto es probablemente alcanzable en Node. "Probablemente" es el problema: es un compromiso que la spec ya hizo, y un stack que cumple un nmero publicado con margen delgado es un stack que falla ese nmero en mquina ms lenta.

Gate G2 mide esto **antes** de escribir cdigo, en la mquina objetivo. Eso es el punto de hacerlo gate: la decisin es falsificable, no asercin.

## Por qu no Electron

- Enva Chromium por instancia. Esa memoria compite con los 9.6 MB del graph y el render loop cuyos presupuestos defendemos.
- No da beneficio aqu que Tauri no d, y el costo de spawn es el mismo costo Node que el presupuesto preocupa.
- La tesis del producto es que no mantenemos modelo paralelo del repo. Un runtime de 150MB es mala expresin de "somos capa honesta y delgada sobre tu git".

## Por qu no todo Rust

Tentador, ya que el core es Rust. Rechazado porque el renderer validado es Canvas + typed arrays. Un renderer Rust significa o bridge webview o path GPU nativo, y ambos tiran el 19.5ms p99 medido por una reimplementacin que nadie ha validado. Mantener el webview es gratis; el costo es un lmite IPC, que Article IV requiere de todos modos.

## La debilidad honesta en esta decisin

**El builder del grafo se paga dos veces**: una como spike JS, otra como puerto Rust. Es trabajo duplicado real, y es el precio del presupuesto de spawn. Si gate G2 luego muestra que Node cumple 5ms cmodamente en hardware objetivo, el movimiento correcto es reconsiderar - un core en un solo lenguaje borrara el puerto, el lmite IPC, y reutilizara el spike directo.

Por eso el gate existe y mide **antes** de escribir cdigo. Esta decisin es condicional a un nmero, y el nmero no se ha tomado todava.

## Lmite de interop

Superficie de comandos Tauri. El grafo lo cruza como bytes crudos (`graph.bin`), no JSON, que es precisamente por qu el layout CSR es un blob binario empaquetado en lugar de un grafo de objetos - una decisin que el spike ya tom, y que sera errnea si el renderer no fuera webview.

Feature 001 no crea UI ni crate desktop. La decisin de renderer ata desde feature 002 en adelante.