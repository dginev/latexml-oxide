# Startup-cost analysis — 2026-06-21

Measured on the dedicated box (AMD Threadripper 9980X, 64c/128t Zen5, 246 GB).
Timings pinned (`taskset -c 112-127`), medians of ≥5 runs, on the `maxperf`
binary unless noted. Companion to `PERFORMANCE.md`. Records a measured lever and
the decision **not** to pursue it, so it is not re-investigated from scratch.

## Decomposition (trivial `\documentclass{article}` doc)

Per-process floor — paid on EVERY paper in the cortex fleet
(one-conversion-per-process). Decomposed with env-gated `Instant` probes in
`dump_reader.rs` + `core_interface.rs::initialize_singletons` (probes reverted
after measurement).

| phase | ms (maxperf) | notes |
|---|---|---|
| process+link+runtime init | ~6 | negligible |
| bootstrap: TeX.pool + plain dump (1 ms) + TeX `_constructs` | ~9 | `model::initialize_model()` (RelaxNG) is ~1 µs — schema is NOT a startup cost |
| **latex dump load** | **~85** | loaded during digestion at `\documentclass`; **24k records**; **parse+build 60% (~53 ms) / apply 40% (~32 ms)** |
| latex `_constructs` + digest + build + serialize | ~80 | for a trivial body, dominated by `_constructs` (compiled-in Rust constructors → State) |
| **total** | **~161** | NODUMP path is **~98 ms** (−63 ms): most of the kernel is compiled-in, so the dump's apply is largely **redundant re-write** kept for strict-Perl unconditional-apply parity |

Key facts:
- The **dump load is text-parsing-bound**: re-tokenizing 24k macro bodies
  (`parse_token_list`), `url_decode`, integer parse. State maps are already
  capacity-reserved (`meaning`: 131072, FxHashMap), so apply is near-optimal.
- The dump is **kept for parity** (the ~72-CS Perl-only tail + Perl-faithful
  serializable token bodies + unconditional apply), not for speed — NODUMP is
  faster but loses that parity guarantee.
- `model::initialize_model()` is ~1 µs — the RelaxNG schema is **not** a startup
  cost (it is consulted lazily, not parsed up front).

## Lever (measured, NOT pursued)

"Halving" the 161 ms floor is not cleanly reachable — two ~co-equal fixed costs
(latex dump ~85 ms, latex `_constructs` ~80 ms). The biggest **clean** lever is
the dump's ~53 ms text-parse, via either:
- **Binary dump format** (bincode/rkyv instead of TAB-text) — removes the text
  tokenization; symbols are runtime-assigned (arena IDs unstable across runs) so
  strings still re-intern at load → apply (~32 ms) stays. Predicted dump
  85→~35 ms, startup 161→~110 ms.
- **Parallel-parse + serial-apply** — parse 24k records text→structs across rayon
  threads, apply serially in dump order (State is thread-local). No format
  change, lower parity risk. Predicted dump 85→~40 ms, startup 161→~115 ms.

Each yields a realistic **~50 ms (~30%) startup cut**, also ~50 ms off every
LaTeX paper's latency (~10% of the 507 ms median).

**DECISION (2026-06-21, user): do NOT pursue.** ~50 ms / ~10%-of-median is too
small to justify a release-critical change to the dump pipeline + the parity
re-validation it would force. (A prior attempt to compile the dumps into the
binary as generated Rust code — vs the current gzip-blob embed — lifted compile
time ~10 min for no runtime win, and was abandoned.) The ~53 ms dump text-parse
and the ~80 ms `_constructs` cost are both **accepted as-is**. Revisit only if a
future change makes per-process startup a dominant fleet cost.

## Repro
Harnesses + raw results under `/home/deyan/pgo_bench/` (STATUS.md). Corpus
`/data/arxiv_shuffle_1902/`. Probes were temporary (git-reverted).
