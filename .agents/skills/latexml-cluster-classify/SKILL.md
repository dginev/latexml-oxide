---
name: latexml-cluster-classify
description: Classify many failing corpus or sweep results into honest root-cause clusters before choosing fixes. Use after an arXiv stage, cortex run, telemetry batch, or log collection produces multiple failures that need grouping, representative sampling, prioritization, and handoff documentation.
---

# Classify a corpus failure set

A failing-paper count is a measurement, not a regression count. Grouping by a
headline diagnostic is only the start; sample against current same-host Perl
before assigning engineering scope.

Read `docs/THERMALS.md` before conversions. Do not overlap this work with a full
suite or another sweep, and do not add inner parallelism beyond the documented
job budget.

## Protocol

1. Inspect the results header and identify columns by name. Do not reuse a
   remembered numeric field position after a schema change.
2. Extract every non-success paper and its first non-cascade diagnostic. Strip
   ANSI before matching. Bucket missing logs, empty matches, fatals, timeouts,
   and OOMs explicitly; a parse miss is never a clean paper.
3. Tally the first-error classes, then list paper IDs for each candidate cluster.
   Verify that apparently identical undefined-control-sequence classes refer to
   the same control sequence and context.
4. Randomly sample 5 to 10 papers per substantial cluster. For a five-paper
   cluster, inspect all five. Run current Rust plus verbose same-host Perl,
   normally through:

```bash
shuf -n8 /tmp/cluster.txt | TIMEOUT_SECS=180 tools/parity_check.sh -
```

5. Interpret the sample:

| Sample | Cluster action |
|---|---|
| consistently Rust-only | root-cause one representative, then measure the halo |
| consistently shared | document or evaluate with `latexml-surpass-perl` |
| consistently Perl-only | preserve the Rust win; do not regress to parity |
| stale/currently clean | close the stale result |
| mixed | subgroup by the next diagnostic or actual mechanism and resample |
| Perl capped/timed out | retain uncertainty and inspect partial evidence |

6. Prioritize cluster-level fixes by legitimate affected-paper count and risk.
   A shared root fix for a large homogeneous cluster outranks unrelated
   single-paper patches.
7. After a fix, rerun the entire cluster. If only the chosen witness improves,
   the grouping or root-cause claim was wrong and must be narrowed.

## Durable record

Record active cluster state in the owning section of `docs/SYNC_STATUS.md`:

```text
Cluster: <diagnostic and semantic headline> (<size>)
Status: <Rust-only/shared/Perl-only/mixed>
Sample: <ids and verdict totals>
Root cause: <mechanism, when established>
Action: <next witness/fix/defer decision and expected halo>
```

Keep raw per-paper lists in the run artifacts rather than bloating durable docs.
Include exact evidence for capped, timed-out, or heterogeneous cases so a future
session does not turn uncertainty into a clean verdict.

