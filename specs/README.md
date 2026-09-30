# specs/

One directory per feature. Created by `/specify`.

```
specs/
  001-<feature-name>/
    spec.md          WHAT and WHY, no HOW.  Created by /specify.
    plan.md          HOW, constitution-checked.  Created by /plan.
    research.md      evidence behind a technical choice.  Optional.
    data-model.md    entities and their relations.  Optional.
    contracts/       interface definitions.  Optional.
    implementation-details/
                     code samples, kept out of plan.md.  Optional.
    tasks.md         ordered, test-first, parallel waves.  Created by /tasks.
```

## Status

No features specified yet. First feature goes in `specs/001-`.

## The harness validates nothing by example

Every competitive claim in a spec's differentiation table should eventually be
backed by a test. A claim with no test is a marketing claim, not a product
guarantee. Track that gap deliberately - it is the honest measure of whether we
are actually beating the incumbents or only intending to.
