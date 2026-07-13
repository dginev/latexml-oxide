# Math-parser over-parse deep dive (2026-06-30)

> Point-in-time deep dive on the Marpa grammar's ambiguity/over-parse cost —
> the top remaining `math_parse` lever (17% of corpus wall, 17% over-parse).
> All and-node counts **freshly measured on the release binary** (method below),
> which **corrects stale claims** in `archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md`.
> Companion to [`PERFORMANCE.md`](../performance/PERFORMANCE.md) §P1-math and
> [`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md).

## Method

Each formula isolated in its own `.tex`; run with
`LATEXML_MARPA_HYBRID_AND_NODE_LIMIT=1 LATEXML_MARPA_ASF_AUDIT=1` (forces every
`metric≥2` formula through the fallback so its bocage `and_nodes` always prints).
`and_nodes` = Marpa bocage AND-node count = the real cost signal; routing sends
`>500` and-nodes to the slow legacy Tree-iter path (`parser.rs`
`HYBRID_AND_NODE_LIMIT`).

## Measured — the docs were stale

| Formula | and-nodes | note |
|---|---:|---|
| `a+b=c`, `x`, `\sum_i x_i`, `\int_0^1 y` (no `dx`) | unamb | baseline |
| **`\Pi^N(p,q,r)`** | **unamb** | doc claimed 2762–3555 → **CLOSED** (scripted UNKNOWN blocks speculative apply) |
| **`|x| ≤ |y|`** (simple) | **unamb** | doc claimed 48–58 alts → **STALE** for the simple case |
| `dx` | 71 | ← a 2-token formula, pure diffop waste |
| `f(x)` | 112 | the dominant residual |
| `|v(x)|` | 159 | inner apply-ambiguity |
| `\int_0^1 x\,dx` | 432 | integral amplifies the differential ~4× |
| `\int_0^1 f(x)\,dx` | **523 (legacy fallback)** | differential × apply |
| `|v(x)| ≤ |v(x')|` | **625 (legacy)** | bar-pairing × inner apply × prime |
| `\left|v(x)\right| ≤ \left|v(x')\right|` | **707 (legacy)** | STRETCHY pairing fixed *pairing*, not the inner explosion |

**Two independent drivers, cross-multiplied by enclosing constructs:**
- **(a) the differential `d`** — the lexer emitted `XDIFFUNK`/`XDIFFID` (the
  diffop-competing terminal) for *every* `d`; `\int_0^1 y` (no `dx`) is unambiguous
  but adding `dx` explodes it. Integrals are ubiquitous → highest volume.
- **(b) `f(x)` apply-vs-multiply** — `fx` unambiguous, `f(x)` = 112; the parens
  alone create it; compounds (`h(x)=f(x)g(x)` = 394).

The VERTBAR explosion (625/707) is (a)+(b) stacked around a relation.

## Lever 1 — differential-`d` lexer gating — ✅ LANDED (output-neutral)

Root cause: `diffunk`/`diffid` differ from plain `unknown`/`id` in exactly ONE
grammar rule — `diffunk factor_base => diffop_apply` (`grammar/builder.rs`) — and
`diffop_apply` (`semantics.rs`) prunes EVERY diffop parse unless an `INTOP` is in
the formula. So for an INTOP-free formula the diffop branch is dead weight Marpa
still builds (~71 and-nodes per `d<var>`).

**Fix** (`util.rs::node_to_grammar_lexemes_from`): after flattening, if no `INTOP`
node is present, downgrade `XDIFFUNK→UNKNOWN` / `XDIFFID→ID`. Same `has_intop`
predicate `diffop_apply` uses, over the same node list that becomes `ctxt.nodes`,
so it is **provably byte-identical** to the action-time pruning — it just never
builds the branch. Verified: byte-identical HTML on a differential test doc; the
non-integral `d` cases (`dx`, `dt`, `d\mu`, …) drop from 71–76-and-node fallbacks
to the unambiguous fast path; integral cases (432/523) unchanged. Suite 1503/0.

**Step 2 (open):** emit a dedicated in-integral `DIFFOP_D` terminal so `∫(x·d·x)`
is never built — collapses `\int … dx` (432) and pulls `\int … f(x)\,dx` (523) and
`\int_a^b … = G(b)-G(a)` (887) OFF the legacy fallback path. Low-medium risk.

## Open levers (ranked)

1. **`f(·)` apply-vs-multiply** (biggest aggregate over-parse). `speculative_prefix_apply`
   (`semantics.rs`) no longer checks the `MATHPARSER_SPECULATE` state flag, so the
   speculative apply is effectively always-on — every `f(...)` doubles the bocage
   and `f(x)`→apply by default, contradicting the grammar comment at
   `builder.rs` ("Only active when MATHPARSER_SPECULATE is set"). **Likely a latent
   regression, HIGH parity risk:** confirm `f(x)` apply-vs-times against same-host
   Perl (default profile) BEFORE touching — whichever way it resolves changes a huge
   number of outputs. A lexer "UNKNOWN-immediately-followed-by-`(`" hint that lets a
   single apply rule fire (keeping the apply output) is the safer route if Perl reads
   apply.
2. **Lever 1 Step 2** (differential in-integral terminal) — above; the integral
   cases are the largest current volume on the legacy fallback.
3. **Bare-`|x|` balanced-pair pre-lexer pass** — a new lexer phase (peer of the
   `STRETCHY_VERTBAR` side-tagging) that pairs `|…|` where unambiguous, removing the
   bar-pairing factor from the 625/707 modulus fallbacks. MEDIUM risk (must not break
   the genuinely-bidirectional cases: `a|a|+b|b|`, conditional `(a|b)`, eval-at
   `f|_{x=0}`, set-builder `{x|P}`, Dirac bra/ket).
4. **Raise `HYBRID_AND_NODE_LIMIT` / fix the ASF allocation cliff** — the common
   integrals (510–887) and moduli (625–707) sit *just over* 500 and pay the slow
   legacy Tree-iter path instead of the multiplicatively-cheaper ASF traversal. Best
   done AFTER 1/2/3 shrink most bocages, paired with a marpa-side ASF alloc fix
   (no-cap ASF historically OOM'd 19/100).

## MathML-quality note

**The hot cases are quality-CORRECT** — `f(x,y)` → `f ⁡ (x,y)` (apply inserted),
`\int_0^1 x\,dx` → `∫ x⁢(𝑑x)` (differential-d), `|v(x)| ≤ |v(x')|` → correct nested
abs-value fences. So Levers 1–4 are **pure speedups with no output change** (the
cleanest possible win for the rerun), NOT quality fixes. The one quality item to
verify is the `f(x)`-as-apply-by-default question entangled with Lever 1 above.

Separately noted (faithfulness, not perf): `\iint`/`\iiint` are tagged **ATOM, not
INTOP**, so their `dx` is not recognized as a differential — check against Perl.
