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

## Method notes

- Sweep failure logs: `~/data/large_scale_canvas_3/canvas/stage_*/failures/<id>.<KIND>.log`.
  The sweep's actual main file is in the log's `Processing content …/X.tex` line
  (ad-hoc largest-`.tex` picking diverges for multi-file papers).
- Math count is in the log's `MathML[Presentation] … N to process` line.
- Always `/usr/bin/time -v` for RSS; cap wall with `timeout` to stay safe.
