# 10k sandbox triage — fresh worklist

Run: `~/data/sandbox_full_2026-04-24/` (binary `b9bc02155`,
post-1112.6246 fix). Output dir: 7898 inputs, results.tsv = 7898 rows.

## Top-level distribution

| Status | Count | % | Notes |
|---|---:|---:|---|
| `ok` | **7363** | **93.2%** | converts error-free (some may carry recoverable warnings) |
| `conversion_error` | 497 | 6.3% | exit ≠ 0 from cortex_worker, finite wall time |
| `abort` | 35 | 0.4% | exit 134 (SIGABRT) — assertion / panic / OOM |
| `conversion_fatal` | 3 | 0.04% | 10001-error mode-leak cascade |

Per the user directive, only **papers Perl latexml converts
error-free** are in scope. Cross-check each cell below against
`latexml --strict --quiet` on the same source before opening a Rust
fix; ignore the rest.

## Hard failures (priority 1) — 3 fatals + 35 aborts

### Fatal (10001-error cascade)

| Paper | Memory note |
|---|---|
| `1803.03288` | triaged off worklist — modern xparse 2018+/pgfplots/cleveref cascade; Perl also struggles |
| `1902.08705` | triaged off worklist — pgfmath `\ifdim` empty-token loop; Perl also times out |
| `hep-ph0702114` | babel-french `\bbl@exp@aux` infinite loop — see `project_babel_francais_gap.md` |

### Aborts (exit 134, SIGABRT) — 35 papers

Full list in `/tmp/worklist/abort.txt`. First batch:

```
0711.4787      0903.3289      1210.1891      1308.1148      1310.6857
1312.5864      1403.4135      1406.1495      1407.5769      1502.06237
1506.09203     1510.06919     1510.08290     1511.07586     1602.02322
1607.06257     1707.01155     1710.03688     1709.05096     1710.04068
…
```

Many of these were timeout-territory in earlier runs (1210.1891,
1308.5727, 1407.5769, 1709.05096, 1707.01155, 1711.10191, 1711.11576,
1710.03688) — recheck under the current binary; some may have moved
into hard-fail territory after recent changes.

## conversion_error (priority 2) — 497 papers

Stored in `/tmp/worklist/conv_err.txt` for the time being; will be
folded into this doc once the cross-check against Perl narrows the
list to "Rust-actionable only".

Two papers in this bucket exceeded 60 s wall time and look like
disguised timeouts rather than emit errors:
- `1606.08766` (wall ≫ 60 s)
- `1708.04706` (wall ≫ 60 s)

## Investigation cadence

1. For each candidate paper, confirm Perl latexml converts it
   error-free first; if Perl also fails, defer (`KNOWN_PERL_ERRORS.md`).
2. Cluster Rust-only failures by primary error category
   (undefined / malformed / unexpected / abort signature).
3. Pick the highest-yield cluster, identify the binding/engine gap,
   port from Perl. Update `WISDOM.md` if the fix exposes a new
   tactical insight.

Detailed per-paper logs: `~/data/sandbox_full_2026-04-24/<paper>.log`.
