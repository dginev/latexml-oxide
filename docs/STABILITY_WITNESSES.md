# Stability & Optimization Witnesses

> **Living worklist** (not a dated snapshot). Tracks specific arXiv papers that
> are valuable witnesses for *reliability* (timeout / OOM / panic / hang) and
> *performance* (wall-time, peak RSS) — as distinct from plain correctness
> errors (which live in `SYNC_STATUS.md`). Goal per user directive (2026-05-29):
> *"improve in ALL aspects … find root causes, develop careful best-practice
> solutions, be faithful to the original Perl LaTeXML."*
>
> Re-measure with the **current `--release` binary** before acting — sweep
> failure records are often stale. Classify each witness Rust-only vs SHARED by
> gating against Perl (`--path=$HOME/git/ar5iv-bindings/bindings
> --preload=ar5iv.sty`), measuring **both** wall-time and peak RSS
> (`/usr/bin/time -v`).

## Cluster A — peak RSS in post-processing on large-math documents (PRIMARY)

**Symptom.** Documents with thousands of math elements complete correctly
single-threaded but consume **1.4–3 GB peak RSS** during post-processing
(MathML[Presentation] + MathML[Content] generation, then XSLT). Under the
parallel canvas sweep (8–20 workers × 2–3 GB), this exhausts RAM → the worker is
OOM-killed (recorded as `FATAL_134` "out of memory") or exceeds the 120 s
worker timeout under memory pressure (recorded as `TIMEOUT`). The engine
(digestion) itself finishes cleanly — the cost is entirely post-processing.

**Witnesses** (current release binary, `--timeout 0`, single-threaded):

| Paper | maths | wall | peak RSS | status | sweep record |
|-------|------:|-----:|---------:|--------|--------------|
| 1901.10171 | 18829 | 49 s | **3.06 GB** | clean | stage_77 TIMEOUT |
| 1906.06650 |  3751 | 68 s | **2.89 GB** | clean (143 warn) | stage_80 TIMEOUT |
| 1905.00087 |  5297 | 57 s | **2.54 GB** | clean (1 warn) | stage_79 TIMEOUT |
| 1810.11713 |  4389 | 51 s | **2.39 GB** | clean | stage_75 TIMEOUT |
| 1902.03551 |  6122 | 36 s | 1.42 GB | clean (311 warn) | stage_77 TIMEOUT |
| 1902.05175 |  3870 | 20 s | 2.90 GB | clean (now) | stage_78 FATAL_134 (was OOM) |

**Root-cause hypotheses (to confirm — needs Perl RSS baseline, in flight):**
1. **XSLT input duplication.** `latexml_post/src/xslt.rs:286` does
   `doc.get_document().dup()` (`xmlCopyDoc`, full DOM deep-copy) before
   `transform`, because the libxslt-crate `transform()` consumes its source by
   value. On a multi-GB DOM this transiently *doubles* peak RSS. Perl's
   `LaTeXML::Post::XSLT` (`XSLT.pm:79`) passes `$doc->getDocument` directly to
   `transform` — no pre-copy. **Candidate fix:** avoid the deep dup (transform a
   moved/borrowed handle, or free the source tree before the result is
   serialized). Must stay faithful: Perl keeps the original doc alive only
   because libxslt copies internally; verify the crate's ownership model first.
2. **MathML duplication.** Both pMML and cMML are generated for every math
   (XMDual content+presentation), so a 18829-math doc holds ~2× the math node
   count of the source. Check whether Perl prunes/shares more aggressively, or
   whether we retain the parsed XMath alongside both MathML branches.
3. **`xsltMaxDepth` not set.** Perl sets `XML::LibXSLT->max_depth(1000)`
   (`XSLT.pm:48`); our binding leaves libxslt's default (3000). This is about
   recursion *depth*, not breadth RSS, so it won't fix this cluster — **but it
   is a faithful, independent stability port** (prevents runaway-recursion OOM
   on pathological nesting where Perl aborts gracefully). `libxslt-0.1.3`
   exposes `xsltMaxDepth` as a raw mutable static (`bindings.rs:15`); set it
   once alongside `register_exslt()`.

**Decision rule.** If Perl's peak RSS on these is ≪ Rust's, it's a Rust-only
memory bug → fix hypotheses 1/2. If Perl is comparable (2–3 GB), the docs are
inherently heavy → the OOM is sweep RAM-contention, addressed by worker-count /
per-worker-RSS-cap config (not an engine bug), though hypothesis 1 still helps.

### RESOLVED 2026-05-29 — memory is SHARED-inherent; Rust massively faster

Perl baseline (`/usr/bin/time -v`, same main, 600 s cap):

| Paper | Rust time / RSS | Perl time / RSS | verdict |
|-------|-----------------|-----------------|---------|
| 1810.11713 | 51 s / 2.39 GB | **>600 s (TIMEOUT) / 2.03 GB** | Rust >11× faster; RSS comparable |
| 1902.03551 | 36 s / 1.42 GB | 319 s / **1.43 GB** | Rust ~9× faster; RSS **identical** |

So the peak RSS (~2–2.4 GB) is **SHARED / inherent to the document size** (an
18829-math DOM held as source + pMML + cMML + HTML result), NOT a Rust-only
bug — Perl holds a comparable tree and is **far slower** (couldn't finish
1810.11713 in 10 minutes; Rust did it in 51 s). **The sweep `TIMEOUT`/`FATAL_134`
records on this cluster are therefore RAM-contention / wall-budget artifacts of
running many ~2.4 GB papers concurrently under a 120 s cap — not an engine
defect.** Engine verdict: healthy and surpassing Perl on these.

**Landed (faithful, verified `cargo test` rc=0, identical output):**
- **`xsltMaxDepth = 1000`** (hypothesis 3) — faithful port of Perl
  `XML::LibXSLT->max_depth(1000)`; graceful abort instead of stack-overflow OOM
  on pathological recursion. `latexml_post/src/xslt.rs`.
- **`dup()` → Rc `clone()`** (hypothesis 1) — drops the transform-time deep DOM
  copy; measured **−120–130 MB/paper** (3.06→2.93, 2.89→2.76, 2.54→2.42 GB) with
  byte-identical output. `latexml_post/src/xslt.rs`.

**Remaining (optional, would SURPASS Perl — not a parity gap):** hypothesis 2
(pMML+cMML duplication) is the bulk of the ~2.4 GB. Perl also keeps both
branches, so trimming it is a beyond-Perl optimization, not a bug fix; defer
unless the heavy-doc OOM tail justifies it. The operational mitigation for the
sweep is per-worker RSS budgeting / fewer concurrent workers on the heavy tail
(see [[feedback_worker_sweep_parallelism]]).

**Concrete next-step (needs a focused session + real heap profiling).** Don't
guess at the 2.4 GB — measure it. Recommended: run one witness (e.g.
1902.05175, 3870 maths, 2.9 GB) under `heaptrack` (or valgrind massif) on the
release binary, and read the peak-RSS allocation tree. Likely suspects to
confirm/refute, in order: (a) the pre-XSLT document still carrying all source
`ltx:XMath` trees alongside both MathML branches (3× math node count fed to
XSLT) — check whether our XMath unlink (`latexml_post/src/mathml/mod.rs:1213`)
fires for every math and matches Perl's keep/drop policy (Perl associates the
generated node with the source XMath but the default non-parallel path does not
retain XMath in the serialized HTML); (b) the core `arena` string interner
retaining every interned string for the whole run; (c) libxml DOM overhead per
node. Only after the profile identifies the dominant allocator should a fix be
attempted — and it must stay faithful (match Perl's XMath retention semantics,
not merely prune to save bytes).

## Cluster B — xy-pic via raw `\@@input xypic` (SHARED, not memory)

1810.09054, 1903.02279 were recorded as TIMEOUT but the **current** binary fails
them fast (~1.2 s, ~128 MB) in digestion with ~109 xy-pic errors
(`\xymatrix`/`\xyrequire`/`\lx@xy@*`/`\frm@*` undefined). These load xy via
`\csname@@input\endcsname xypic` — **SHARED**: Perl also fails (`\xyoption`/`\ar`
undefined + closed-mouth, see `SYNC_STATUS.md` 2026-05-29 re-mine). Not a memory
witness; tracked here only to explain the stale TIMEOUT records.

## Cluster C — engine-phase slowness (RESOLVED — not a hang)

1810.05230 (stage_75 TIMEOUT) was recorded hung in the **Building** (engine)
phase. Current release binary: **completes in 47 s / 0.81 GB / clean** (86
warnings). It was debug-profile + sweep-contention slowness under the 120 s cap,
not an engine hot loop. No fix needed.

## OOM witness 1902.05175 (RESOLVED — contention, not a bug)

The one fresh-stage `FATAL_134` (recorded "out of memory" during post/XSLT):
current release binary **completes in 19.8 s / 2.90 GB / clean**. The 2.90 GB is
the inherent large-doc peak (Cluster A); the OOM was parallel RAM-contention in
the sweep, not an engine defect.

**Net:** the entire fresh-stage (75-81) hard-fail bucket is either SHARED-heavy
(resource contention on inherently-large docs — Rust faster than Perl, comparable
RSS) or SHARED-error (xy-pic via `\@@input`). No genuine Rust-only engine defect
remains in it. Engine + post-processor verdict: healthy.

## Cluster D — custom plain-TeX `\line`/picture width-loop (SHARED hang; Rust aborts gracefully)

**Witnesses:** `math0102053`, `math0102089`, `math0212126` (all `canvas_3_failures_
sandbox/all_failures.txt`, originally classified OOM). All are plain-TeX papers
(`\magnification`, no documentclass) that **inline their own copy of the LaTeX
`picture`/`\line` code** under private names — `\droite`/`\@sline`/`\@whiledim`
(math0102053 L123-158). The diagonal-line routine `\@sline` draws a sloped line by
repeating a line-font glyph box:

```
\setbox\@linechar\hbox{\@linefnt\@getlinechar(\@xarg,\@yyarg)}%   % \@linefnt = linew10
\@whiledim \@clnwd <\@linelen \do {... \advance\@clnwd \wd\@linechar}%
```

The loop advances `\@clnwd` by `\wd\@linechar` each turn. **LaTeXML is not a
typesetter**: it does not compute real TFM box metrics for an `\hbox{\font <char>}`,
so `\wd\@linechar` is **0** → `\@clnwd` never grows → the `\@whiledim` loop never
terminates, appending boxes until memory is exhausted. (`linew10.tfm`/`line10.tfm`
DO exist in texmf, but neither engine reads glyph widths from them — this is a
shared architectural limit, not a missing-font issue. The
`Info:fontmap:line Couldn't find fontmap for 'line'` line is a downstream symptom.)

**SHARED, confirmed:** Perl `latexml` on math0102053 runs **unbounded** — measured
71 s → 107 s with RSS climbing 1.1 GB → 1.57 GB, still at the same `line 1405 col 7`,
no termination. **Rust is strictly better:** its `Fatal:Timeout:MemoryBudget` guard
aborts gracefully at RSS 4500 MB (rc=3, one fatal) instead of growing without bound.
This is the correct behavior for an unsatisfiable typesetting loop — neither engine
can render these custom pictures without real box metrics, and the standard LaTeXML
`\line` binding (which sidesteps the loop) is bypassed by the document's private
`\droite`. **Not a Rust-only defect; no parity fix.** A faithful "make it terminate"
fix would require giving `\font`-declared glyph boxes real TFM widths — a beyond-Perl
typesetter feature (Perl hangs identically), high-risk, deferred. The graceful abort
is the right floor.

(The other `all_failures.txt` records re-tested 2026-05-31 on the current binary:
3 `FATAL_139` segfaults → all clean rc=0 (stale transients); `math0104252`/
`math0203082`/`gr-qc0209055`/`gr-qc0301024` OOM/TIMEOUT → all clean rc=0 (stale);
`hep-ph0012156` (12,778 maths) → graceful OOM-abort under 6 GB ulimit, Cluster A
inherent-large-math. No genuine Rust-only defect in the batch.)

## Cluster E — tikz/pgf path-processing memory blowup (✅ FIXED 2026-06-20)

> **ROOT-CAUSED + FIXED 2026-06-20 (`pgfmath_code_tex.rs`, `\pgfmathsetlength`).**
> The blowup is a **non-terminating pgf decoration automaton** (`decorations.text`
> / `text along path`), not "pgf allocates more than Perl". The automaton walks the
> path consuming `width=+.5\wd\pgf@lib@dec@text@box` per state and terminates when a
> state's width exceeds the remaining distance (`switch if … to final`); the
> end-of-text trick sets the box to `\wd=16383pt` for a huge final advance. The move
> is applied via `\pgfmathsetlength\pgf@decorate@distancetomove{\pgf@decorate@width}`
> — where `\pgf@decorate@width` is a **macro** expanding to `+.5\wd\box`. Rust's
> native `\pgfmathsetlength` tested the **raw** first token for the `+` glue/native
> fast-path (which alone can read `\wd<box>`; pgfmath's expression parser returns 0
> for box registers, same as Perl & pdflatex). The raw token was the macro, not `+`,
> so it fell to pgfmath → `\wd`→0 → **move 0 → remaining-distance never decreased →
> infinite loop placing boxes → RSS runaway**. Fix: **expand the argument before the
> `+` test**, so the macro-delivered `+.5\wd\box` takes the native path (pdflatex
> ground truth: `\pgfmathsetlength\d{+.5\wd0}` = 3.75pt = .5×7.5pt). Confirmed
> against all four gate witnesses (no more `MemoryBudget`):
> 1709.07916 8.2s/272MB · 1912.13052 5.4s/759MB · 2004.14791 3.1s/490MB · 1312.6499
> 2.7s/304MB clean · 2110.08101 0.5s/171MB. Suite 1459/0, clippy clean. Minimal
> repros + diagnostic trace tooling in `~/scratch/{rss_1709,pgfmath_box}`. The
> residual per-paper errors (`\smartdiagramset`, `\thref`, `\weight`, …) are
> unrelated missing-macro issues, not the RSS cluster.

The diagnostic record below is retained for context.

**Witness:** `2110.08101` (third-batch canvas, `Fatal:Timeout:MemoryBudget RSS 4500 MB`).
**Differential (2026-06-08, current binary + release):** Perl **completes** (1 error) on the
same paper; Rust blows the 4500 MB RSS cap → RUST-ONLY. The blowup is while digesting
`FIG/Flow_Chart.tikz` (a `pgfcircflow` flowchart) at line 112 — a `\draw[-latex, rounded
corners=10pt] (block4) -- node{…} (com2) |- (block10);` path (the `|-` H-then-V path op).
No `\foreach`/loop in the file (121 lines), so it is not a loop explosion — Rust's pgf path/
coordinate machinery allocates far more than Perl (`arena:strings_allocated 220193` before the
cap). Only ~4 of ~37 `MemoryBudget` fatals are tikz-related (the rest are diverse "regular
`.tex`" blowups, sampled mostly SHARED), so this is a minority cluster. DEFERRED — deep pgf/tikz
internals; needs a focused profile of pgf path-op allocation vs Perl. Sibling of the pgfplots
`symbolic x coords` Rust-only case (SYNC_STATUS.md differential-sweep note).

**More witnesses (2026-06-20, from the 3-sandbox `PERL_VS_RUST_FATAL_ANALYSIS` gate — moved
here from the SYNC_STATUS correctness gate, as these are perf/RSS not correctness):**

| Paper | Perl | Rust | peak RSS | locus (from cortex.log) |
|---|---|---|---|---|
| 1709.07916 | ok   | Fatal MemoryBudget | 4500 MB (cap) | `tikzpicture` in a `figure`, pgfplots `compat` mode |
| 1912.13052 | warn | Fatal MemoryBudget | 4500 MB (cap) | pgf/tikz digestion |
| 2004.14791 | warn | Fatal MemoryBudget | 4500 MB (cap) | pgf/tikz digestion |
| 1312.6499  | warn | Fatal MemoryBudget | 4500 MB (cap) | pgf/tikz (was TokenLimit→MemoryBudget) |

All four hit the **deliberate 4500 MB per-worker RSS fuse** (a fleet-safety mechanism Perl
*lacks* — Perl uses unbounded RAM, so "Perl ok" partly reflects the absence of a cap, not lower
usage). Diagnostic tell on 1709.07916: `gullet:progress 7784` (LOW) with 4.4 GB RSS ⇒ **not** a
token-loop explosion but memory-heavy per-op pgfplots allocation (coordinate/path/plot data),
matching the 2110.08101 path-op profile above. They do **not** reduce to small repros (basic
pgfplots/tikz convert cleanly in both engines). **FIXED 2026-06-20** — see the banner at the
top of this cluster (the `\pgfmathsetlength` expand-before-`+` fix); all four now convert
without `MemoryBudget`. The earlier "deep pgf allocation vs Perl" hypothesis was wrong: it was a
single decoration-automaton non-termination bug.

## Cluster F — xint raw-load runaway native recursion → stack-overflow SIGABRT (✅ FIXED 2026-06-20 — no longer crashes)

> **FIXED 2026-06-20 (`gullet.rs`, `read_x_token` `Outcome::Invoke`).** Wrapped the
> per-expansion `defn.invoke(false)` call in the stack-growth guard (default
> 256 KiB red zone, 8 MiB segment) — the same idiom as the recursive tree walks in
> `document.rs` / the math parser. All three sites now go through the single
> configurable `latexml_core::stack_guard::maybe_grow` (params tunable via
> `LATEXML_STACK_RED_ZONE_BYTES` / `LATEXML_STACK_SEGMENT_BYTES`, or
> `stack_guard::set_*` for a future CLI flag). Every deep gullet-recursion cycle
> passes through this point (~every ≤10 frames, ≪ the red zone), so the native
> stack grows ahead of the recursion. The
> SIGABRT is **gone**: 1804.01117 now exits **124 (graceful wall-clock timeout)**
> instead of **134 (SIGABRT)** — it degrades gracefully like Perl (which fails-soft
> to an empty doc) rather than crashing the process. `maybe_grow` is *transparent*
> (it only provides more stack; it never changes results), so the full suite stays
> **1459/0** and a math-heavy perf spot-check is unchanged (calculus.tex 0.57s,
> aastex631_deluxetable.tex 0.69s). This is a *robustness* win, not a coverage one
> — the paper still doesn't convert (times out, as it does in Perl). A faster
> fail-soft (a Perl-`$MAXSTACK`-style depth-guard that *bails* instead of growing,
> so it doesn't spend the full timeout window) remains a possible future
> refinement, but the crash — the actual defect — is resolved.

The diagnostic record below is retained for context.

**Witness:** `1804.01117` (under the ar5iv profile / `INCLUDE_STYLES=true`, the
cortex path).

**Differential (2026-06-20, matched configs — Perl `--includestyles` ⇔ Rust
`--preload=ar5iv.sty`):** **neither engine converts the paper** — both raw-load
the xint engine (`xintexpr`→…→`xinttrig`) and fail. Perl **fails soft**: 39
errors via its `$MAXSTACK=200` recursion guard (`Core/Stomach.pm:169-178`
`invokeToken` "Excessive recursion(?)"), exits 0 with a **39-byte EMPTY**
`<document/>`. Rust **fails hard**: **stack overflow → SIGABRT (exit 134)**,
overflowing the conversion thread's **256 MB** stack (`latexml_oxide.rs:327`) — so
the recursion is genuinely *runaway* (Perl finishes the same work in 2.74s). The
overflow is in an xint-triggered gullet expansion: repeated
`read_number`/`\the` over `\XINT_expr_var_!` error tokens (xinttrig lines
~9253-9259) then `fatal runtime error: stack overflow, aborting`.

**Not** reproducible from a minimal `\usepackage{xintexpr}` ar5iv repro (that
completes with 8 errors); needs the full-paper tikz+xint cumulative context. NB
this is a *newer* symptom than the prior SYNC_STATUS record (bounded "FATAL at the
100-error cap" + pgffor self-ref) — intervening engine changes shifted it from a
bounded error-cap to an unbounded native recursion.

**Exact recursion cycle (gdb backtrace at SIGSEGV, 2026-06-20).** Period ~10
frames, driven by *number-argument reading* — NOT direct self-reference (so the
existing `expandable.rs` self-ref guard correctly does not fire):

```
read_number (gullet.rs:2018) → read_normal_integer (2062) → read_digits (2529)
  → read_x_token (908) → Expandable::invoke (expandable.rs:139)   [a tex_macro]
    → Parameters::read_arguments (parameter.rs:602) → Parameter::read (328)
      → base_parameter_types Number-reader (base_parameter_types.rs:169)
        → read_number ↺   (also via etex: etex_readexpr etex.rs:51/67/84
                            → read_value gullet.rs:1834 → read_x_token ↺)
```

i.e. an xint number-argument macro whose Number argument is read by expanding the
next number-argument macro, ~25 000 levels deep (×~10 KB/cycle ⇒ ~256 MB). Perl
survives because its `$MAXSTACK=200` `invokeToken` guard fires, Fatals, and is
caught → the 39-byte empty doc; Rust has no equivalent guard on this gullet path.

**Priority: low (reliability hardening, not a parity win)** — fixing the crash
would only make Rust fail-soft (empty doc) like Perl, NOT actually convert the
paper. **Faithful fix:** a Perl-style recursion-depth guard at the
`read_x_token`/`Expandable::invoke` chokepoint (a cheap thread-local depth counter
with an RAII dec — this path is hot, ~350k invokes in si.tex, so keep it a single
`usize` compare) that raises a Fatal-style recoverable error (propagated via the
existing `?`/Result chain, like `stomach::check_timeout`) before the 256 MB
overflow. **The threshold MUST be calibrated corpus-wide, not guessed:** between
the legitimate max expansion-nesting depth across real arXiv (normally ≪ a few
hundred) and the ~25 000 overflow — too low silently false-positive-Fatals a
legitimately deep paper (a coverage regression the 1459-test suite alone won't
catch); too high never prevents the crash (~5–10k is the plausible window). So a
dedicated session should first *measure* the depth distribution over a corpus
slice, then pick the limit, add the guard, and validate against the full suite
**plus** an arXiv slice. The cycle + chokepoint above make the implementation
straightforward; only the calibration is open work. Distinct from Cluster A's
*memory* (RSS-cap) blowups: this is *stack* (recursion-depth) exhaustion.

## Cluster G — vbox `\ht` is `\hsize`-invariant → `\hsize`-shrink loops never terminate (GENUINE Rust-only; OPEN)

**Witness:** `1707.02464` (`sandbox-arxiv-10k-shuffle`, fatal/Timeout/Convert).
Mined 2026-06-22 from the fatal cross-join: **Perl `latexml` 0.8.8 COMPLETES in
11.76s** (10 errors, exit 0) but **Rust hangs → `Fatal:Timeout:Convert`** (60s
watchdog; release, current HEAD; RSS only 168 MB so it's CPU/loop, not memory).
Not env, not parity — a real Rust-only divergence.

**Root cause (airtight).** The paper's custom `\narrow[#1]#2\par` macro
(used via `\disp`) line-fits a paragraph by shrinking `\hsize` 1pt at a time and
**looping while the vbox height is unchanged**, expecting the paragraph to
eventually wrap to one more line (taller box):
```
\loop \setbox\tmpbox\vbox{\hsize=\wd\tmpbox \advance\hsize by -1pt #2}%
      \ifdim\ht\tmpbox=\tmpdim \relax \repeat
```
Rust's `\vbox{\hsize=W …para…}` height does **not** depend on `W`. Direct probe:
a vbox at `\hsize=400pt` vs `\hsize=20pt` of the same 20-word paragraph compares
**equal** (`\ifdim\ht=\ht` true → `BUG:VBOX-HEIGHT-HSIZE-INVARIANT`). So the loop
condition never flips → it runs until the wall-clock guard. Threshold is ~18
words (below that the natural width is ≤ `\hsize`, `\ifdim\wd>` is false, the
loop is skipped). Minimal repro: preamble + `\disp{<≥18 words>}`.

**Why this is Rust-only (vs the SHARED Cluster D).** Cluster D loops on `\wd` of
a `\hbox{\font<glyph>}` which is 0 in BOTH engines (neither reads TFM glyph
widths) → both hang → Rust's graceful abort is *better*. Here the loop depends on
**paragraph height as a function of `\hsize`**, which **Perl LaTeXML models** (its
`\narrow` loop terminates → 11.76s) but **Rust does not** (no `\hsize`-aware
line-count/height estimate). Confirmed: removing the `$$`/math is still slow
(B), and a *fixed* 200-iteration vbox-remeasure loop is fast (C) — so it's the
non-terminating loop, not per-iteration math/box cost.

**ROOT CAUSE — PRECISE (2026-06-22).** The `\hsize`-aware paragraph-wrap machinery
**already exists and is correct**: `Font::compute_boxes_size_lines(wrapwidth, …)`
(`common/font.rs:1629`) breaks a line when `wd > wrapwidth` → more lines → taller
box, and `compute_boxes_size` derives `wrapwidth` from the paragraph List's
`width` property (set by `repack_horizontal`/`make_horizontal_list` =
`LookupDimension('\hsize')`). The bug is a **frame-ordering divergence** in
`base_utilities.rs::predigest_box_contents_in_mode`:
- Rust uses **TWO** frames — the mode frame (`begin_mode`) **plus** a body-group
  frame pushed by `invoke_token(&T_BEGIN!())`. The inner `\hsize=W` is assigned in
  the body-group frame, and the body's closing `}` (T_END) pops that frame
  (**restoring `\hsize` to the outer value**) BEFORE the manual
  `repack_horizontal_in_list(&mut item)` runs → `make_horizontal_list` captures the
  OUTER `\hsize` → every implicit vbox paragraph wraps at the outer width
  regardless of the inner `\hsize`.
- Perl uses **ONE** frame (the mode frame IS the group): `endMode`
  (`Stomach.pm:544-557`) runs `leaveHorizontal_internal`→`repackHorizontal`
  (capturing the inner `\hsize`, L487) and ONLY THEN `popStackFrame`. Rust's
  `end_mode` (`stomach.rs:726`) already mirrors this order — but it never sees the
  inner `\hsize` because the body-group frame already restored it.

**Fix spec (OPEN — focused, core-path).** Collapse the double-framing so the box
body's closing `}` drives `end_mode` (`leave_horizontal_internal`→repack, then pop)
rather than an early body-group egroup — i.e. let the mode frame BE the body group,
as Perl's `readBoxContents` (TeX_Box.pool.ltxml L139-160) does: a custom digest
loop that STOPS at the matching `}` (`level >= get_frame_depth()`) WITHOUT
processing it, then `end_mode` repacks (inner `\hsize` still in scope) + pops.

**PROTOTYPED + VALIDATED + REVERTED 2026-06-22.** Implemented the faithful loop in
`base_utilities.rs::predigest_box_contents_in_mode` for `mode.ends_with("vertical")`
only (hbox/math keep the `invoke_token(T_BEGIN)` path). **CONFIRMED it fixes Cluster
G**: 1707.02464 hang → **completes in 4.8s with PERFECT Perl parity** (10 errors,
byte-identical to local Perl's 10: 1 babel-russian env + 9 `Attempt to close a
group…`; the `\narrow` loop now terminates because `\ht` grows as `\hsize`
shrinks). BUT the naive loop **over-triggers** for general vboxes → 4 suite
regressions (1463/4), so REVERTED. The clean impl must additionally:
1. **Preserve VBoxContents reversion braces.** Rebuilding the body as
   `List::new(boxes)` loses the group `{}`, so `tex=` reverts `\vbox{a}` → `\vbox a`
   (box_test; Perl keeps `{a}`). The List must revert as a brace group.
2. **Not paragraph-wrap measured / non-paragraph boxes.** A short `\vbox` now
   measures at full `\hsize` (sizes_test 37.06pt → 469.75pt) because
   `compute_boxes_size_lines` returns `wrapwidth` for a fitting final line
   (`final_wd = ww if wd>0`, font.rs:1667) once the content is a width-tagged
   paragraph List. Perl measures natural width here — so either the box must not be
   repacked into a paragraph in this context, or the single-fitting-line width must
   be natural (not `ww`). Also affects graphrot_test, xytest_test (graphics sizing).
The core mechanism is PROVEN; these two interactions are the remaining work for a
focused session. NOTE: a `\loop`-iteration guard would make Rust *Fatal* where Perl
*succeeds* (worse) — the frame-ordering correction is the only faithful fix.

**SHARED ROOT with the global p{}→VBox port (found 2026-06-22).** The same
`\hsize`-invariant box model blocks SYNC_STATUS **1610.00974 step-3**: porting the
global `p{}` column to Perl's faithful `\lx@tabular@p`/VBoxContents form is
Perl-exact for non-rotated tables (3 fixtures become clean parity fixes, fixes the
genuine `<ltx:itemize> isn't allowed in <ltx:p>` correctness bug — 1510.07685) but
**rotated `p{}` cells mis-size** (graphrot `rotfloat2`: Rust 810pt vs Perl 214pt)
because the `p{1in}` cell **height** under `\hsize` is wrong — the very same gap.
One box-model fix (`\hsize`-constrained line-breaking feeding `compute_boxes_size`)
unblocks BOTH. Empirically verified + reverted; see SYNC_STATUS 1610.00974 step-3
and `docs/reproducers/pcolumn_block_content_in_p.tex`.

## Method notes

- Sweep failure logs: `~/data/large_scale_canvas_3/canvas/stage_*/failures/<id>.<KIND>.log`.
  The sweep's actual main file is in the log's `Processing content …/X.tex` line
  (ad-hoc largest-`.tex` picking diverges for multi-file papers).
- Math count is in the log's `MathML[Presentation] … N to process` line.
- Always `/usr/bin/time -v` for RSS; cap wall with `timeout` to stay safe.
