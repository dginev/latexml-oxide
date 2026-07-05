# Archived Documentation

Snapshot audits and one-shot worksheets preserved for forensic context.
Do not drive current planning from these files without revalidating on
current `HEAD` — class/file layout, definitions, and counts have shifted.

## 2026-07-05 `ar5iv-2606-prep` branch merge (PR #273, tag 0.7.2)

The branch merged to `main` and shipped as release 0.7.2 ("First public use
of latexml-oxide in ar5iv 2606"). Its two pre-merge review snapshots are
frozen here; live residuals were lifted into `../SYNC_STATUS.md`.

- `PR_READINESS_2026-07-05.md` — critical pre-PR review of the branch
  (105-commit risk filter + 3-agent cluster findings + ranked fix list).
- `COMMIT_REVIEW_2026-07-05.md` — adversarial overconfidence audit of the 79
  code commits from July 1–5 (6 parallel passes; disposition table). Its
  open follow-ups (lxDeclare font fidelity, gullet `read_balanced` locking
  test, foreignObject geometry-vs-text anchor split, bibliography
  field-interpretation parity) are tracked in `../SYNC_STATUS.md`. NOTE: it
  cites pre-squash commit SHAs — those resolve while the `ar5iv-2606-prep`
  branch ref survives; after the squash merge they are not on `main`'s
  first-parent history.

## 2026-07-02 docs consolidation

- `SYNC_SESSIONS_2026-06.md` — completed "Landed this session" logs
  (2026-06-22 … 2026-07-01), the slowest-100 batch #201–300 triage, the
  finished upstream-sync U1–U11 mission log, and the mined-out 2026-06
  cortex-cross-join methodology history — lifted out of the live
  `../SYNC_STATUS.md` (which keeps only current status + actionable items).
- `BIBTEX_PORT_PLAN_2026-06-20.md` — the BibTeX port plan; Phases 1–8
  shipped. Live residuals (Phase 4–5 field-handler tail, B1–B6, deferred
  native `.bst`) tracked in `../SYNC_STATUS.md` "Other tracks".
- `MATH_AMBIGUITY_AUDIT_2026-05-21.md` — the original ambiguity sweep;
  patterns 1/3/4 closed, and its remaining claims were **corrected** by the
  fresher `../MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md` (the live lever doc).
- `SANDBOX_TRIAGE_2026-05-21.md` — the 10k sandbox triage workflow
  reference; the judgement now lives in the `canvas-triage` skill
  (`.claude/skills/canvas-triage/`) and the failure-cluster classes in the
  per-cluster code comments that cite this file.
- `PERL_VS_RUST_FATAL_ANALYSIS_2026-06-19.md` — the tikz-cd/xy/tcolorbox
  3-sandbox fatal comparison (Rust ~3.5× better; residual divergences are
  deep tikz-cd/pgf/babel). Superseded as evidence by the 2026-07 full-arXiv
  rerun data; the tikz-cd verdict was re-confirmed 2026-07-02 (PARITY).
- `STARTUP_COST_ANALYSIS_2026-06-21.md` — the ~161 ms startup decomposition
  and the DECLINED ~50 ms dump-parse lever; outcome carried by
  `../PERFORMANCE.md` "Closed levers" (do not re-investigate).

## Resolved diagnostics & completed references (archived 2026-06-18)

- `MEMORY_GUARD_HARDENING_2026-06-09.md` — canvas_3 OOM-cluster root cause +
  the layered runaway-guard architecture (gullet/stomach cycle guards, the
  block-sampled byte budget, the boxing-depth cap). RESOLVED: the guards landed
  AND the witness cluster was root-cause-fixed (shipping `line`/`lcircle`
  fontmaps so the LaTeX-2.09 line-drawing loops terminate). Kept as the
  guard-design record / defense-in-depth reference. Cited by
  `../CORTEX_WORKER_HARNESS.md`.
- `PGF_ARC_BISECTION_2201.09268_2026-06-09.md` — pgf line–arc bisection
  non-termination (a 1e-5 last-place drift in the composed
  `\pgfmathanglebetweenpoints` makes pgf's exact-match loop exit miss in Rust).
  Root-caused; mitigated by the stomach cycle guard (clean `Fatal` instead of a
  4.5 GB OOM); the bit-exact-trig fix is deferred as deep/high-risk for one
  paper. Cited by `../SYNC_STATUS.md`.
- `XMLID_ACCESSOR_AUDIT_2026-06-08.md` — the libxml `xml:id`/`xml:lang`
  string-accessor footgun (stored namespaced as local `id`/`lang`, so the
  string-keyed `get/has/remove_attribute("xml:…")` silently fail). The three
  confirmed active bugs were fixed; a ratchet lint
  (`tools/lint_xmlid_accessor.sh` + baseline) and WISDOM #60 prevent new sites;
  the broad migration is deliberately NOT done (the masked sites are
  load-bearing — see the audit). Cited by `../EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`,
  `rewrite.rs`, `document.rs`, `dump`-adjacent comments.
- `DUMP_FORMAT_PERL_ANALYSIS_2026-04-30.md` — close reading of Perl
  `Core/Dumper.pm` and the on-disk record format. All implementation steps
  landed; the v3 structured-Parameter encoding it specifies is the stable live
  dump format (summarized in the living `../DUMP_DESIGN.md`, which links here).
  Cited by `dump_reader.rs` / `dump_writer.rs` / `../WISDOM.md`.

## Completed missions & resolved diagnostics (archived 2026-06-10)

- `UPSTREAM_SYNC_2767_to_2833_2026-06-26.md` — the per-PR catalog for the
  "translate brucemiller/LaTeXML PRs since #2767" mission (U1–U11, #2783 → #2833,
  incl. #2798 Leavehorizontal). All landed and merged to `main` via PR #271
  (`7869b5f459`); lifted out of the active `SYNC_STATUS.md` worklist on completion.
- `frontmatter_api_refactor.md` — design + decisions log for the upstream
  LaTeXML PR #2767 frontmatter-API port. Implemented and landed (commit
  `da495dd335`); kept as the historical design record.
- `PORTABILITY_MACOS_PROBE_2026-06-07.md` — Phase-0 macOS native-dependency
  probe for issue #217 (the kpathsea dichotomy → subprocess-`kpsewhich`
  spec). Issue RESOLVED 2026-06-08: full suite green on `macos-15` arm64,
  `kpathsea` 0.3.0 on crates.io, and the libxml2 merged-text-node
  use-after-free fixed (WISDOM #58); macOS is now a gating CI job.

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

## Script bindings (Rhai) — historical log

- `SCRIPT_BINDINGS_LOG_2026-06.md` — the M0 spike, M1/M2-M4 progress
  log, `\footnote`/DefEnvironment landing notes, and the two dated
  critical re-evaluations, archived from `docs/script_bindings_plan.md`
  (the live doc keeps the current surface reference).

## `--server` editor LSP (beyond-Perl; landed, deprioritized)

Archived 2026-06-05 to keep the top-level `docs/` focused on the parity
mission. These are NOT stale — they describe the shipped `--server`
code (PR #243) — they are just out of the current focus. Live smoke:
`tools/lsp_smoke.py`.

- `LSP_SERVER.md` — design/status of the warm-preamble + fork-body
  server: architecture, the PR #243 review records (code review
  2026-06-04, performance review 2026-06-05 incl. the stale-preamble
  fix), and the known-gaps worklist (unpreemptible warm-up, graphics
  CWD output, `.bib` overlay).
- `LSP_MULTIFILE_PLAN.md` — the multi-file project-root + overlay
  model (landed 2026-06-04), with implementation-delta notes.
  `lsp_server/{project,overlay}.rs` comments cite its §3A/§3B.

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
