# Archived Documentation

Snapshot audits and one-shot worksheets preserved for forensic context.
Do not drive current planning from these files without revalidating on
current `HEAD` — class/file layout, definitions, and counts have shifted.

## Pre-Round-25 archive

- `BABEL_TIMEOUT_BISECT.md` — 2026-04-26 babel/dump timeout bisection.
- `TRANSLATION_GAPS.md` — 2026-03-15 Perl→Rust function-gap snapshot;
  substantially resolved by Round-21.
- `sandbox_failures_SYNC_STATUS.md` — 2026-04-26 focused 181-paper
  sandbox worksheet; superseded by `../SYNC_STATUS.md`.
- `SYNC_STATUS_2026-04-30_pre-tasklist.md` — pre-tasklist `SYNC_STATUS`.
- `round19_iteration_log.md` — pre-Round-25 sprint narratives.

## Line-by-line audits (2026-04 walks, all RESOLVED)

The line-by-line walks of `Engine/*.pool.ltxml` vs `latexml_engine/src/*.rs`.
Each ran to completion and the actionable findings landed as commits.

- `LATEX_CONSTRUCTS_LINE_AUDIT.md` (6,014-line pool walk, 26 phases).
- `LATEX_BASE_LINE_AUDIT.md` (865-line pool).
- `PLAIN_BASE_LINE_AUDIT.md` (622-line pool).
- `PLAIN_CONSTRUCTS_LINE_AUDIT.md` (322-line pool).
- `LATEX_BOOTSTRAP_LINE_AUDIT.md`, `PLAIN_BOOTSTRAP_LINE_AUDIT.md`.

## LoadFormat / dump-parity mission (completed)

- `PERL_LOADFORMAT_AUDIT.md` — the strict-`LoadFormat` dump-parity audit.
  Mission complete: zero-error `--init=plain.tex`/`latex.ltx`, dumps match
  Perl line-for-line, eager-vs-lazy LaTeX load resolved (`tex.rs:213`). The
  one live residual (~72-CS Perl-only long tail) is an active item in
  `../SYNC_STATUS.md` "Engine file open gaps (MINOR)".

## Parity audits (one-shot, completed)

- `LATEX_CONSTRUCTS_PARITY_AUDIT.md` — Rust 54%-larger investigation.
- `DEF_PARITY_AUDIT.md` — `Def*!` macro-kind parity, FULLY TRIAGED.
- `EXPL3_PARITY_AUDIT.md` — 2026-04-26 expl3 strict-parity audit.
- `POOL_PARITY_AUDIT.md` — `InnerPool!` invocation audit, completed.
- `ERROR_PARITY_AUDIT.md` — 2026-05-03 Error/Fatal parity verification.
- `PERL_XML_DIFFS.md` — 2026-04-19 `LaTeXML/t/*.xml` ↔ Rust XML diffs.
- `rewrite_subsystem_audit.md` — Rewrite.pm ↔ rewrite.rs (snapshot).

## Performance snapshots

- `TIKZ_DIGEST_HOTSPOTS_2026-05-21.md` — 2026-05-16 callgrind profiling of
  TikZ/pgfplots digestion (research-only). Live handoff items folded into
  `../PERFORMANCE.md`; reusable bucketing script at
  `../scripts/bucket_callgrind_hot.py`.

## Round-18 broken-paper snapshot

- `BROKEN_PAPERS.md` — 2026-05-01 confirmed Rust regressions list;
  superseded by Round-25 deferred set in `../SYNC_STATUS.md`.

## Raw data

- `def_parity_engine.tsv`, `def_parity_package.tsv`,
  `def_parity_contrib.tsv` — TSV row backing the `DEF_PARITY_AUDIT`.
- `parity_data/` — `latex_combined_perl_only.txt` / `*_rust_only.txt`
  parity-set snapshots feeding the Round-25 audits.
- `sandbox_failure_181_triage.tsv`, `sandbox_failure_181.txt` —
  Round-18 181-paper triage rows.
