# RESOLVED: `malformed:ltx:text` in 30_script_bindings was a bad specimen + logger-order, NOT contamination

## Final root cause (2026-06-10)

The `\numbered` e2e specimen used template
`<ltx:text class="eq">#tags #1</ltx:text>`. `#tags` (from `RefStepCounter`)
is an `ltx:tags` — a BLOCK element that cannot live in an inline `ltx:text`.
Absorbing it makes the Document VALIDLY auto-close `ltx:text`→`ltx:p`→
`ltx:para` to host the block (observed: "closing ltx:para to insert
ltx:tags"). The template's trailing `</ltx:text>` then has no open text →
a **benign, output-neutral** malformed close (tree recovers; XML identical).

It looked like a second-conversion contamination only because of LOGGER
ORDER: the condition fires on EVERY run (confirmed: the isolated macro test
also hits the NOTFOUND branch), but `Error!` only PRINTS once the logger is
initialized — which the alphabetically-first `discovered` test does via
`logger::init`. Run alone, the macro test's logger is uninitialised so the
same error is silently dropped.

## Why the earlier evidence misled

- Parallel reproduced (process-global libxml) — TRUE but irrelevant: the
  shared thing was the process-global LOGGER init, not document/model state.
- can_contain / auto-close decisions were byte-identical clean vs dirty
  (correctly — the model was never corrupted).
- The decisive step: a global atomic seq counter showed the isolated clean
  run ALSO emitted `CE#8 NOTFOUND`; only the `Error!` print differed.

## Fix (landed)

1. `\numbered` renders the refnum as a STRING property (`#refnum` =
   `CounterValue` after `RefStepCounter` steps it), not the block `#tags`
   inline — well-formed, still exercises RefStepCounter-as-properties.
2. `30_script_bindings` now binds its own log (`logger::init` + `bind_log`)
   and asserts the conversion logs no `Error:`/`Fatal:` — order-independent,
   and pins the malformed-close class so any future invalid specimen fails.

## Standing lesson

A template that renders a block-level `#prop`/`#n` hole inside an inline
element produces a benign-but-logged malformed close. Specimens (and real
bindings) must place block holes in block contexts.
