# 10k sandbox triage — post-session worklist

Latest run: `~/data/sandbox_full_2026-04-26_postfix/` (binary
`3088dbd17` with the `endgroup()` INTERPRETING_DEFINITIONS
suppression fix). Prior: `~/data/sandbox_full_2026-04-25b/`
(`188ed64ee`).

## Top-level distribution (2026-04-26, post-fix)

| Status | Count | % | Δ from 2026-04-25b |
|---|---:|---:|---:|
| `ok` | **7538** | **95.44%** | **+29** |
| `conversion_error` | 322 | 4.08% | -38 |
| `timeout` | 31 | 0.39% | +8 |
| `conversion_fatal` | 5 | 0.06% | +1 |
| `abort` | 2 | 0.03% | 0 |
| **Total failures** | **360** | **4.56%** | **-29** |

Net session: +29 ok, -29 total failures. The endgroup() fix unblocks
~29 papers that previously halted on the false-positive `\group_end:`
mode-switch error during expl3-code.tex raw load. Some bordering
papers shifted from conv_err → timeout/fatal as their loads now
run further before hitting other resource limits.

Older 2026-04-25b distribution shown for reference:

## Top-level distribution (2026-04-25b, run complete)

| Status | Count | % | Δ from 2026-04-25 first run |
|---|---:|---:|---:|
| `ok` | **7509** | **95.07%** | **+12** |
| `conversion_error` | 360 | 4.56% | -9 |
| `timeout` | 23 | 0.29% | (new category — was `abort`) |
| `conversion_fatal` | 4 | 0.05% | +1 |
| `abort` | 2 | 0.03% | -27 |
| **Total failures** | **389** | **4.93%** | **-12** |

Most aborts re-classified as timeouts after `1c85a137f` (timeout
alignment fix). Net session improvement modest but consistent.

### Dominant cluster (highest-leverage next target)

**93+ conv_err papers + 8 ok-but-warned** all share the pattern:
```
Error:unexpected:\group_end: Attempt to close a group that switched
to mode horizontal; current frame is mode-switch to horizontal due to
```
triggered at `latexml_core/src/stomach.rs:284` from `endgroup()`'s
`is_value_bound("BOUND_MODE", Some(0))` check. Reproducer locations:
`xparse-2018-04-12.sty:1762-1776` and `lipsum.sty:401`. See
`project_explsyntax_midload.md` memory for the full hypothesis trail
and next-cycle fix candidates (instrument `begin_mode_opt` to find
the mid-load BOUND_MODE binder; or relax the strict frame-0 check
when in raw .sty load context).

## Older 2026-04-25 distribution (for reference)

| Status | Count | % | Δ from 2026-04-24 |
|---|---:|---:|---:|
| `ok` | **7497** | **94.9%** | **+134** |
| `conversion_error` | 369 | 4.7% | -128 |
| `abort` | 29 | 0.37% | -6 |
| `conversion_fatal` | 3 | 0.04% | 0 |
| **Total failures** | **401** | **5.1%** | **-134 (25% reduction)** |

Earlier 2026-04-24 distribution shown for reference:

| Status | Count | % |
|---|---:|---:|
| `ok` | 7363 | 93.2% |
| `conversion_error` | 497 | 6.3% |
| `abort` | 35 | 0.4% |
| `conversion_fatal` | 3 | 0.04% |

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
1710.03688).

### Session 2026-04-25 progress on aborts

Re-ran the 35-paper abort subset against post-session binary
(commit `188ed64ee`). Result: **10 papers cleared** (28% reduction).

Still aborting (25):

```
0711.4787   0903.3289   1407.5769   astro-ph0612758  hep-ph9210253
hep-ph9512208 hep-th0101151 math0505371 math9204211 (math_parser
stack overflow — see #17 finalize-phase)

1210.1891   1709.05096  1710.04068  1711.11576  1806.06448
math0005251 math9805021 math9810139 alg-geom9604001
alg-geom9604020 alg-geom9703018 (real timeouts — likely throughput
limits, not bugs)

1710.03688 hep-ph0702114 (babel-french — see project_babel_francais_gap.md)
```

Cleared (10): mostly former timeouts that finished within the 90 s
budget on the current binary. Notable: 1308.1148 (5918 maths) and
1310.6857 now complete.

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
