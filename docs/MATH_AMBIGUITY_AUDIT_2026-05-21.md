# Math-parser ambiguity audit

> Originally a 2026-05-16 research sweep across two math-heavy papers
> (`1911.09517`, `2112.10748`) to identify grammar-ambiguity hotspots.
> Status (2026-05-18): patterns 1/3/4 closed during the 2026-05-16/17
> ambiguity sprint; pattern 2 (VERTBAR-modulus pairing) remains open.
> The original per-equation profiling tables were removed during the
> 2026-05-18 doc compaction; the patterns and proposed fixes survive
> below.

## Method (for reproducing the audit)

```bash
LATEXML_GRAPHICS_CACHE_OFF=1 LATEXML_PARSE_AUDIT=1 \
  ./target/release/cortex_worker --standalone \
    --input <math-heavy-paper>.zip \
    --output /tmp/audit.zip \
  > /tmp/audit.log 2>&1

grep PARSE_AUDIT: /tmp/audit.log | sort -rn -k1 | head -20
```

`PARSE_AUDIT` lines are emitted only for equations with >1 tree OR
>50 ms wall, so the noise floor is automatically pruned.

## Hot patterns

### Pattern 1 — `\log^+ <bigop> <expression>` — **closed**

Original witness: `\log^+ \int_{D(0,r)} \left| A_p(z) \right|^{1/(n-p)} dm(z)`
produced 5000 trees, none of which survived semantic pruning.
53 equations of the pattern in `1911.09517`; five hit the cap.

Closed via the 2026-05-16 afternoon fix: removed OPFUNCTION from
`factor` in `latexml_math_parser/src/grammar/builder.rs` (so
`\log^+` cannot anchor an implicit-times chain on the left),
plus a new `tight_term opfunction → apply_invisible_times` rule
for trailing-OPFUNCTION cases and re-instated `applied_func +=
opfunction opfunction → prefix_apply` so `FGH` chains still
cascade.

Impact on `1911.09517`: `math_parse` 7.71 s → 7.28 s, 5 → 3
remaining 5000-cap failures (the 3 became pattern #2 territory).

### Pattern 2 — VERTBAR-modulus + `||·||` norms — **open**

Witness (`2112.10748`):

```
VERTBAR:|   UNKNOWN:v   OPEN:(   UNKNOWN:x   CLOSE:)   VERTBAR:|
RELOP:less-than-or-equals
VERTBAR:|   UNKNOWN:v   OPEN:(   UNKNOWN:x'   CLOSE:)   VERTBAR:|
```

48–58 alternatives per such inequality, even though structurally
there's only one reading. Cause: the lexer cannot pair `|…|`
brackets without semantic input; the grammar admits `VERTBAR`
both as `divides` (relational) and `modulus` (delimiter), so
each `|` site doubles parses.

Partial mitigation (2026-05-17, landed): the lexer now emits
`STRETCHY_VERTBAR:|:idx` for `\left|…\right|` pairs (DOM
`stretchy="true"` hint), which the grammar can pair
unambiguously via `stretchy_vertbar expression stretchy_vertbar
→ fenced`. That handles the `\left|…\right|` family.

Still open: bare `|x|` modulus (no `\left/\right`) — the
hint isn't available there. A balanced-pair preprocessor that
consumes `|x|` triples and emits paired open/close VERTBAR
tokens (analogous to how braces are paired) would close
this. Where pairing is ambiguous (e.g. `|x| + |y|`), fall
back to today's bidirectional rule. This is a **new lexer
phase**, not a grammar rule.

### Pattern 3 — `\sin^2 x`, `\cos^{-1} x`, `\log_2 x` — well-tuned

Already handled by `scripted_trigfunction` /
`scripted_opfunction` in `grammar/builder.rs` (~lines 636-659).
1–3 trees per occurrence. No action needed.

### Pattern 4 — Subscripted variables under product — bounded

Pattern: `f_{0,1} \cdot f_{1,1} \cdots f_{p,1}` (long products
of indexed quantities) routinely produces 40-80 alternatives per
equation. The grammar correctly admits all reorderings of the
implicit-multiplication chain; semantic dedup collapses them to
6-10 unique trees. Bounded cost. Mentioned for completeness.

## WIDE_PUNCT — additional lexer hint (2026-05-17, landed)

`PUNCT` with `rpadding ≥ 5pt` (from `\quad`/`\qquad` spacing) is
the arXiv idiom for "main formula, side condition". The lexer
re-emits as `WIDE_PUNCT:,:idx`; the grammar admits it in BOTH
the `formulae` and `statements` alternations. Pragmas
(`list_apply: both relational` / `formulae_apply: no
relational`) decide which interpretation survives based on the
items.

Cumulative `math_parse` improvement on `1911.09517` across the
patterns landed: 7.7 s → 5.15 s (4 remaining 5000-cap
equations, all standalone-passing but failing in full-paper
context — Marpa recognizer-state persistence across formulae,
parser.rs `Avoiding reset_engine` note).

## Open next steps

1. Pattern 2: design and benchmark the bare-`|x|` pre-lexer
   pairing pass.
2. The 4 paper-context-only failures: choose between (a)
   force-reset before each parse (~8% CPU cost) for
   determinism, or (b) further grammar tightening around bare
   `|x|`. (a) is the cheap fix; (b) is the principled one.
3. Wire the audit into the canvas perf workflow
   (`tools/run_perf_corpus.sh`) so regressions in ambiguity
   counts surface alongside wall-time regressions.
