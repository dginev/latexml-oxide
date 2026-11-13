---
name: latexml-start-session
description: Ground a fresh latexml-oxide session in current repository state before choosing work. Use when resuming the project, asking what to work on, continuing a sweep, or diagnosing whether a remembered status is still current.
---

# Start a latexml-oxide session

Use current repository evidence, not imported memory, to establish state.

## Grounding reads

Run these independent reads together:

```bash
sed -n '1,140p' docs/SYNC_STATUS.md
git log --oneline -20
git status --short
sed -n '1,180p' docs/README.md
```

Then read the documentation owning the user's request. In particular:

- performance or memory: `docs/performance/PERFORMANCE.md` and the latest audit;
- parity or a failing paper: `docs/parity/WISDOM.md`, then use
  `latexml-canvas-triage`;
- current perfect-kernel work: `docs/perfect_kernel/README.md` and its ledger;
- release work: `docs/release/RELEASING.md` and
  `docs/release/RELEASE_CRITERIA.md`.

Read `docs/THERMALS.md` before starting any heavy workload.

## Interpret carefully

- Verify a stale-looking status against its named test, current source, or live
  issue before acting on it. This repository squash-merges, so ancestry alone is
  not a reliable status check.
- Treat a dirty worktree as shared work. Identify relevant changes and preserve
  everything unrelated.
- Confirm referenced tools and symbols still exist with `rg` before following a
  historical recipe.
- Address the user's current request after grounding. Do not turn startup into a
  broad audit when the conversation already supplies current context.

## Common routes

| Request | Next workflow |
|---|---|
| Benchmark or profile a conversion | `latexml-perf-check` |
| Classify a failing paper | `latexml-canvas-triage` |
| Shrink a confirmed Rust-only failure | `latexml-min-repro` |
| Port or correct a binding | `latexml-perl-port` |
| Diagnose dump versus raw loading | `latexml-dump-debug` |
| Classify a sweep | `latexml-cluster-classify` |
| Handle a public ticket | `latexml-resolve-issue` |

