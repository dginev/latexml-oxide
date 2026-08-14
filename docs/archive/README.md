# Archived Documentation

Snapshot audits, completed-mission logs, and one-shot worksheets preserved for
forensic context. **Do not drive current planning from these** — revalidate on
current `HEAD` first; class/file layout, definitions, and counts have shifted.
Several are cited from `.rs` comments or the living docs (noted below); those
citations are why the file is kept.

> A 2026-08-14 compaction removed ~19 fully-superseded snapshots (the 2026-04
> line-by-line pool walks, the one-shot Def/Error/expl3/XML parity audits, two
> pre-PR review snapshots, and assorted bisections) plus their backing raw data.
> They remain in `git log`; recover with `git show`.

## Design & mission logs (referenced from live docs / code)

- `PERL_LOADFORMAT_AUDIT.md` — strict-`LoadFormat` dump-parity audit; mission
  complete (zero-error `--init`, dumps match Perl). Cited by `CLAUDE.md`,
  `dump_writer.rs`, `../parity/{DUMP_DESIGN,ORGANIZATION}.md`.
- `DUMP_FORMAT_PERL_ANALYSIS_2026-04-30.md` — close reading of Perl `Core/Dumper.pm`
  + the on-disk record format; the v3 structured-Parameter encoding it specifies is
  the live format. Cited by `dump_{reader,writer}.rs`, `../parity/DUMP_DESIGN.md`, `../parity/WISDOM.md`.
- `BIBTEX_PORT_PLAN_2026-06-20.md` — the BibTeX port plan (Phases 1–8 shipped;
  live residuals in `../SYNC_STATUS.md`). Cited by `bibtex.rs`.
- `frontmatter_api_refactor.md` — design/decisions log for the upstream PR #2767
  frontmatter-API port (landed). Cited by `../parity/OXIDIZED_DESIGN_DIVERGENCES.md`.
- `UPSTREAM_SYNC_2767_to_2833_2026-06-26.md` — per-PR catalog for the "translate
  upstream PRs since #2767" mission (U1–U11, all landed via PR #271).
- `MATHML_POST_LINE_AUDIT_2026-07-05.md` — exhaustive MathML-post line audit
  (every `MathML.pm` sub + 197 `DefMathML` regs vs Rust); sweep complete, open
  feature-gaps tracked in SYNC_STATUS.
- `MATH_AMBIGUITY_AUDIT_2026-05-21.md` — original math-ambiguity sweep; its live
  claims are superseded by `../math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md`. Cited
  by the math-parser sources.
- `XMLID_ACCESSOR_AUDIT_2026-06-08.md` — the libxml `xml:id`/`xml:lang`
  string-accessor footgun; active bugs fixed, broad migration deliberately NOT done.
  Cited by `document.rs`, `../parity/WISDOM.md` (#60).
- `MEMORY_GUARD_HARDENING_2026-06-09.md` — canvas OOM-cluster root cause + the
  layered runaway-guard architecture (resolved). Cited by `../performance/CORTEX_WORKER_HARNESS.md`.
- `POOL_PARITY_AUDIT.md` — `InnerPool!` invocation audit (completed). Cited by
  `latexml_engine/src/lib.rs`.
- `PORTABILITY_MACOS_PROBE_2026-06-07.md` — Phase-0 macOS dependency probe (#217,
  resolved; macOS now a gating CI job). Cited by `xslt.rs`, `../release/RELEASE_CRITERIA.md`.
- `ARXIV_FORK_AUDIT_2026-07-03.md` — due-diligence survey of the arXiv/LaTeXML
  velocity fork (items 1–4 landed). Cited by `../SYNC_STATUS.md`.
- `SANDBOX_TRIAGE_2026-05-21.md` — the 10k-sandbox triage workflow reference;
  judgement now lives in the `canvas-triage` skill. Cited by engine/package sources.
- `SCRIPT_BINDINGS_LOG_2026-06.md` — historical Rhai script-bindings progress log,
  split from `../parity/script_bindings_plan.md` (the live surface reference).
- `STARTUP_COST_ANALYSIS_2026-06-21.md` — ~161 ms startup decomposition + the
  DECLINED dump-parse lever; outcome carried by `../performance/PERFORMANCE.md`.

## `--server` editor LSP (landed PR #243, deprioritized — not stale)

- `LSP_SERVER.md` — design/status of the warm-preamble + fork-body server. Cited
  by `../README.md`, `../release/SAFETY.md`.
- `LSP_MULTIFILE_PLAN.md` — the multi-file project-root + overlay model (landed).
  Cited by `lsp_server/{project,overlay}.rs`.

## Session logs (completed "Landed this session" narratives)

- `SYNC_SESSIONS_2026-06.md`, `SYNC_SESSIONS_2026-07.md`, `SYNC_SESSIONS_2026-08.md`
  — completed worklist entries lifted out of the live `../SYNC_STATUS.md`
  (the `-08` file holds the 2026-07-09 … 07-27 landings, lifted 2026-08-14).
  Cited by SYNC_STATUS.
- `round19_iteration_log.md` — pre-Round-25 sprint narratives. Cited by `CHANGELOG.md`.
- `TRANSLATION_GAPS.md` — 2026-03 Perl→Rust function-gap snapshot (substantially
  resolved). Cited by `CHANGELOG.md`.
- `BABEL_TIMEOUT_BISECT.md` — 2026-04 babel/dump timeout bisection. Cited by
  `../parity/OXIDIZED_DESIGN_DIVERGENCES.md`.
- `sandbox_failures_SYNC_STATUS.md` — 2026-04 181-paper sandbox worksheet
  (superseded by `../SYNC_STATUS.md`). Cited by `binding/content.rs`.

## Raw data

- `sandbox_failure_181_triage.tsv`, `sandbox_failure_181.txt` — Round-18 181-paper
  triage rows, backing `sandbox_failures_SYNC_STATUS.md` / `frontmatter_api_refactor.md`.
