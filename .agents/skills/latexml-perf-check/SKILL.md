---
name: latexml-perf-check
description: Measure and analyze latexml-oxide time or memory performance with valid same-host A/B evidence. Use for benchmarks, profiling, slow or OOM conversions, build-profile choices, performance regressions, algorithmic complexity, allocation reduction, or proposed optimization work.
---

# Measure latexml-oxide performance

Read these first:

1. `docs/performance/PERFORMANCE.md` for the measurement contract and closed
   levers.
2. `docs/performance/PERFORMANCE_AUDIT_2026-09-03.md` for ranked current findings
   and handoff boundaries.
3. `docs/performance/STABILITY_WITNESSES.md` for timeout, OOM, and RSS witnesses.
4. `docs/THERMALS.md` before builds, suites, or corpus runs.

## Choose the correct profile

| Profile | Use |
|---|---|
| default test/dev | local development, correctness triage, backtraces |
| `ci` | GitHub runner only |
| `release` | optimized corpus sweeps and production-like Perl comparisons |
| `bench` | profiling and benchmarks; symbols retained, binary in `target/release/` |
| `maxperf` | distribution artifact only |

Do not benchmark an unoptimized development binary. Do not use `maxperf` for an
ordinary iteration cycle.

## Measurement protocol

1. State one falsifiable hypothesis and choose witnesses that exercise the
   suspected path. Read the owning source before selecting a metric.
2. Check the performance docs for an already measured dead end. PGO, native ISA
   tuning, and dump/startup work are closed absent a changed architecture or new
   contrary evidence.
3. Isolate one implementation lever. Build baseline and candidate under the same
   profile and environment.
4. Run them back-to-back on the same host. Prefer interleaved best-of-three runs
   on a quiescent machine. Do not compare a historical idle run to a current busy
   run.
5. Use the existing tooling where it fits:

```bash
tools/run_perf_corpus.sh
tools/perf_compare.py <baseline-telemetry> <candidate-telemetry>
tools/perf_phase_summary.py <telemetry>
```

6. Compare exact output bytes or the repository's documented normalized parity,
   exit/status, diagnostic counts and severities, phase timings, wall time, CPU
   time, and peak RSS. A faster result with changed output or hidden errors is a
   correctness regression unless an intentional divergence was authorized.
7. Report distributions and per-witness regressions, not only a favorable total.
   Separate compile time, startup, engine, post-processing, and serialization so
   an aggregate does not hide the actual bottleneck.

## Performance bug checks

- Look for superlinear scans, repeated full-tree traversals, unbounded retention,
  retry loops, accidental cloning, owned-string creation on probes, and work done
  after a terminal status.
- For memory, distinguish peak live data from cumulative allocation. Establish
  ownership and lifetime before proposing a representation change.
- Preserve the self-contained-binary invariant. An optimization may not replace
  embedded project resources with runtime filesystem dependencies.
- A plain `cargo test` `MemoryBudget` cascade that disappears at
  `--test-threads=1` can be aggregate process RSS. Use
  `cargo nextest run --workspace` for the supported process-isolated suite.

## Handoff

Record the exact revisions/binaries, profile, machine state, input set, commands,
raw telemetry paths, parity checks, uncertainty, rejected alternatives, and the
single next experiment. Put durable conclusions in the owning performance doc;
do not promote a one-off reading into policy.

