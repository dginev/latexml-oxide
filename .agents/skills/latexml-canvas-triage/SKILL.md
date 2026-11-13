---
name: latexml-canvas-triage
description: Classify a failing or suspicious arXiv conversion as a current Rust-only regression, shared Perl failure, Perl regression, environment artifact, stale result, or deferred case. Use before fixing conversion errors, fatals, malformed output, rendering differences, cortex candidates, or canvas failures.
---

# Classify a conversion before fixing it

The output of this workflow is a classification. Only a current, same-host
Rust-only failure is a normal parity-fix candidate.

## Signal rules

- Re-run the current Rust binary. Corpus and cortex data are candidate screens
  and may predate the fix already in the checkout.
- Run Perl on the same host and TeX tree, verbosely. Never use `latexml --quiet`
  for classification because it can suppress the evidence being counted.
- Prefer cortex `Status:conversion:N` or the ANSI-free on-disk `.latexml.log`.
  If grepping stderr, strip color first:

```bash
sed 's/\x1b\[[0-9;]*m//g' run.log | grep -acE '^(Error|Fatal):'
```

- A missing or unparsable log is a failure to investigate, not a clean result.
- TeX package/version differences between hosts are environment artifacts, not
  Rust regressions. The project does not ship the host's `.def`/`.ldf` ecosystem.
- Never reduce severity, suppress diagnostics, or relax a guard to manufacture a
  better verdict.

## Workflow

1. Reproduce Rust on the current binary and capture output, status, and a log.
   `tools/triage_failure.sh <arxiv-id>` provides a full backtrace path;
   `tools/first_error.sh <log>` finds the first non-cascade error.
2. Reproduce verbose Perl on the same extracted source and host.
3. When applicable, run `TIMEOUT_SECS=180 tools/parity_check.sh <arxiv-id>` and
   inspect both underlying logs rather than trusting only the label.
4. Assign one verdict:

| Verdict | Meaning | Action |
|---|---|---|
| BOTH-CLEAN | Both current runs are clean | Close stale candidate |
| SHARED-FAILURE | Same mechanism/severity fails in both | Record or evaluate with `latexml-surpass-perl` |
| PERL-REGRESSION | Rust is correct where Perl fails | Preserve Rust behavior and document durable significance |
| ENV-ARTIFACT | Different host package/version explains delta | Do not patch Rust |
| RUST-ONLY | Perl is clean, Rust fails on same host | Reduce and fix |
| DEFERRED | Owning docs explicitly defer the family | Leave it in the owning worklist |

5. For RUST-ONLY, preserve the exact first-error canary, use
   `latexml-min-repro`, read the Perl source with `latexml-perl-port`, and validate
   both the reduced case and original witness.

For structural output differences, compare semantics and the documented
intentional normalization, not only error counts.

