# Memory-runaway guard hardening — root cause + WIP handoff (2026-06-09)

**Status: WORK IN PROGRESS, paused at a handoff point.** The code changes
described below are committed but INCOMPLETE (see "Remaining work"). They are
*inert for normal documents* (the new guard only activates past 200 k unflushed
boxes), so the suite is green (1403/0) and nothing regresses — but they do NOT
yet achieve the goal of firing earlier than the RSS cap on the witness paper.

## The question (user)

> Why do we have a cluster hitting OOM if we already guard the accumulation of
> Digested objects? Can we find more aberrant accumulation paths to guard? It is
> always better to Fatal early with a detected runaway RAM allocation, before we
> hit a RAM guard from outside.

## Witness

`~/data/canvas_3_failures_sandbox/zips/math0102053.zip` (also math0102089,
math0212126, math0504436, math0506088 — 5 of the 16 canvas_3 papers). A
**plain-TeX** document (171 KB, `\magnification`, custom macros, no
`\documentclass`) whose `\@whiledim` line-drawing loop (file line 37) builds
~1.87 M nested `\hbox{\raise…\hbox{\lower…\setbox…}}` boxes. **Perl OOMs too**
(rc=124, 3m19s, hit a 6 GB vmem cap — backtrace: `\@iwhile…` → nested
`\hbox`/`\raise`/`\lower`/`\setbox`). So this is a SHARED pathological runaway,
not a Rust-only correctness gap — it is a reliability-hardening case.

## Root cause: why the existing guards didn't fire early

The guards (added earlier 2026-06-09, see [[feedback_oom_cycle_guards]]):
1. **Gullet cycle guard** — periodicity over the read-token stream → infinite *expansion*.
2. **Stomach cycle guard** — periodicity over box pushes (window ≤10 repeated ≥100×) → infinite *digestion*.
3. **Stomach box hard-cap** (`STOMACH_BOX_HARD_CAP = 2_000_000`) — aperiodic box-count backstop.
4. **RSS soft cap** (4.5 GB, `check_timeout`, reads `/proc/self/statm`) — Linux-only, last resort.

Why each missed it:
- **Cycle guard (periodicity):** the loop is APERIODIC. Each iteration draws at a
  different position/dimension, so the *content-aware* `Digested::cycle_fingerprint`
  (deliberately content-aware to avoid false positives) produces a DIFFERENT
  fingerprint each iteration → no repeating window → silent. This is the core
  trade-off: content-aware fingerprints avoid false positives but cannot catch a
  counter-varying loop.
- **Box hard-cap (2 M COUNT):** the boxes are HEAVY (~2.4 KB each as measured by
  RSS: 4.5 GB / 1.87 M). They reached 1.87 M < 2 M when RSS hit the 4.5 GB cap, so
  the count cap never fired. **The cap is a byte-proxy calibrated for LIGHT boxes
  (~600 B); for heavy boxes 2 M ≈ 4.8 GB, i.e. above the RSS ceiling, so it is
  effectively unreachable.**
- **RSS cap (4.5 GB):** fired — correctly, gracefully, well below the 46 GB
  cgroup / system OOM-killer. So the kernel is protected; the system never
  crashes. The "OOM cluster" is really "RSS-cap-fired-at-4.5 GB cluster."

**First-principles diagnosis:** our internal guards measure box COUNT as a proxy
for the resource that matters (BYTES), but per-box weight varies several-fold, so
a count cap cannot bound memory. AND the RSS cap is Linux-only — on macOS/Windows
(portability track, RELEASE_CRITERIA) there is NO memory guard at all for a
heavy-box runaway.

## Aberrant accumulation paths found

1. **`stomach.rs:341` (`box_list.extend(filtered)`)** — the group-flush path
   (`<beforeAfterGroup>`, the path a grouped drawing loop flushes through) wrote
   straight to `box_list`, BYPASSING the guarded `extend_box_list`. **FIXED**:
   now routed through `extend_box_list` so the cycle/count/byte guards see it.
2. **`localized_box_list` (the boxing stack)** — when a loop builds *inside* a box
   (`\setbox`/`\hbox`), `new_local_box_list` swaps the partial outer list onto the
   localized stack and starts a fresh `box_list`. The membudget dump caught
   `box_list=340, localized_box_list_total=1_870_928` at one point — i.e. the
   accumulation can live entirely in the boxing stack, which **no guard watches.**
   STILL UNGUARDED (see Remaining work).
3. **`stomach.rs:874`** (`box_list.push` in the paragraph-flush) — one box per
   paragraph, not a runaway path; left as-is.

## What was done (this WIP commit)

- `Digested::estimate_bytes()` (digested.rs) — a depth-bounded (`EB_BUDGET=256`)
  STRUCTURAL byte estimate (counts the `Rc`/`RefCell`/`Vec` of each box + nested
  children; text is interned/shared so not counted).
- Stomach **byte-budget guard** (stomach.rs): in the runaway path (past
  `STOMACH_CYCLE_ACTIVATE`), every `BYTE_CHECK_EVERY=50_000` boxes past
  `BYTE_CHECK_ACTIVATE=200_000`, estimate the box-list footprint by **block
  sampling** (32 contiguous blocks × 256 boxes = `BYTE_SAMPLE_N=8192`, robust to
  clustered heavy boxes) and `Fatal:Timeout:MemoryBudget` if it exceeds
  `STOMACH_BOX_BYTES_BUDGET` (currently 1.6 GB). Portable (no `/proc`).
- Routed `stomach.rs:341` through the guarded `extend_box_list` (path #1 above).

## Remaining work (resume here)

**The byte estimate UNDERCOUNTS RSS by ~3.7×, so the guard does NOT yet fire
before the RSS cap on the witness.** Measured: the block-sampled estimate is
smooth + monotonic but plateaus at **~1.2 GB while RSS is 4.5 GB** (≈ 639 B/box
structural vs ≈ 2.4 KB/box RSS). The missing cost is each box's `properties`
HashMap + allocator overhead, which the structural estimate ignores.

1. **Make `estimate_bytes` count the `properties` HashMap** of `Whatsit`
   (`whatsit.rs:30 properties: HashMap<Stored>`) and `List` (`list.rs:24
   properties: HashMap<Stored>`) — roughly `entries × (48 + value_size)`. This
   should lift the estimate toward ~2–3 GB at 1.87 M boxes, close enough to RSS
   that a budget BELOW the 4.5 GB cap fires first. Also consider a flat
   allocator-overhead multiplier (RSS/est ≈ 2–4×) instead of perfect accounting.
2. **Recalibrate `STOMACH_BOX_BYTES_BUDGET`** once the estimate tracks RSS, so it
   `Fatal`s at ~3.5–4 GB-equivalent (before the 4.5 GB RSS cap) WITHOUT
   false-positives on legitimate large figures.
3. **Guard the `localized_box_list` (boxing stack)** — path #2. Either sum its
   byte estimate into the check, or cap the boxing-stack depth / total box count.
   A loop building 1.87 M-deep nested boxes is never legitimate; a depth/total
   cap is a clean portable signal.
4. **Reconsider lowering `STOMACH_BOX_HARD_CAP`** (2 M → ~1.5 M) as a *portable*
   early signal (the RSS cap is Linux-only). RISK: a legitimate huge tikz/pgfplots
   figure could hold 1.5–2 M boxes (~3.6–4.8 GB) and complete under the RSS cap;
   lowering would kill it. Cannot assess without canvas source (the 551 k corpus
   at `~/data/large_scale_canvas_3_third` is output-only — see
   [[project_large_scale_corpus_location]]). Defer until testable, or keep 2 M.
5. **Validate**: once the estimate is fixed, confirm the byte guard fires on
   math0102053 BEFORE the RSS cap (target ~3.5 GB), and run the full suite +ideally
   a canvas sample to confirm no legitimate large document is newly killed.
6. **Add unit tests** for `estimate_bytes` (monotonicity, bounded cost) mirroring
   the `cycle_fingerprint` tests in digested.rs.

## Files touched

- `latexml_core/src/digested.rs` — `estimate_bytes()` + `EB_BUDGET`.
- `latexml_core/src/stomach.rs` — byte-budget guard + sampling helper +
  constants; `:341` routed through `extend_box_list`; membudget debug prints the
  estimate.

Suite: **1403/0/0** (guards inert for normal docs). Not pushed.
