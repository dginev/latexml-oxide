# Math-parser ambiguity audit — 2026-05-16

Diagnostic study of where the Marpa grammar produces excessive parses
on math-heavy papers. Driven by Win #4 of the 2026-05-16 performance
sprint (see `docs/PERFORMANCE.md`). **This document is research-only**;
no grammar edits land here. The findings are a punch list for a
follow-up math-parser sweep.

## Method

Two math-heavy round22 papers were re-converted with the standard
release binary and `LATEXML_PARSE_AUDIT=1` to capture per-equation
parse statistics:

* `1911.09517` — 117 equations, complex-analysis (Wiman–Valiron
  theory of higher-order linear ODEs), single-thread `math_parse`
  cost ~7.9s.
* `2112.10748` — 276 equations, kinetic theory + nonlinear analysis,
  single-thread `math_parse` cost ~8.2s.

The audit emits one line per non-trivial equation, of the form

```
PARSE_AUDIT: N trees (X ok, Y pruned, Z dedup→U unique) in T | <lexer tokens…>
```

`N` is the raw Earley enumeration count, capped at 5000.

Aggregated 7.61s of `math_parse` time across both papers. Top
hotspots sort cleanly into a small number of patterns.

## Hot patterns, ranked by aggregate cost

### 1. `\log^+ <bigop> <expression>` (5000-tree truncation) — ~5s

**Witness equation shape**:

```latex
\log^+ \int_{D(0,r)} \left| A_p(z) \right|^{\frac{1}{n-p}} dm(z)
```

The lexer emits roughly:

```
OPFUNCTION:logarithm   start_POSTSUPERSCRIPT ADDOP:plus end_POSTSUPERSCRIPT
INTOP:integral         start_BIGOPSUB ATOM:D(0,r) end_BIGOPSUB
VERTBAR:|              UNKNOWN:A   start_POSTSUBSCRIPT UNKNOWN:p end_POSTSUBSCRIPT
VERTBAR:|              start_POSTSUPERSCRIPT … end_POSTSUPERSCRIPT
ATOM:dm(z)
```

The parser produces **5000 trees**, none of which survive the
semantic prune. The equation emits no `<XMath>`.

`1911.09517` has **53 equations** containing the `\log^+` pattern.
**Five** of them combine `\log^+` with an integral + modulus + outer
superscript and hit the 5000-tree cap. Each of those five costs
~0.9–1.0 s of pure Earley enumeration.

**Why the explosion?** Three interacting brackets:

1. `\log^+` could greedily absorb the entire integral expression as
   its argument, OR it could only consume the integral itself, with
   the modulus + superscript binding as a separate factor in an
   outer product.
2. The integrand `|A_p|^{1/(n-p)} dm(z)` can bracket as
   `(|A_p|^{1/(n-p)}) · dm(z)` (modulus then power, applied to
   `dm(z)`) OR as `|A_p|^{(1/(n-p)) · dm(z)}` (modulus, full
   superscript) — modulus closing position is ambiguous.
3. The bigop integral's subscript+superscript form makes both
   "as bigop" and "as factor with scripts" trees viable.

**Proposed fix candidates** (no edits yet — needs grammar review):

* **Idiom rule for `\log^+` / `\log^-` / `\log^{\pm}`**: pin the
  superscript-with-an-ADDOP-payload to `OPFUNCTION` greedily,
  emitting a single atomic `LIMITOP/OPFUNCTION` token at the lexer
  layer. This is analogous to how `bigop` already gets specialised
  script tokens (`start_BIGOPSUB`, `start_BIGOPSUP`) to reduce
  competition. Witness: `tex_math::lexicalize_postscripts` already
  has the apparatus for this kind of token-specialisation.
* **Modulus delimiter pairing**: `VERTBAR:|` opens/closes ambiguity
  is the most common shared symptom (see pattern #2 below); a
  greedy left-to-right pairing rule with stricter mid-expression
  acceptance criteria would cut the explosion at the source.
* **Hard cap with diagnostic**: when an equation hits the 5000-tree
  cap with zero semantic OK trees, emit a single `<XMath>` shell
  with the source TeX as content, rather than producing no XMath
  at all. The current behaviour drops the equation entirely. This
  doesn't speed anything up but bounds the user-visible damage.

### 2. `VERTBAR:|` modulus + `||·||` norms — ~1.5s aggregate

Both papers have this. Witness from `2112.10748`:

```
VERTBAR:|   UNKNOWN:v   OPEN:(   UNKNOWN:x   CLOSE:)   VERTBAR:|
RELOP:less-than-or-equals
VERTBAR:|   UNKNOWN:v   OPEN:(   UNKNOWN:x'   CLOSE:)   VERTBAR:|
```

48-58 alternatives per such inequality, even though structurally
there's only one reading. The cause is the lexer's inability to pair
`|…|` brackets without consulting the semantic layer; the grammar
allows `VERTBAR` to appear as both `divides` (relational) and
`modulus` (delimiter), so each `|` site doubles parses.

**Proposed fix**: a balanced-pair preprocessor (similar to how
braces are paired by the lexer) that consumes `|x|` triples and
emits paired open/close VERTBAR tokens. Where pairing is ambiguous
(e.g. `|x| + |y|`), fall back to today's bidirectional rule. This
needs a new lexer phase, not a grammar rule.

### 3. `\sin^2 x`, `\cos^{-1} x`, `\log_2 x` — minor — ~0.3s

These are already handled by `scripted_trigfunction` /
`scripted_opfunction` in `grammar/builder.rs` (lines 636-659).
Audit shows them parsing in 1–3 trees per occurrence — well-tuned.
Mentioned for completeness, no action needed.

### 4. Subscripted variables under product (UNKNOWN ATOM·ATOM) — minor — ~0.6s

Pattern: `f_{0,1} \cdot f_{1,1} \cdots f_{p,1}` (long products of
indexed quantities) routinely produce 40-80 alternatives per
equation. The grammar correctly admits all reorderings of the
implicit-multiplication chain; semantic dedup collapses them to 6-10
unique trees. Bounded cost. Mentioned for completeness.

## Cumulative impact estimate

If pattern #1 (the `\log^+ ∫ ...` 5000-tree cap) were closed:

* `1911.09517` `math_parse` 7.9s → ~3s (~60% reduction).
* `2112.10748` would benefit less (no `\log^+` idiom in its corpus,
  but pattern #2 fixes would apply).

Across the corpus (`PERFORMANCE.md §4` already calls out the math
parser as 7-10% of corpus wall for math-heavy classes), I'd budget
3–5% off corpus wall as the upper bound of math-parser-only fixes.
Most papers are math-light and won't move.

## Reproducing this audit

```bash
LATEXML_GRAPHICS_CACHE_OFF=1 LATEXML_PARSE_AUDIT=1 \
  ./target/release/cortex_worker --standalone \
    --input ~/round22_validate/inputs_oos/1911.09517.zip \
    --output /tmp/audit.zip \
  > /tmp/audit.log 2>&1

grep PARSE_AUDIT: /tmp/audit.log | sort -rn -k1 | head -20
```

`PARSE_AUDIT` lines are emitted only for equations with >1 tree OR
>50 ms wall, so the noise floor is automatically pruned.

## Next steps (handoff)

1. Verify pattern #1's `\log^+` token-specialisation idea on a
   single witness equation. Compare parse counts pre/post.
2. Design the VERTBAR-pairing pre-lexer pass and benchmark on the
   subset of `2112.10748` equations that match pattern #2.
3. Re-run the audit after each change to confirm cumulative impact.
4. Wire the audit into the canvas perf workflow (`tools/run_perf_corpus.sh`)
   so regressions in ambiguity counts surface alongside wall-time
   regressions.
