# Sandbox Failures Worksheet — 181 papers

Tracks per-cluster Rust→Perl translation gaps for the focused
~/data/sandbox_failures sandbox of error-producing papers. Each
row tracks the cluster size, root cause, fix approach, and
status.

Workflow: edit code → rebuild → `./tools/rerun_failures.sh` →
diff `~/data/sandbox_failures_<TS>/results.tsv` against the saved
baseline `docs/sandbox_failure_181_triage.tsv` → mark recovered
papers with `[x]`.

## Initial baseline (post-AR-flip, 2026-04-26)

`results.tsv` totals: 5119 Status:0 + 2598 Status:1 + 172 Status:2 +
3 Status:3 + 6 empty = **181 problem papers** (97.71% clean of
7898). Cluster shape captured in `docs/sandbox_failure_181_triage.tsv`.

## Active investigation tracks

### Track A — Plain TeX dump coverage gap

**Symptom.** Plain-TeX papers using `\settabs N \columns` (5 papers:
`astro-ph9308008, astro-ph9708022, funct-an9711006, hep-th9404085,
q-alg9505016`) error with `\columns undefined`.

**Root cause.** Verified 2026-04-26:
- Perl `\meaning\settabs` returns the full plain.tex chain
  (`\setbox\tabs\null \futurelet\next\sett@b`), defined at
  `/usr/local/texlive/2025/texmf-dist/tex/plain/base/plain.tex:602`.
- The Rust `latex.dump.txt` has **zero** `\settabs`/`\sett@b`/
  `\sett@bb`/`\s@tt@b` entries.
- The Rust `--init=latex.ltx` pipeline (`latexml_oxide/src/ini_tex.rs`
  L67-95) raw-loads ONLY `latex.ltx`. Perl's format build implicitly
  loads `plain.tex` first (TeX format build convention).
- `latex.ltx` itself does not redefine `\settabs`.
- Net: Perl's format file has `plain.tex`'s definitions baked in;
  Rust's dump is missing them entirely.

**Fix approach.** In `ini_tex.rs`, raw-load `plain.tex` BETWEEN the
state snapshot and the `latex.ltx` raw load. The diff then captures
the plain.tex chain, which `dump_writer` serializes into
`latex.dump.txt`.

Status: in progress (this turn).

### Other clusters (181 - 5 = 176 remaining, deferred behind Track A)

| Cluster | Papers | Class breakdown | Notes |
|---|---|---|---|
| `XMApp` in `<ltx:text>` | 19 | mixed | task #11 — math-parser shape |
| `XMTok` in `<ltx:text>` | 11 | mixed | task #11 — math-parser shape |
| `\regex_const:Nn` (mhchem/expl3) | 11 | various | task #11 — expl3 regex |
| `XMApp` in `<ltx:p>` | 7 | mixed | task #11 — math-parser shape |
| `\end{equation}` mode mismatch | 7 | mixed | math env close |
| `}` brace mismatch | 6 | mixed | gullet/parameter |
| `\columns` (plain-TeX) | 5 | (plain) | **Track A** |
| `\section` (AmSTeX dispatch) | 4 | amsppt | `project_amstex_pool_dispatcher.md` |
| `\@nil` (pgf cascade) | 4 | mixed | pre-existing pgf catcode |
| `\gnuplot` (gnuplot.sty) | 4 | mixed | per-package |
| `\+` undefined | 3 | mixed | LaTeX tabbing CS gap |
| `\columns` undefined | 3 | mixed | plain-TeX (subset of Track A) |
| `\CITE` undefined | 3 | mixed | custom .sty per-paper |
| `<box> was supposed to be here` | 3 | mixed | brace mismatch |
| `\affil` undefined | 3 | revtex | per-paper |
| `\lx@end@gen@cases` | 3 | mixed | amsmath cases |
| `XMArray` in `<ltx:para>` | 3 | mixed | math-parser shape |
| Other singletons + per-class | ~95 | mostly article | long-tail |

## Fix log

| Date | Commit | Cluster | Papers cleared | Total problem |
|---|---|---|---|---|
| 2026-04-26 (baseline) | — | — | 0 | 181 |

(Append rows here after each run.)
